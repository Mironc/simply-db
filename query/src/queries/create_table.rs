use storage::{
    db::Database,
    schema::{FieldType, Schema, SchemaError},
    table::Table,
};
use structures::VecMap;

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum CreateTableError {
    AlreadyExists,
    SchemaError(SchemaError),
}

impl From<SchemaError> for CreateTableError {
    fn from(v: SchemaError) -> Self {
        Self::SchemaError(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    name: String,
    fields: VecMap<String, FieldType>,
    if_not_exists: bool,
}
impl CreateTable {
    pub fn new(name: String, fields: VecMap<String, FieldType>, if_not_exists: bool) -> Self {
        Self {
            name,
            fields,
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
        let schema = Schema::new(self.fields.clone())?;
        let table = Table::new(schema);
        db.insert_table(self.name.clone(), table).unwrap();
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use storage::{common_types::ScalarType, db::Database, schema::FieldType};
    use structures::VecMap;

    use crate::queries::create_table::CreateTable;

    #[test]
    fn success() {
        let mut fields = VecMap::new();
        fields.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        fields.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        let mut db = Database::new();
        let create_table = CreateTable::new("table1".to_string(), fields, false);
        assert!(create_table.execute(&mut db).is_ok());
    }

    #[test]
    fn already_exists() {
        let mut fields = VecMap::new();
        fields.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        fields.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        let mut db = Database::new();
        let create_table = CreateTable::new("table1".to_string(), fields.clone(), false);
        assert!(create_table.execute(&mut db).is_ok());

        let create_table = CreateTable::new("table1".to_string(), fields.clone(), false);
        assert!(create_table.execute(&mut db).is_err());

        let create_table = CreateTable::new("table1".to_string(), fields, true);
        assert!(create_table.execute(&mut db).is_ok());
    }
}
