use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use parser::queries::query::parse_query;
use parser::tokenizer::tokenize;
use std::hint::black_box;

// Helper macro to query_parse a string cleanly in tests
macro_rules! query_parse {
    ($input:expr) => {{
        let tokens = tokenize($input).unwrap();
        parse_query(tokens)
    }};
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query parsing");

    let query = "SELECT * FROM sometablename WHERE id == 0 AND name == 'Steve'";
    group.throughput(Throughput::Bytes(query.len() as u64));
    group.bench_function("select", |b| {
        b.iter(|| {
            black_box(query_parse!(query)).unwrap();
        })
    });

    let query = "INSERT INTO sometablename (id, name) VALUES (0,'Steve')";
    group.throughput(Throughput::Bytes(query.len() as u64));
    group.bench_function("insert", |b| {
        b.iter(|| {
            black_box(query_parse!(query)).unwrap();
        })
    });

    let query = "CREATE TABLE IF NOT EXISTS sometablename (id INT PRIMARY KEY, name TEXT NOT NULL)";
    group.throughput(Throughput::Bytes(query.len() as u64));
    group.bench_function("create table", |b| {
        b.iter(|| {
            black_box(query_parse!(query)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
