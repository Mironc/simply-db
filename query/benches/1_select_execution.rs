mod formatter;
mod setup;
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use query::{
    expr::{ComparisonOp, Expr, LiteralValue},
    queries::select::{Projection, SelectError, SelectQueryBuilder},
};
use storage::{common_types::DataValue, db::Database};

use crate::{
    formatter::WallTimeQps,
    setup::{init_db, insert_records},
};

fn select_rows(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQueryBuilder::new("users".to_owned(), Projection::Row).build();
    query.execute(db)
}
fn select_projection(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQueryBuilder::new(
        "users".to_owned(),
        Projection::Expr(vec![Expr::Field("id".to_owned())]),
    )
    .build();
    query.execute(db)
}
fn select_where(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQueryBuilder::new("users".to_owned(), Projection::Row)
        .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("email".to_owned()),
            Expr::Literal(LiteralValue::Text("fconsidinerr@uiuc.edu".to_owned())),
        ))))
        .build();
    query.execute(db)
}
fn select_where_projection(db: &mut Database) -> Result<Vec<Vec<DataValue>>, SelectError> {
    let query = SelectQueryBuilder::new(
        "users".to_owned(),
        Projection::Expr(vec![Expr::Field("id".to_owned())]),
    )
    .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
        Expr::Field("email".to_owned()),
        Expr::Literal(LiteralValue::Text("fconsidinerr@uiuc.edu".to_owned())),
    ))))
    .build();
    query.execute(db)
}
fn criterion_benchmark(c: &mut Criterion<WallTimeQps>) {
    for record_spawn in [|| setup::load_records_1k(), || setup::load_records_12_5k()] {
        let records = record_spawn();
        let db = init_db();
        let batch_size = records.len() as u64;
        insert_records(&db, &records);
        let mut group = c.benchmark_group(format!("select_execution_{}", batch_size));
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
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_measurement(WallTimeQps);
    targets = criterion_benchmark
}
criterion_main!(benches);
