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
    UnexpectedNull {
        field: String,
    },
    SetTypeMismatch {
        expected: ScalarType,
        given: DataValue,
    },
    UniqueConstraint {
        field: String,
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
        let mut row_modifications = Vec::new();

        let filter_function: Box<dyn for<'a> Fn(&Context<'a>) -> Result<bool, UpdateError>> =
            if let Some(expr) = &self.filter_expr {
                Box::new(|context: &Context<'_>| {
                    if let data_value = expr.execute(&context)?.deref()
                        && let DataValue::Scalar(ScalarValue::Bool(b)) = data_value
                    {
                        Ok(*b)
                    } else {
                        return Err(UpdateError::BadExpr);
                    }
                })
            } else {
                Box::new(|_: &Context<'_>| Ok(true))
            };

        let mut prepared_updates = Vec::new();
        for (set_field, set_expr) in self.set_exprs().iter() {
            let (field_id, (field_name, field)) = table
                .schema()
                .fields()
                .iter()
                .enumerate()
                .find(|x| x.1.0 == *set_field)
                .ok_or_else(|| UpdateError::NoField {
                    field: set_field.clone(),
                })?;
            prepared_updates.push((field_id, field_name, field, set_expr));
        }

        for (i, row) in table.rows().iter().enumerate() {
            let context = Context::new(row.data(), table.schema());
            if !filter_function(&context)? {
                continue;
            }
            let mut modifications = Vec::with_capacity(prepared_updates.len());

            for &(field_id, field_name, field, set_expr) in prepared_updates.iter() {
                let res = set_expr.execute(&context)?.into_owned();
                // Uniqueness check
                if field.is_unique() {
                    if table.rows().iter().enumerate().any(|(id, row)| {
                        row.data()[field_id] == res
                            && row_modifications
                                .binary_search_by(|x: &(usize, Vec<(usize, DataValue)>)| {
                                    x.0.cmp(&id)
                                })
                                .is_err()
                            && id != i
                    }) || row_modifications.iter().any(|x| {
                        x.1.iter()
                            .any(|(mod_field_id, val)| *mod_field_id == field_id && val == &res)
                    }) {
                        return Err(UpdateError::UniqueConstraint {
                            field: field_name.to_owned(),
                        });
                    }
                }

                if let DataValue::Scalar(scalar_value) = &res
                    && field.data_type() == scalar_value.scalar_type()
                {
                    modifications.push((field_id, res));
                } else {
                    if res == DataValue::Null {
                        if field.is_nullable() {
                            modifications.push((field_id, DataValue::Null));
                            continue;
                        } else {
                            return Err(UpdateError::UnexpectedNull {
                                field: field_name.clone(),
                            });
                        }
                    }
                    return Err(UpdateError::SetTypeMismatch {
                        expected: field.data_type(),
                        given: res.clone(),
                    });
                }
            }

            row_modifications.push((i, modifications));
        }
        for (i, new_row) in row_modifications.into_iter() {
            for (field_id, modification) in new_row.into_iter() {
                table.rows_mut()[i].data_mut()[field_id] = modification;
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
    use std::ops::Deref;

    use storage::{
        common_types::{DataValue, ScalarType, ScalarValue},
        db::Database,
        row::Row,
        scalar,
        schema::{FieldModifier, FieldType},
    };
    use structures::VecMap;

    use crate::{
        expr::{ArithmeticOp, ComparisonOp, Expr, ExprError, LiteralValue},
        queries::{
            create_table::CreateTable,
            insert::InsertQuery,
            update::{UpdateError, UpdateQuery},
        },
    };

    fn init_db() -> Database {
        let mut db = Database::new();

        let mut field_types = VecMap::new();
        field_types.insert(
            "id".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::PrimaryKey]),
        );
        field_types.insert(
            "name".to_string(),
            FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]),
        );
        field_types.insert(
            "email".to_string(),
            FieldType::new(ScalarType::Text, vec![]),
        );

        let create_table = CreateTable::new("table".to_string(), field_types, false);
        create_table.execute(&mut db).unwrap();

        let row = Row::new(vec![
            scalar!(Text("Alice")),
            scalar!(Int(0)),
            scalar!(Text("alice@email.com")),
        ]);
        let row1 = Row::new(vec![
            scalar!(Text("Bob")),
            scalar!(Int(1)),
            scalar!(Text("bob@email.com")),
        ]);
        let insert_table = InsertQuery::new(
            "table".to_string(),
            vec!["name".to_owned(), "id".to_owned(), "email".to_owned()],
            vec![row, row1],
        );
        insert_table.execute(&mut db).unwrap();
        db
    }

    #[test]
    fn unknown_table_error() {
        let mut db = Database::new();

        let update = UpdateQuery::new("nonexistent_table".to_string(), vec![], None);
        assert!(update.execute(&mut db).is_err());
    }

    #[test]
    fn missing_field_error() {
        let mut db = init_db();
        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("field".to_string(), Expr::Literal(LiteralValue::Null))],
            None,
        );
        assert_eq!(
            update.execute(&mut db).unwrap_err(),
            UpdateError::NoField {
                field: "field".to_owned()
            }
        );
    }

    #[test]
    fn unexpected_null_error() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("name".to_string(), Expr::Literal(LiteralValue::Null))],
            None,
        );
        assert_eq!(
            update.execute(&mut db).unwrap_err(),
            UpdateError::UnexpectedNull {
                field: "name".to_owned()
            }
        );
    }

    #[test]
    fn type_mismatch_error() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "id".to_string(),
                Expr::Literal(LiteralValue::Text("Alice".to_owned())),
            )],
            None,
        );
        assert_eq!(
            update.execute(&mut db).unwrap_err(),
            UpdateError::SetTypeMismatch {
                expected: ScalarType::Int,
                given: DataValue::Scalar(ScalarValue::Text("Alice".to_owned()))
            }
        );
    }

    #[test]
    fn expression_propagation_error() {
        let db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Arithmetic(Box::new(ArithmeticOp::Add(
                    Expr::Field("name".to_string()),
                    Expr::Literal(LiteralValue::Int(1)),
                ))),
            )],
            None,
        );
        assert_eq!(
            update.execute(&db).unwrap_err(),
            UpdateError::ExprErr(ExprError::NotApplicable)
        );

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Literal(LiteralValue::Text("Steve".to_string())),
            )],
            Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("name".to_string()),
                Expr::Literal(LiteralValue::Int(1)),
            )))),
        );
        assert_eq!(
            update.execute(&db).unwrap_err(),
            UpdateError::ExprErr(ExprError::NotApplicable)
        );
    }
    #[test]
    fn expression_returns_not_bool_error() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Literal(LiteralValue::Text("Steve".to_owned())),
            )],
            Some(Expr::Literal(LiteralValue::Int(10))),
        );
        assert_eq!(update.execute(&mut db).unwrap_err(), UpdateError::BadExpr);
    }

    #[test]
    fn unique_constraint_error() {
        let db = init_db();
        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("id".to_string(), Expr::Literal(LiteralValue::Int(0)))],
            None,
        );
        assert_eq!(
            update.execute(&db).unwrap_err(),
            UpdateError::UniqueConstraint {
                field: "id".to_owned()
            }
        );
    }

    #[test]
    fn update() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Literal(LiteralValue::Text("Steve".to_owned())),
            )],
            None,
        );
        assert!(update.execute(&mut db).is_ok());
        assert_eq!(
            db.get_table("table").unwrap().rows().deref(),
            &vec![
                Row::new(vec![
                    scalar!(Int(0)),
                    scalar!(Text("Steve".to_string())),
                    scalar!(Text("alice@email.com"))
                ]),
                Row::new(vec![
                    scalar!(Int(1)),
                    scalar!(Text("Steve".to_string())),
                    scalar!(Text("bob@email.com".to_string()))
                ])
            ]
        );
    }

    #[test]
    fn update_with_filter() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "name".to_string(),
                Expr::Literal(LiteralValue::Text("Steve".to_owned())),
            )],
            Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("id".to_owned()),
                Expr::Literal(LiteralValue::Int(0)),
            )))),
        );
        assert!(update.execute(&mut db).is_ok());
        assert_eq!(
            db.get_table("table").unwrap().rows().deref(),
            &vec![
                Row::new(vec![
                    scalar!(Int(0)),
                    scalar!(Text("Steve".to_string())),
                    scalar!(Text("alice@email.com"))
                ]),
                Row::new(vec![
                    scalar!(Int(1)),
                    scalar!(Text("Bob".to_string())),
                    scalar!(Text("bob@email.com".to_string()))
                ])
            ]
        );
    }

    #[test]
    fn update_with_unique() {
        let db = init_db();
        let update = UpdateQuery::new(
            "table".to_string(),
            vec![(
                "id".to_string(),
                Expr::Arithmetic(Box::new(ArithmeticOp::Add(
                    Expr::Field("id".to_owned()),
                    Expr::Literal(LiteralValue::Int(2)),
                ))),
            )],
            None,
        );
        assert!(update.execute(&db).is_ok());
        assert_eq!(
            db.get_table("table").unwrap().rows().deref(),
            &vec![
                Row::new(vec![
                    scalar!(Int(2)),
                    scalar!(Text("Alice".to_string())),
                    scalar!(Text("alice@email.com"))
                ]),
                Row::new(vec![
                    scalar!(Int(3)),
                    scalar!(Text("Bob".to_string())),
                    scalar!(Text("bob@email.com".to_string()))
                ])
            ]
        );
    }
    #[test]
    fn update_set_null() {
        let mut db = init_db();

        let update = UpdateQuery::new(
            "table".to_string(),
            vec![("email".to_string(), Expr::Literal(LiteralValue::Null))],
            None,
        );
        update.execute(&mut db).unwrap();
        assert_eq!(
            db.get_table("table").unwrap().rows().deref(),
            &vec![
                Row::new(vec![
                    scalar!(Int(0)),
                    scalar!(Text("Alice")),
                    DataValue::Null,
                ]),
                Row::new(vec![
                    scalar!(Int(1)),
                    scalar!(Text("Bob".to_string())),
                    DataValue::Null,
                ])
            ]
        );
    }
}
