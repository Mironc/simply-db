use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use query::{
    expr::{Expr, LiteralValue},
    queries::{
        insert::InsertQuery,
        update::{UpdateError, UpdateQuery},
    },
};
use storage::{
    common_types::{FieldModifier, FieldType, Schema, SchemaValue},
    db::Database,
    hashmap, scalar, scalar_type,
    table::Table,
};
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Record {
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
fn read_records() -> Vec<Record> {
    let data = include_str!("MOCK_DATA.csv");
    csv::Reader::from_reader(data.as_bytes())
        .into_deserialize::<Record>()
        .filter_map(|x| if let Ok(x) = x { Some(x) } else { None })
        .collect::<Vec<Record>>()
}
fn init_db() -> Database {
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
fn insert_multiple(db: &Database, records: &[Record]) {
    let insert_query = InsertQuery::new(
        "users".to_owned(),
        records.iter().map(|x| x.into_schema_value()).collect(),
    );
    insert_query.execute(db).unwrap();
}
fn update_rows(db: &mut Database) -> Result<(), UpdateError> {
    let query = UpdateQuery::new(
        "users".to_owned(),
        vec![(
            "name".to_owned(),
            Expr::Literal(LiteralValue::Text("Bob".to_owned())),
        )],
        None,
    );
    query.execute(db)
}
fn criterion_benchmark(c: &mut Criterion) {
    let records = read_records();
    let db = init_db();
    insert_multiple(&db, &records);

    c.bench_function("update", |b| {
        b.iter_batched_ref(|| db.clone(), |db| update_rows(db), BatchSize::LargeInput);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
