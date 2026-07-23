use std::ops::Deref;

use storage::{
    common_types::{DataValue, ScalarType, ScalarValue},
    db::Database,
};

use crate::{
    context::Context,
    expr::{Expr, ExprError},
};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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
        expected: ScalarType,
        given: DataValue,
    },
    /// Filter expression returns not bool
    BadExpr,
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
            let context = Context::new(row.data(), &schema);
            if let Some(expr) = &self.filter_expr {
                if let DataValue::Scalar(ScalarValue::Bool(b)) = expr.execute(&context)?.deref() {
                    if !b {
                        continue;
                    }
                } else {
                    return Err(UpdateError::BadExpr);
                }
            }
            let mut values = Vec::new();
            for (set_field, set_expr) in self.set_exprs().iter() {
                let res = set_expr.execute(&context)?.into_owned();
                if let Some(field) = schema.fields().iter().find(|x| x.0 == *set_field) {
                    if let DataValue::Scalar(scalar_value) = &res
                        && field.1.data_type() == scalar_value.scalar_type()
                    {
                        values.push((set_field, res));
                    } else {
                        if matches!(res, DataValue::Null) && !field.1.is_nullable() {
                            if !field.1.is_nullable() {
                                return Err(UpdateError::NullExpection {
                                    field: set_field.clone(),
                                });
                            } else {
                                values.push((set_field, DataValue::Null));
                            }
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
                let set_idx = schema
                    .fields()
                    .get_index(set_field)
                    .expect("Expected valid set_field");
                *(row
                    .data_mut()
                    .get_mut(set_idx)
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
        common_types::ScalarType,
        db::Database,
        row::Row,
        scalar,
        schema::{FieldModifier, FieldType, Schema},
    };
    use structures::VecMap;

    use crate::{
        expr::{Expr, LiteralValue},
        queries::{
            create_table::CreateTable,
            insert::InsertQuery,
            update::{UpdateError, UpdateQuery},
        },
    };

    fn init_db() -> Database {
        let mut db = Database::new();

        // Create a table with one field
        let mut field_types = VecMap::new();
        field_types.insert(
            "age".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::NotNull]),
        );
        let schema = Schema::new(field_types);

        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let row = Row::new(vec![scalar!(Int(10))]);
        let insert_table = InsertQuery::new("table".to_string(), vec!["age".to_owned()], vec![row]);
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
    fn bad_filter_expression() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("age".to_string(), Expr::Literal(LiteralValue::Int(10)))],
            Some(Expr::Literal(LiteralValue::Int(10))),
        );
        assert_eq!(update.execute(&mut db), Err(UpdateError::BadExpr));
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
