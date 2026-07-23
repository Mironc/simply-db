use query::queries::insert::InsertQuery;
use storage::{
    common_types::ScalarType,
    db::Database,
    row::Row,
    scalar,
    schema::{FieldModifier, FieldType, Schema},
    table::Table,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Record {
    id: i32,
    name: String,
    email: String,
}
impl Record {
    pub fn into_row(&self) -> Row {
        Row::new(vec![
            scalar!(Int(self.id)),
            scalar!(Text(self.name.clone())),
            scalar!(Text(self.email.clone())),
        ])
    }
    pub fn field_names() -> Vec<String> {
        vec!["id".to_owned(), "name".to_owned(), "email".to_owned()]
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
    let field_id = FieldType::new(ScalarType::Int, vec![FieldModifier::NotNull]);
    let field_name = FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]);
    let field_email = FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]);
    let schema = Schema::new(
        vec![
            ("id".to_owned(), field_id),
            ("name".to_owned(), field_name),
            ("email".to_owned(), field_email),
        ]
        .into(),
    );
    let table = Table::new(schema);
    db.insert_table("users".to_owned(), table).unwrap();
    db
}

pub fn init_db_unique() -> Database {
    let db = Database::new();
    let field_id = FieldType::new(
        ScalarType::Int,
        vec![FieldModifier::NotNull, FieldModifier::Unique],
    );
    let field_name = FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]);
    let field_email = FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]);
    let schema = Schema::new(
        vec![
            ("id".to_owned(), field_id),
            ("name".to_owned(), field_name),
            ("email".to_owned(), field_email),
        ]
        .into(),
    );
    let table = Table::new(schema);
    db.insert_table("users".to_owned(), table).unwrap();
    db
}
pub fn insert_records(db: &Database, records: &[Record]) {
    let insert_query = InsertQuery::new(
        "users".to_owned(),
        Record::field_names(),
        records.iter().map(|x| x.into_row()).collect(),
    );
    insert_query.execute(db).unwrap();
}
