use criterion::Throughput;
use criterion::measurement::{Measurement, ValueFormatter};
use std::time::Instant;

pub struct WallTimeQps;
impl Measurement for WallTimeQps {
    type Intermediate = Instant;
    type Value = u64; // nanosecs

    fn start(&self) -> Self::Intermediate {
        Instant::now()
    }

    fn end(&self, start: Self::Intermediate) -> Self::Value {
        start.elapsed().as_nanos() as u64
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        v1 + v2
    }

    fn zero(&self) -> Self::Value {
        0
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        *value as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &QpsFormatter
    }
}

struct QpsFormatter;
impl ValueFormatter for QpsFormatter {
    fn scale_throughputs(
        &self,
        _: f64,
        throughput: &Throughput,
        values: &mut [f64],
    ) -> &'static str {
        if let Throughput::Elements(elems) = throughput {
            for val in values.iter_mut() {
                *val = (*elems as f64 / *val) * 1_000_000_000.0;
            }
            "QPS"
        } else {
            "B/s"
        }
    }
    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "ns"
    }

    fn scale_values(&self, typical_value: f64, values: &mut [f64]) -> &'static str {
        if typical_value < 1_000.0 {
            "ns"
        } else if typical_value < 1_000_000.0 {
            for val in values.iter_mut() {
                *val /= 1_000.0;
            }
            "µs"
        } else if typical_value < 1_000_000_000.0 {
            for val in values.iter_mut() {
                *val /= 1_000_000.0;
            }
            "ms"
        } else {
            for val in values.iter_mut() {
                *val /= 1_000_000_000.0;
            }
            "s"
        }
    }
}
