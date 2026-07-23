use storage::{db::Database, schema::Schema, table::Table};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum CreateTableError {
    AlreadyExists,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    name: String,
    schema: Schema,
    if_not_exists: bool,
}
impl CreateTable {
    pub fn new(name: String, schema: Schema, if_not_exists: bool) -> Self {
        Self {
            name,
            schema,
            if_not_exists,
        }
    }

    pub fn execute(&self, db: &Database) -> Result<(), CreateTableError> {
        if db.has_table(&self.name) {
            if self.if_not_exists {
                return Ok(());
            }
            return Err(CreateTableError::AlreadyExists);
        }
        let table = Table::new(self.schema.clone());
        _ = db.insert_table(self.name.clone(), table).unwrap();
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use storage::{
        common_types::ScalarType,
        db::Database,
        schema::{FieldType, Schema},
    };
    use structures::VecMap;

    use crate::queries::create_table::CreateTable;

    #[test]
    fn success() {
        let mut fields = VecMap::new();
        fields.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        fields.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        let row_type = Schema::new(fields);
        let mut db = Database::new();
        let create_table = CreateTable::new("table1".to_string(), row_type, false);
        assert!(create_table.execute(&mut db).is_ok());
    }

    #[test]
    fn already_exists() {
        let mut fields = VecMap::new();
        fields.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        fields.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        let row_type = Schema::new(fields);
        let mut db = Database::new();
        let create_table = CreateTable::new("table1".to_string(), row_type.clone(), false);
        assert!(create_table.execute(&mut db).is_ok());

        let create_table = CreateTable::new("table1".to_string(), row_type.clone(), false);
        assert!(create_table.execute(&mut db).is_err());

        let create_table = CreateTable::new("table1".to_string(), row_type, true);
        assert!(create_table.execute(&mut db).is_ok());
    }
}
