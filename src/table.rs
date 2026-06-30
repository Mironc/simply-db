use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{common_types::Schema, row::Row};

#[derive(Debug)]
pub struct Table {
    rows: RwLock<Vec<Row>>,
    schema: Schema,
}

impl Table {
    pub fn new(schema: Schema) -> Self {
        Self {
            rows: RwLock::new(Vec::new()),
            schema,
        }
    }
    pub fn insert_row(&self, row: Row) -> Result<(), TableError> {
        if !row.data().validate(&self.schema) {
            return Err(TableError::SchemaMismatch);
        }
        self.rows_mut().push(row);
        Ok(())
    }
    /// Returns read guard over table's rows
    ///
    /// While not dropped, all writers are blocked
    pub fn rows<'a>(&'a self) -> RwLockReadGuard<'a, Vec<Row>> {
        self.rows.read().expect("Poisoned table")
    }
    /// Returns write guard over table's rows
    ///
    /// While not dropped, all readers and other writers are blocked
    pub fn rows_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Vec<Row>> {
        self.rows.write().expect("Poisoned table")
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}
#[derive(Debug)]
pub enum TableError {
    /// Schema in table doesn't match with inserted row
    SchemaMismatch,
}
