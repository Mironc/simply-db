mod formatter;
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use query::{
    expr::{ComparisonOp, Expr, LiteralValue},
    queries::{
        insert::InsertQuery,
        select::{Projection, SelectError, SelectQuery},
    },
};
use storage::{
    common_types::{DataValue, FieldModifier, FieldType, Schema, SchemaValue},
    db::Database,
    hashmap, scalar, scalar_type,
    table::Table,
};

use crate::formatter::WallTimeQps;
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
fn select_rows(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQuery::new("users".to_owned(), Projection::Row, None);
    query.execute(db)
}
fn select_projection(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQuery::new(
        "users".to_owned(),
        Projection::Expr(vec![Expr::Field("id".to_owned())]),
        None,
    );
    query.execute(db)
}
fn select_where(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQuery::new(
        "users".to_owned(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("email".to_owned()),
            Expr::Literal(LiteralValue::Text("fconsidinerr@uiuc.edu".to_owned())),
        )))),
    );
    query.execute(db)
}
fn select_where_projection(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQuery::new(
        "users".to_owned(),
        Projection::Expr(vec![Expr::Field("id".to_owned())]),
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("email".to_owned()),
            Expr::Literal(LiteralValue::Text("fconsidinerr@uiuc.edu".to_owned())),
        )))),
    );
    query.execute(db)
}
fn criterion_benchmark(c: &mut Criterion<WallTimeQps>) {
    let records = read_records();
    let db = init_db();
    insert_multiple(&db, &records);
    let mut group = c.benchmark_group("select");
    group.throughput(Throughput::Elements(1));
    group.bench_function("row", |b| {
        b.iter_batched_ref(|| db.clone(), |db| select_rows(db), BatchSize::PerIteration);
    });

    group.bench_function("projection", |b| {
        b.iter_batched_ref(
            || db.clone(),
            |db| select_projection(db),
            BatchSize::PerIteration,
        );
    });

    group.bench_function("row_where", |b| {
        b.iter_batched_ref(
            || db.clone(),
            |db| select_where(db),
            BatchSize::PerIteration,
        );
    });
    group.bench_function("projection_where", |b| {
        b.iter_batched_ref(
            || db.clone(),
            |db| select_where_projection(db),
            BatchSize::PerIteration,
        );
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_measurement(WallTimeQps);
    targets = criterion_benchmark
}
criterion_main!(benches);
