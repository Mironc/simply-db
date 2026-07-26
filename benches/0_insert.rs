mod formatter;
mod setup;
use std::str::FromStr;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use parser::parse_query_request;
use storage::db::Database;

use crate::{
    formatter::WallTimeQps,
    setup::{Record, init_db, init_db_unique},
};
fn insert_single(db: &Database, records: &[Record]) {
    for record in records {
        let q_req = parse_query_request(&format!(
            "INSERT INTO users (id, name, email) VALUES ({},'{}','{}')",
            record.id, record.name, record.email
        ))
        .unwrap();
        q_req.execute(db).iter().for_each(|x| {
            x.as_ref().unwrap();
        });
    }
}
fn insert_batch(db: &Database, records: &[Record]) {
    let mut values = String::from_str("INSERT INTO users (id, name, email) VALUES").unwrap();

    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            values.push_str(", ");
        }
        let _ = std::fmt::write(
            &mut values,
            format_args!("({},'{}','{}')", record.id, record.name, record.email),
        );
    }

    let q_req = parse_query_request(&values).unwrap();

    q_req.execute(db).iter().for_each(|x| {
        x.as_ref().unwrap();
    });
}
fn criterion_benchmark(c: &mut Criterion<WallTimeQps>) {
    let records = setup::load_records();
    let mut group = c.benchmark_group("insert");
    // Makes 10000 unique queries
    group.throughput(Throughput::Elements(10000));
    group.bench_function("single", |b| {
        b.iter_batched_ref(
            || init_db(),
            |db| insert_single(db, &records),
            BatchSize::PerIteration,
        );
    });
    // Each query inserts 10000 objects, but it still one query
    group.throughput(Throughput::Elements(1));
    group.bench_function("batch", |b| {
        b.iter_batched_ref(
            || init_db(),
            |db| insert_batch(db, &records),
            BatchSize::PerIteration,
        );
    });
    group.finish();

    let mut group = c.benchmark_group("insert_unique");
    // Makes 10000 unique queries
    group.throughput(Throughput::Elements(10000));
    group.bench_function("single", |b| {
        b.iter_batched_ref(
            || init_db_unique(),
            |db| insert_single(db, &records),
            BatchSize::PerIteration,
        );
    });
    // Each query inserts 10000 objects, but it still one query
    group.throughput(Throughput::Elements(1));
    group.bench_function("batch", |b| {
        b.iter_batched_ref(
            || init_db_unique(),
            |db| insert_batch(db, &records),
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(formatter::WallTimeQps);
    targets = criterion_benchmark
);
criterion_main!(benches);
