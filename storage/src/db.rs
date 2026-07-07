//! Contains `Database` structure
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::table::Table;

#[derive(Debug, Default)]
pub struct Database {
    tables: RwLock<HashMap<String, Arc<Table>>>,
}
impl Database {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert_table(&self, name: String, table: Table) -> Option<()> {
        let mut write = self.tables.write().expect("DB lock poisoned:");
        if write.contains_key(&name) {
            return None;
        }
        write.insert(name, Arc::new(table));
        Some(())
    }
    pub fn get_table(&self, name: &str) -> Option<Arc<Table>> {
        let read = self.tables.read().expect("DB lock poisoned:");
        read.get(name).cloned()
    }
    /// Deletes table, returns `Some(())` if table was deleted and was present in database
    pub fn delete_table(&self, name: &str) -> Option<()> {
        let mut write = self.tables.write().expect("DB lock poisoned:");
        write.remove(name)?;
        Some(())
    }
    pub fn tables<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<String, Arc<Table>>> {
        self.tables.read().expect("DB lock poisoned:")
    }
    pub fn has_table(&self, name: &str) -> bool {
        self.tables().contains_key(name)
    }
}
