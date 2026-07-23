use std::borrow::Cow;

use storage::{
    common_types::{DataValue, ScalarValue},
    db::Database,
};

use crate::{
    context::Context,
    expr::{Expr, ExprError},
};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum SelectError {
    /// Table not found
    NoTable {
        table: String,
    },
    ExprErr(ExprError),
    BadExpr,
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
    pub fn execute(&self, context: &Context) -> Result<Vec<DataValue>, ExprError> {
        Ok(match self {
            Projection::Row => {
                if let Some(fields) = context.fields() {
                    fields.clone()
                } else {
                    Vec::new()
                }
            }
            Projection::Expr(exprs) => {
                let mut res = Vec::new();
                for expr in exprs.iter() {
                    let value = expr.execute(context)?;
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
            let context = Context::new(row.data(), table.schema());
            if let Some(filter) = &self.filter_expr {
                match *filter.execute(&context)? {
                    DataValue::Scalar(ScalarValue::Bool(val)) => {
                        if val {
                            projected.push(self.projection.execute(&context)?);
                        }
                    }
                    DataValue::Null => continue,
                    _ => return Err(SelectError::BadExpr),
                }
            } else {
                projected.push(self.projection.execute(&context)?);
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
