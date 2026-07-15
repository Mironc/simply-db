use storage::{common_types::SchemaValue, db::Database, row::Row, table::TableInsertError};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertError {
    TableInsertError(TableInsertError),
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
            table
                .insert_rows(
                    self.rows_data
                        .iter()
                        .cloned()
                        .map(|x| Row::new(x))
                        .collect(),
                )
                .map_err(|x| InsertError::TableInsertError(x))
        } else {
            Err(InsertError::UnknownTable(self.table.clone()))
        }
    }
}
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use storage::{
        common_types::{
            DataType, DataValue, FieldModifier, FieldType, ScalarType, ScalarValue, Schema,
            SchemaValue,
        },
        db::Database,
        table::TableInsertError,
    };

    use crate::queries::{
        create_table::CreateTable,
        insert::{InsertError, InsertQuery},
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
        assert_eq!(
            insert.execute(&mut db),
            Err(InsertError::TableInsertError(
                TableInsertError::SchemaMismatch
            ))
        );
    }
    #[test]
    fn unique_constraint() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), DataValue::Scalar(ScalarValue::Int(30)));
        let schema_value = SchemaValue::new(data);

        let mut field_types = Vec::new();
        field_types.push((
            "id".to_string(),
            FieldType::new(
                DataType::Scalar(ScalarType::Int),
                vec![FieldModifier::Unique],
            ),
        ));
        let schema = Schema::new(field_types);
        let mut db = Database::new();

        // Creating table with empty type
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new(
            "table".to_string(),
            vec![schema_value.clone(), schema_value],
        );
        assert_eq!(
            insert.execute(&mut db),
            Err(InsertError::TableInsertError(
                TableInsertError::UniqueConstraint
            ))
        );
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
            FieldType::new(DataType::Scalar(ScalarType::Int), vec![]),
        ));
        field_types.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));
        let schema = Schema::new(field_types);

        let mut db = Database::new();
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new("table".to_string(), vec![schema_value]);
        assert!(insert.execute(&mut db).is_ok());
    }
}
