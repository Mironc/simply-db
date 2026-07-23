use storage::{db::Database, row::Row, table::TableInsertError};

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
    field_names: Vec<String>,
    rows_data: Vec<Row>,
}
impl InsertQuery {
    pub fn new(table: String, field_names: Vec<String>, rows_data: Vec<Row>) -> Self {
        Self {
            table,
            field_names,
            rows_data,
        }
    }

    pub fn execute(&self, db: &Database) -> Result<(), InsertError> {
        if let Some(table) = db.get_table(&self.table) {
            table
                .insert_rows(&self.field_names, self.rows_data.clone())
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
        common_types::{DataValue, ScalarType, ScalarValue},
        db::Database,
        row::Row,
        scalar,
        schema::{FieldModifier, FieldType, Schema},
        table::TableInsertError,
    };
    use structures::VecMap;

    use crate::queries::{
        create_table::CreateTable,
        insert::{InsertError, InsertQuery},
    };

    #[test]
    fn unknown_table() {
        // Empty value
        let type_value = Row::new(vec![]);
        let mut db = Database::new();

        let insert = InsertQuery::new("nonexistent_table".to_string(), vec![], vec![type_value]);
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
        let type_value = Row::new(vec![scalar!(Int(30)), scalar!(Text("Alice".to_owned()))]);
        let mut db = Database::new();

        let field_types = VecMap::new();
        let schema = Schema::new(field_types);
        // Creating table with empty type
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new(
            "table".to_string(),
            vec!["age".to_owned(), "name".to_owned()],
            vec![type_value],
        );
        assert_eq!(
            insert.execute(&mut db),
            Err(InsertError::TableInsertError(
                TableInsertError::SchemaMismatch
            ))
        );
    }
    #[test]
    fn unique_constraint() {
        let data = vec![scalar!(Int(30))];
        let row = Row::new(data);

        let mut field_types = VecMap::new();
        field_types.insert(
            "id".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::Unique]),
        );
        let schema = Schema::new(field_types);
        let mut db = Database::new();

        // Creating table with empty type
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new(
            "table".to_string(),
            vec!["id".to_owned()],
            vec![row.clone(), row],
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
        let data = vec![scalar!(Int(30)), scalar!(Text("Alice".to_owned()))];
        let row = Row::new(data);

        let mut field_types = VecMap::new();
        field_types.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        field_types.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        let schema = Schema::new(field_types);

        let mut db = Database::new();
        let create_table = CreateTable::new("table".to_string(), schema, false);
        create_table.execute(&mut db).unwrap();

        let insert = InsertQuery::new(
            "table".to_string(),
            vec!["age".to_owned(), "name".to_owned()],
            vec![row],
        );
        assert!(insert.execute(&mut db).is_ok());
    }
}
