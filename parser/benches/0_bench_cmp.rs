//! # Token Comparison Performance Benchmarks
//!
//! ## TL;DR
//! - Don't use `.to_string()` or `.as_str()` for comparison; prefer direct enum comparison.
//! - Use `.to_string()` and `.as_str()` only when absolutely necessary.
//!
//! ### Purpose
//! This benchmark quantifies the performance impact of different string and enum
//! comparison strategies for `TokenValue`. It serves as a guardrail against
//! accidental performance regressions in hot parsing/lexing loops.
//!
//! ### Why This Matters
//! 1. `to_string()` triggers dynamic heap allocations (`String`), which are slow.
//! 2. `as_str()` borrows a reference (`&str`), avoiding allocation.
//! 3. Direct enum comparison (`TokenValue == TokenValue`) is a simple enum variant match.
//!
//! ### Expected Results
//! * `tokenvalue_cmp_*` -> Fastest (picoseconds)
//! * `as_str_cmp_*`     -> Fast (picoseconds, but slightly worse than tokenvalue_cmp)
//! * `to_string_cmp_*`  -> Slowest (nanoseconds)

use criterion::{Criterion, criterion_group, criterion_main};
use parser::lexer::{Sign, TokenValue};
use std::hint::black_box;

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

    let value = TokenValue::Ident("Identifier");
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
    value.to_string() == "Identifier"
}

fn as_str_cmp_not_str(value: &TokenValue) -> bool {
    value.as_str() == "+"
}
fn as_str_cmp_str(value: &TokenValue) -> bool {
    value.as_str() == "Identifier"
}

fn tokenvalue_cmp_not_str(value: &TokenValue) -> bool {
    value == &TokenValue::Sign(Sign::Plus)
}
fn tokenvalue_cmp_str(value: &TokenValue) -> bool {
    value == &TokenValue::Ident("Identifier")
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
