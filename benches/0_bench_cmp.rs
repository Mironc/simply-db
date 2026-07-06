use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use simply_db::sql::parser::tokenizer::{Sign, TokenValue};

fn criterion_benchmark(c: &mut Criterion) {
    let value = TokenValue::Sign(Sign::Plus);
    c.bench_function("to_string_cmp_not_str", |b| {
        b.iter(|| black_box(to_string_cmp_not_str(&value)))
    });

    c.bench_function("tokenvalue_cmp_not_str", |b| {
        b.iter(|| {
            black_box(tokenvalue_cmp_not_str(&value));
        })
    });

    c.bench_function("as_str_cmp_not_str", |b| {
        b.iter(|| {
            black_box(as_str_cmp_not_str(&value));
        })
    });

    let value = TokenValue::Keyword("WHERE");
    c.bench_function("to_string_cmp_str", |b| {
        b.iter(|| black_box(to_string_cmp_str(&value)))
    });

    c.bench_function("tokenvalue_cmp_str", |b| {
        b.iter(|| {
            black_box(tokenvalue_cmp_str(&value));
        })
    });

    c.bench_function("as_str_cmp_str", |b| {
        b.iter(|| {
            black_box(as_str_cmp_str(&value));
        })
    });
}

fn to_string_cmp_not_str(value: &TokenValue) -> bool {
    value.to_string() == "+"
}
fn to_string_cmp_str(value: &TokenValue) -> bool {
    value.to_string() == "WHERE"
}

fn as_str_cmp_not_str(value: &TokenValue) -> bool {
    value.as_str() == "+"
}
fn as_str_cmp_str(value: &TokenValue) -> bool {
    value.as_str() == "WHERE"
}

fn tokenvalue_cmp_not_str(value: &TokenValue) -> bool {
    value == &TokenValue::Sign(Sign::Plus)
}
fn tokenvalue_cmp_str(value: &TokenValue) -> bool {
    value == &TokenValue::Ident("WHERE")
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
