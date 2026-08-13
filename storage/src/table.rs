use crate::{
    common_types::{DataValue, ScalarValue},
    row::Row,
    schema::Schema,
};
use std::{
    collections::HashSet,
    sync::{
        RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Debug)]
pub struct Table {
    rows: RwLock<Vec<Row>>,
    schema: Schema,
    counter: AtomicU64,
}

impl Table {
    pub fn new(schema: Schema) -> Self {
        Self {
            rows: RwLock::new(Vec::new()),
            schema,
            counter: AtomicU64::new(0),
        }
    }
    /// Inserts single row. Returns nothing on success.
    ///
    /// # Errors:
    /// - returns `SchemaMismatch` if row fields do not match with the schema in the table.
    ///
    /// - returns `UniqueConstraint` if some unique field value is already present in the table.
    pub fn insert_row(&self, field_names: &[String], mut row: Row) -> Result<(), TableInsertError> {
        let index_map = if let Some(index_map) = self.schema.build_index_map(field_names) {
            index_map
        } else {
            return Err(TableInsertError::SchemaMismatch);
        };
        self.validate_row(
            &index_map,
            &mut Vec::with_capacity(self.schema.fields().len()),
            &[],
            &mut row,
        )?;
        if let Some(ai_idx) = self.schema.autoincrement_field() {
            let next_id = self.counter.fetch_add(1, Ordering::SeqCst);
            row.data_mut()[ai_idx] = DataValue::Scalar(ScalarValue::Int(next_id as i32));
        }
        self.rows_mut().push(row);
        Ok(())
    }
    /// Inserts multiple rows at once. Returns nothing on success.
    ///
    /// # Errors:
    /// - returns `SchemaMismatch` if row fields do not match with the schema in the table.
    ///
    /// - returns `UniqueConstraint` if some unique field value is already present in the table or in the other rows.
    pub fn insert_rows(
        &self,
        field_names: &[String],
        mut rows: Vec<Row>,
    ) -> Result<(), TableInsertError> {
        let index_map = if let Some(index_map) = self.schema.build_index_map(field_names) {
            index_map
        } else {
            return Err(TableInsertError::SchemaMismatch);
        };
        let mut temp_buffer = Vec::with_capacity(self.schema.fields().len());
        for i in 0..rows.len() {
            let (l, r) = rows.split_at_mut(i + 1);
            let row = &mut l[i];
            self.validate_row(&index_map, &mut temp_buffer, r, row)?;
        }
        if let Some(ai_idx) = self.schema.autoincrement_field() {
            let batch_size = rows.len() as u64;
            let start_id = self.counter.fetch_add(batch_size, Ordering::SeqCst);

            let mut table_rows = self.rows_mut();
            for (i, mut row) in rows.into_iter().enumerate() {
                let current_id = start_id + i as u64;

                row.data_mut()[ai_idx] = DataValue::Scalar(ScalarValue::Int(current_id as i32));
                table_rows.push(row);
            }
        } else {
            for row in rows.into_iter() {
                self.rows_mut().push(row);
            }
        }
        Ok(())
    }
    /// Validates row against table and other rows in one insert request
    ///
    /// # Errors:
    /// - returns `SchemaMismatch` if row fields do not match with the schema in the table.
    ///
    /// - returns `UniqueConstraint` if some unique field value is already present in the table or in the other rows.
    pub fn validate_row(
        &self,
        index_map: &[Option<usize>],
        temp_buffer: &mut Vec<DataValue>,
        other_rows: &[Row],
        row: &mut Row,
    ) -> Result<(), TableInsertError> {
        if !self.schema.validate(index_map, row.data()) {
            return Err(TableInsertError::SchemaMismatch);
        }
        self.schema
            .order_row(index_map, row.data_mut(), temp_buffer);
        let table_rows = self.rows();
        for (id, field) in self.schema.fields().iter().enumerate() {
            // It already guaranteed to be unique
            if field.1.auto_increment() {
                continue;
            }
            if field.1.is_unique() {
                for cmp_row in table_rows.iter().chain(other_rows.iter()) {
                    let cmp_value = cmp_row.data().get(id);
                    let row_value = row.data().get(id);
                    if row_value == cmp_value {
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

    pub fn counter(&self) -> &AtomicU64 {
        &self.counter
    }
}
impl Clone for Table {
    fn clone(&self) -> Self {
        Self {
            rows: RwLock::new(self.rows().clone()),
            schema: self.schema.clone(),
            counter: AtomicU64::new(self.counter.load(Ordering::SeqCst)),
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
    use std::ops::Deref;

    use crate::{
        self as storage,
        common_types::{DataValue, ScalarType, ScalarValue},
        row::Row,
        schema::{FieldModifier, FieldType, Schema},
    };
    use storage::table::{Table, TableInsertError};
    use structures::VecMap;

    #[test]
    fn insert_unique_constraint_error() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = vec![DataValue::Scalar(ScalarValue::Int(0))];
        table
            .insert_row(&vec!["id".to_owned()], Row::new(value.clone()))
            .unwrap();
        let res = table.insert_row(&vec!["id".to_owned()], Row::new(value));
        assert_eq!(res, Err(TableInsertError::UniqueConstraint))
    }

    #[test]
    fn insert_multiple_unique_constraint_error() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = vec![DataValue::Scalar(ScalarValue::Int(0))];
        let value1 = vec![DataValue::Scalar(ScalarValue::Int(0))];
        let rows = vec![Row::new(value), Row::new(value1)];
        let res = table.insert_rows(&vec!["id".to_owned()], rows);
        assert_eq!(res, Err(TableInsertError::UniqueConstraint))
    }

    #[test]
    fn insert_multiple_unique_single_row_success() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = vec![DataValue::Scalar(ScalarValue::Int(0))];
        let rows = vec![Row::new(value)];
        let res = table.insert_rows(&vec!["id".to_owned()], rows);
        assert_eq!(res, Ok(()))
    }

    #[test]
    fn insert_multiple_unique_success() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = DataValue::Scalar(ScalarValue::Int(0));
        let value1 = DataValue::Scalar(ScalarValue::Int(1));
        let rows = vec![Row::new(vec![value]), Row::new(vec![value1])];
        let res = table.insert_rows(&vec!["id".to_owned()], rows);
        assert_eq!(res, Ok(()))
    }

    #[test]
    fn insert_single_unique_multiple_success() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = DataValue::Scalar(ScalarValue::Int(0));
        let value1 = DataValue::Scalar(ScalarValue::Int(1));
        let field_names = vec!["id".to_owned()];
        let res = table.insert_row(&field_names, Row::new(vec![value]));
        let res1 = table.insert_row(&field_names, Row::new(vec![value1]));
        assert_eq!(res, Ok(()));
        assert_eq!(res1, Ok(()));
    }

    #[test]
    fn insert_field_mismatch_error() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::NotNull]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        let value = vec![];
        let res = table.insert_row(&vec!["id".to_owned()], Row::new(value));
        assert_eq!(res, Err(TableInsertError::SchemaMismatch))
    }
    #[test]
    fn autoincrementing() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::AutoIncrement]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);

        table.insert_row(&[], Row::new(vec![])).unwrap();
        table.insert_row(&[], Row::new(vec![])).unwrap();
        table.insert_row(&[], Row::new(vec![])).unwrap();

        assert_eq!(
            table.rows().deref(),
            &vec![
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(0))]),
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(1))]),
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(2))]),
            ]
        )
    }

    #[test]
    fn autoincrementing_batch() {
        let field_type = FieldType::new(ScalarType::Int, vec![FieldModifier::AutoIncrement]);
        let schema = Schema::new(VecMap::from([("id".to_owned(), field_type)])).unwrap();
        let table = Table::new(schema);
        table
            .insert_rows(
                &[],
                vec![Row::new(vec![]), Row::new(vec![]), Row::new(vec![])],
            )
            .unwrap();

        assert_eq!(
            table.rows().deref(),
            &vec![
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(0))]),
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(1))]),
                Row::new(vec![DataValue::Scalar(ScalarValue::Int(2))]),
            ]
        )
    }
}
