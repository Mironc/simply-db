mod formatter;
mod setup;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use parser::parse_query_request;
use storage::db::Database;

use crate::{formatter::WallTimeQps, setup::init_db};

fn select(db: &Database) {
    let q_req =
        parse_query_request("SELECT name FROM users WHERE email == 'fconsidinerr@uiuc.edu'")
            .unwrap();
    q_req.execute(db).iter().for_each(|x| {
        x.as_ref().unwrap();
    });
}
fn criterion_benchmark(c: &mut Criterion<WallTimeQps>) {
    let db = init_db();
    let records = setup::load_records();
    setup::insert_records(&db, &records);
    let mut group = c.benchmark_group("select");
    group.throughput(Throughput::Elements(1));
    group.bench_function("projection_where", |b| {
        b.iter(|| select(&db));
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(formatter::WallTimeQps);
    targets = criterion_benchmark
);
criterion_main!(benches);
