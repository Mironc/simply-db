mod setup;
use std::panic;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use query::queries::insert::InsertQuery;
use storage::db::Database;

use crate::setup::{Record, init_db, init_db_unique, read_records};

fn insert_by_one(db: &mut Database, records: &[Record]) {
    for record in records {
        let insert_query = InsertQuery::new(
            "users".to_owned(),
            Record::field_names(),
            vec![record.into_row()],
        );
        insert_query.execute(db).unwrap();
    }
}
fn insert_batch(db: &mut Database, records: &[Record]) {
    let insert_query = InsertQuery::new(
        "users".to_owned(),
        Record::field_names(),
        records.iter().map(|x| x.into_row()).collect(),
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
