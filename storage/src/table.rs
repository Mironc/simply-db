use std::{
    collections::HashSet,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

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
    /// Inserts single row. Returns nothing on success.
    ///
    /// # Errors
    /// # Errors
    /// returns `SchemaMismatch` if row fields do not match with the schema in the table.
    /// returns `UniqueConstraint` if some unique field value is already present in the table.
    pub fn insert_row(&self, row: Row) -> Result<(), TableInsertError> {
        self.validate_row(&[], &row)?;
        self.rows_mut().push(row);
        Ok(())
    }
    /// Inserts multiple rows at once. Returns nothing on success.
    ///
    /// # Errors
    /// returns `SchemaMismatch` if row fields do not match with the schema in the table.
    /// returns `UniqueConstraint` if some unique field value is already present in the table or in the other rows.
    pub fn insert_rows(&self, rows: Vec<Row>) -> Result<(), TableInsertError> {
        let mut validated_rows = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            self.validate_row(&rows[i + 1..rows.len()], row)?;
            validated_rows.push(row.clone());
        }
        for row in validated_rows.into_iter() {
            self.rows_mut().push(row);
        }
        Ok(())
    }
    /// Validates row against table and other rows in one insert request
    ///
    /// # Errors
    /// returns `SchemaMismatch` if row fields do not match with the schema in the table.
    /// returns `UniqueConstraint` if some unique field value is already present in the table or in the other rows.
    pub fn validate_row(&self, other_rows: &[Row], row: &Row) -> Result<(), TableInsertError> {
        if !row.data().validate(&self.schema) {
            return Err(TableInsertError::SchemaMismatch);
        }
        let table_rows = self.rows();
        for field in self.schema.fields().iter() {
            let row_field = row.data().fields().get(&field.0);
            if field.1.is_unique() {
                for cmp_row in table_rows.iter().chain(other_rows.iter()) {
                    let cmp_value = cmp_row.data().fields().get(&field.0);
                    if row_field == cmp_value {
                        return Err(TableInsertError::UniqueConstraint);
                    }
                }
            }
        }
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
impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            rows: RwLock::new(self.rows().clone()),
            schema: self.schema.clone(),
        }
    }
}
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableInsertError {
    /// Schema in table doesn't match with inserted row
    SchemaMismatch,
    /// Duplicated values
    UniqueConstraint,
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate as storage;
    use storage::{
        common_types::{FieldModifier, FieldType, Schema, SchemaValue},
        hashmap,
        row::Row,
        scalar, scalar_type,
        table::{Table, TableInsertError},
    };

    #[test]
    fn insert_unique_constraint_error() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::Unique]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        table.insert_row(Row::new(SchemaValue::new(value))).unwrap();
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        let res = table.insert_row(Row::new(SchemaValue::new(value)));
        assert_eq!(res, Err(TableInsertError::UniqueConstraint))
    }

    #[test]
    fn insert_multiple_unique_constraint_error() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::Unique]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        let value1 = hashmap!("id".to_owned() => scalar!(Int(0)));
        let rows = vec![
            Row::new(SchemaValue::new(value)),
            Row::new(SchemaValue::new(value1)),
        ];
        let res = table.insert_rows(rows);
        assert_eq!(res, Err(TableInsertError::UniqueConstraint))
    }

    #[test]
    fn insert_multiple_unique_single_row_success() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::Unique]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        let rows = vec![Row::new(SchemaValue::new(value))];
        let res = table.insert_rows(rows);
        assert_eq!(res, Ok(()))
    }

    #[test]
    fn insert_multiple_unique_success() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::Unique]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        let value1 = hashmap!("id".to_owned() => scalar!(Int(1)));
        let rows = vec![
            Row::new(SchemaValue::new(value)),
            Row::new(SchemaValue::new(value1)),
        ];
        let res = table.insert_rows(rows);
        assert_eq!(res, Ok(()))
    }

    #[test]
    fn insert_single_unique_multiple_success() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::Unique]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = hashmap!("id".to_owned() => scalar!(Int(0)));
        let value1 = hashmap!("id".to_owned() => scalar!(Int(1)));
        let res = table.insert_row(Row::new(SchemaValue::new(value)));
        let res1 = table.insert_row(Row::new(SchemaValue::new(value1)));
        assert_eq!(res, Ok(()));
        assert_eq!(res1, Ok(()));
    }

    #[test]
    fn insert_field_mismatch_error() {
        let field_type = FieldType::new(scalar_type!(Int), vec![FieldModifier::NotNull]);
        let schema = Schema::new(vec![("id".to_owned(), field_type)]);
        let table = Table::new(schema);
        let value = HashMap::new();
        let res = table.insert_row(Row::new(SchemaValue::new(value)));
        assert_eq!(res, Err(TableInsertError::SchemaMismatch))
    }
}
