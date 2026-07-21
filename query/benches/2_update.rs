mod formatter;
use crate::{
    formatter::WallTimeQps,
    setup::{init_db, insert_records, read_records},
};
mod setup;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use query::{
    expr::{Expr, LiteralValue},
    queries::update::{UpdateError, UpdateQuery},
};
use storage::db::Database;

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
fn criterion_benchmark(c: &mut Criterion<WallTimeQps>) {
    let records = read_records();
    let db = init_db();
    insert_records(&db, &records);
    let mut group = c.benchmark_group("update");
    group.bench_function("update", |b| {
        b.iter_batched_ref(|| db.clone(), |db| update_rows(db), BatchSize::LargeInput);
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_measurement(WallTimeQps);
    targets = criterion_benchmark
}
criterion_main!(benches);
