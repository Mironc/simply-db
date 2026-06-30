use std::borrow::Cow;

use crate::{
    common_types::{DataValue, ScalarValue, Schema},
    db::Database,
    row::Row,
    sql::expr::{Expr, ExprError},
};
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum SelectError {
    /// Table not found
    NoTable {
        table: String,
    },
    ExprErr(ExprError),
}
impl From<ExprError> for SelectError {
    fn from(err: ExprError) -> Self {
        SelectError::ExprErr(err)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    Row,
    Expr(Vec<Expr>),
}

impl Projection {
    pub fn execute(&self, row: &Row, schema: &Schema) -> Result<Vec<DataValue>, ExprError> {
        Ok(match self {
            Projection::Row => {
                let row_data = row.data().fields();
                schema
                    .fields()
                    .iter()
                    .map(|x| row_data.iter().find(|y| x.0 == *y.0).unwrap().1.clone()) // Ordering by fields
                    .collect::<Vec<DataValue>>()
            }
            Projection::Expr(exprs) => {
                let mut res = Vec::new();
                for expr in exprs.iter() {
                    let value = expr.execute(row)?;
                    match value {
                        Cow::Borrowed(b) => res.push(b.clone()),
                        Cow::Owned(o) => res.push(o),
                    };
                }
                res
            }
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SelectQuery {
    table: String,
    projection: Projection,
    filter_expr: Option<Expr>,
}

impl SelectQuery {
    pub fn new(table: String, projection: Projection, filter_expr: Option<Expr>) -> Self {
        Self {
            table,
            projection,
            filter_expr,
        }
    }

    pub fn execute(&self, db: &Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
        let table = db.get_table(&self.table).ok_or(SelectError::NoTable {
            table: self.table.to_owned(),
        })?;
        let mut projected = Vec::new();
        for row in table.rows().iter() {
            if let Some(filter) = &self.filter_expr {
                if let DataValue::Scalar(ScalarValue::Bool(val)) = *filter.execute(row)?
                    && val
                {
                    projected.push(self.projection.execute(row, table.schema())?);
                }
            } else {
                projected.push(self.projection.execute(row, table.schema())?);
            }
        }
        Ok(projected)
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn table_name(&self) -> &str {
        &self.table
    }

    pub fn filter_expr(&self) -> Option<&Expr> {
        self.filter_expr.as_ref()
    }
}
