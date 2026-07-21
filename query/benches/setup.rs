use query::queries::insert::InsertQuery;
use storage::{
    common_types::{FieldModifier, FieldType, Schema, SchemaValue},
    db::Database,
    hashmap, scalar, scalar_type,
    table::Table,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Record {
    id: i32,
    name: String,
    email: String,
}
impl Record {
    pub fn into_schema_value(&self) -> SchemaValue {
        SchemaValue::new(hashmap!(
            "id".to_owned() => scalar!(Int(self.id)),
            "name".to_owned() => scalar!(Text(self.name.clone())),
            "email".to_owned() => scalar!(Text(self.email.clone()))
        ))
    }
}
pub fn read_records() -> Vec<Record> {
    let data = include_str!("MOCK_DATA.csv");
    csv::Reader::from_reader(data.as_bytes())
        .into_deserialize::<Record>()
        .filter_map(|x| if let Ok(x) = x { Some(x) } else { None })
        .collect::<Vec<Record>>()
}

pub fn init_db() -> Database {
    let db = Database::new();
    let field_id = FieldType::new(scalar_type!(Int), vec![FieldModifier::NotNull]);
    let field_name = FieldType::new(scalar_type!(Text), vec![FieldModifier::NotNull]);
    let field_email = FieldType::new(scalar_type!(Text), vec![FieldModifier::NotNull]);
    let schema = Schema::new(vec![
        ("id".to_owned(), field_id),
        ("name".to_owned(), field_name),
        ("email".to_owned(), field_email),
    ]);
    let table = Table::new(schema);
    db.insert_table("users".to_owned(), table).unwrap();
    db
}

pub fn init_db_unique() -> Database {
    let db = Database::new();
    let field_id = FieldType::new(
        scalar_type!(Int),
        vec![FieldModifier::NotNull, FieldModifier::Unique],
    );
    let field_name = FieldType::new(scalar_type!(Text), vec![FieldModifier::NotNull]);
    let field_email = FieldType::new(scalar_type!(Text), vec![FieldModifier::NotNull]);
    let schema = Schema::new(vec![
        ("id".to_owned(), field_id),
        ("name".to_owned(), field_name),
        ("email".to_owned(), field_email),
    ]);
    let table = Table::new(schema);
    db.insert_table("users".to_owned(), table).unwrap();
    db
}
pub fn insert_records(db: &Database, records: &[Record]) {
    let insert_query = InsertQuery::new(
        "users".to_owned(),
        records.iter().map(|x| x.into_schema_value()).collect(),
    );
    insert_query.execute(db).unwrap();
}
