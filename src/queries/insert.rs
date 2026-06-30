use crate::{common_types::SchemaValue, db::Database, row::Row, table::TableError};
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum InsertError {
    /// Type in table does not matches with type of table
    TypeMismatch,
    /// Database doesn't have requested table
    UnknownTable(String),
}
#[derive(Debug, Clone, PartialEq)]
pub struct InsertQuery {
    table: String,
    rows_data: Vec<SchemaValue>,
}
impl InsertQuery {
    pub fn new(table: String, rows_data: Vec<SchemaValue>) -> Self {
        Self { table, rows_data }
    }
    pub fn execute(&self, db: &Database) -> Result<(), InsertError> {
        if let Some(table) = db.get_table(&self.table) {
            for row_data in self.rows_data.iter() {
                table
                    .insert_row(Row::new(row_data.clone()))
                    .map_err(|x| match x {
                        TableError::SchemaMismatch => InsertError::TypeMismatch,
                    })?;
            }
            Ok(())
        } else {
            Err(InsertError::UnknownTable(self.table.clone()))
        }
    }
}
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        common_types::{
            DataType, DataValue, FieldType, ScalarType, ScalarValue, Schema, SchemaValue,
        },
        db::Database,
        queries::{create_table::CreateTable, insert::InsertQuery},
    };

    #[test]
    fn unknown_table() {
        // Empty value
        let type_value = SchemaValue::new(HashMap::new());
        let mut db = Database::new();

        let insert = InsertQuery::new("nonexistent_table".to_string(), vec![type_value]);
        assert!(insert.execute(&mut db).is_err());
    }

    #[test]
    fn type_mismatch() {
        let mut data = HashMap::new();
        data.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));
        data.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("Alice".to_string())),
        );
        let type_value = SchemaValue::new(data);
        let mut db = Database::new();

        let field_types = Vec::new();
        let schema = Schema::new(field_types);
        // Creating table with empty type
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new("table".to_string(), vec![type_value]);
        assert!(insert.execute(&mut db).is_err());
    }

    #[test]
    fn success() {
        let mut data = HashMap::new();
        data.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));
        data.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("Alice".to_string())),
        );
        let schema_value = SchemaValue::new(data);

        let mut field_types = Vec::new();
        field_types.push((
            "age".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Int), false),
        ));
        field_types.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), false),
        ));
        let schema = Schema::new(field_types);

        let mut db = Database::new();
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new("table".to_string(), vec![schema_value]);
        assert!(insert.execute(&mut db).is_ok());
    }
}
