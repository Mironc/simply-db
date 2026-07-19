use std::panic;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use query::queries::insert::InsertQuery;
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
fn init_db_unique() -> Database {
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

fn insert_by_one(db: &mut Database, records: &[Record]) {
    for record in records {
        let insert_query = InsertQuery::new("users".to_owned(), vec![record.into_schema_value()]);
        insert_query.execute(db).unwrap();
    }
}
fn insert_batch(db: &mut Database, records: &[Record]) {
    let insert_query = InsertQuery::new(
        "users".to_owned(),
        records.iter().map(|x| x.into_schema_value()).collect(),
    );
    insert_query.execute(db).unwrap();
}
fn criterion_benchmark(c: &mut Criterion) {
    let records = read_records();
    let batch_size = records.len() as u64;
    if batch_size == 0 {
        panic!("Expected dataset with more than 0 elements or file was corrupted");
    }

    let mut group = c.benchmark_group("insert");
    group.throughput(Throughput::Elements(batch_size));
    group.bench_function("single", |b| {
        b.iter_batched_ref(
            || init_db(),
            |db| insert_by_one(db, &records),
            BatchSize::PerIteration,
        );
    });

    group.bench_function("batch", |b| {
        b.iter_batched_ref(
            || init_db(),
            |db| insert_batch(db, &records),
            BatchSize::PerIteration,
        );
    });
    group.finish();

    let mut group = c.benchmark_group("insert_unique");
    group.throughput(Throughput::Elements(batch_size));
    group.bench_function("single", |b| {
        b.iter_batched_ref(
            || init_db_unique(),
            |db| insert_by_one(db, &records),
            BatchSize::PerIteration,
        );
    });

    group.bench_function("batch", |b| {
        b.iter_batched_ref(
            || init_db_unique(),
            |db| insert_batch(db, &records),
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
