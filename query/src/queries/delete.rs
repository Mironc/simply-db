use std::ops::Deref;

use storage::{
    common_types::{DataValue, ScalarValue},
    db::Database,
};

use crate::expr::{Expr, ExprError};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum DeleteError {
    UnknownTable(String),
    ExprErr(ExprError),
}
impl From<ExprError> for DeleteError {
    fn from(v: ExprError) -> Self {
        Self::ExprErr(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteQuery {
    DropTable(DropTable),
    DeleteRows(DeleteRows),
    TruncateTable(TruncateTable),
}

impl DeleteQuery {
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        match self {
            DeleteQuery::DropTable(drop_table) => drop_table.execute(db),
            DeleteQuery::DeleteRows(delete_rows) => delete_rows.execute(db),
            DeleteQuery::TruncateTable(truncate_table) => truncate_table.execute(db),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DropTable {
    table: String,
}

impl DropTable {
    pub fn new(table: String) -> Self {
        Self { table }
    }
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        if db.delete_table(&self.table).is_none() {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteRows {
    table: String,
    expr: Expr,
}
impl DeleteRows {
    pub fn new(table: String, expr: Expr) -> Self {
        Self { table, expr }
    }
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        let table = if let Some(table) = db.get_table(&self.table) {
            table
        } else {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        };
        let mut rows = Vec::new();
        for (i, row) in table.rows().iter().enumerate() {
            if let DataValue::Scalar(ScalarValue::Bool(val)) = self.expr.execute(row)?.deref()
                && *val
            {
                rows.push(i);
            }
        }
        for (i, row_idx) in rows.iter().enumerate() {
            table.rows_mut().remove(row_idx - i);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TruncateTable {
    table: String,
}

impl TruncateTable {
    pub fn new(table: String) -> Self {
        Self { table }
    }

    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        let table = if let Some(table) = db.get_table(&self.table) {
            table
        } else {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        };
        table.rows_mut().clear();
        Ok(())
    }
}
