use std::ops::Deref;

use storage::{
    common_types::{DataType, DataValue, ScalarValue},
    db::Database,
};

use crate::expr::{Expr, ExprError};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum UpdateError {
    ExprErr(ExprError),
    NoTable {
        table: String,
    },
    NoField {
        field: String,
    },
    NullExpection {
        field: String,
    },
    SetTypeMismatch {
        expected: DataType,
        given: DataValue,
    },
    /// Filter expression returns not bool
    FilterExpr,
}

impl From<ExprError> for UpdateError {
    fn from(v: ExprError) -> Self {
        Self::ExprErr(v)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateQuery {
    table: String,
    set: Vec<(String, Expr)>,
    filter_expr: Option<Expr>,
}
impl UpdateQuery {
    pub fn new(table: String, set: Vec<(String, Expr)>, filter_expr: Option<Expr>) -> Self {
        Self {
            table,
            set,
            filter_expr,
        }
    }
    pub fn execute(&self, db: &Database) -> Result<(), UpdateError> {
        let table = db.get_table(&self.table).ok_or(UpdateError::NoTable {
            table: self.table.to_owned(),
        })?;
        let schema = table.schema().clone();
        for row in table.rows_mut().iter_mut() {
            if let Some(expr) = &self.filter_expr {
                if let DataValue::Scalar(ScalarValue::Bool(b)) = expr.execute(row)?.deref() {
                    if !b {
                        continue;
                    }
                } else {
                    return Err(UpdateError::FilterExpr);
                }
            }
            let mut values = Vec::new();
            for (set_field, set_expr) in self.set_exprs().iter() {
                let res = set_expr.execute(row)?.into_owned();
                if let Some(field) = schema.fields().iter().find(|x| x.0 == *set_field) {
                    if res.validate(field.1.data_type(), field.1.is_nullable()) {
                        values.push((set_field, res));
                    } else {
                        if matches!(res, DataValue::Null) && !field.1.is_nullable() {
                            return Err(UpdateError::NullExpection {
                                field: set_field.clone(),
                            });
                        }
                        return Err(UpdateError::SetTypeMismatch {
                            expected: field.1.data_type().clone(),
                            given: res.clone(),
                        });
                    }
                } else {
                    return Err(UpdateError::NoField {
                        field: set_field.clone(),
                    });
                }
            }
            for (set_field, value) in values.into_iter() {
                *(row
                    .data_mut()
                    .fields_mut()
                    .get_mut(set_field)
                    .expect("Should be a valid field")) = value.clone();
            }
        }
        Ok(())
    }
    pub fn table_name(&self) -> &str {
        &self.table
    }

    pub fn set_exprs(&self) -> &[(String, Expr)] {
        &self.set
    }

    pub fn filter_expr(&self) -> Option<&Expr> {
        self.filter_expr.as_ref()
    }
}
#[cfg(test)]
mod test {
    use storage::{
        common_types::{FieldType, Schema, SchemaValue},
        db::Database,
        hashmap, scalar, scalar_type,
    };

    use crate::{
        expr::{Expr, LiteralValue},
        queries::{create_table::CreateTable, insert::InsertQuery, update::UpdateQuery},
    };

    fn init_db() -> Database {
        let mut db = Database::new();

        // Create a table with one field
        let mut field_types = Vec::new();
        field_types.push(("age".to_string(), FieldType::new(scalar_type!(Int), false)));
        let schema = Schema::new(field_types);

        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let schema = SchemaValue::new(hashmap!("age".to_string()=>scalar!(Int(10))));
        let insert_table = InsertQuery::new("table".to_string(), vec![schema]);
        insert_table.execute(&mut db).unwrap();
        db
    }
    #[test]
    fn unknown_table() {
        let mut db = Database::new();

        let update = UpdateQuery::new(
            "nonexistent_table".to_string(),
            vec![("age".to_string(), Expr::Literal(LiteralValue::Int(10)))],
            None,
        );
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn missing_field() {
        let mut db = init_db();
        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Literal(LiteralValue::Text("Alice".to_owned())),
            )],
            None,
        );
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn null_value_on_non_nullable_field() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("age".to_string(), Expr::Literal(LiteralValue::Null))],
            None,
        );
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn type_mismatch() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "age".to_string(),
                Expr::Literal(LiteralValue::Text("Alice".to_owned())),
            )],
            None,
        );
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn filter_expression_returns_non_bool() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("age".to_string(), Expr::Literal(LiteralValue::Int(10)))],
            Some(Expr::Literal(LiteralValue::Int(10))),
        );
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn success() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("age".to_string(), Expr::Literal(LiteralValue::Int(30)))],
            None,
        );
        assert!(update.execute(&mut db).is_ok());
    }
}
