use std::time::Instant;

use simply_db::MetricsCollector;

#[derive(Debug)]
pub struct ServerMetrics {}
impl ServerMetrics {
    pub fn new() -> Self {
        Self {}
    }
}
pub struct MetricsToken<'a> {
    raw_sql: &'a str,
    parse_start: Instant,
    execution_start: Instant,
}
impl MetricsCollector for ServerMetrics {
    type RequestToken<'a> = MetricsToken<'a>;
    fn on_parse_start<'a>(&self, _raw_sql: &'a str) -> MetricsToken<'a> {
        log::info!("Got query {}", _raw_sql);
        MetricsToken {
            parse_start: Instant::now(),
            execution_start: Instant::now(),
            raw_sql: _raw_sql,
        }
    }

    fn on_parse_completed<'a>(
        &self,
        token: &mut Self::RequestToken<'a>,
        res: &Result<query::QueryRequest, parser::ParseError<'a>>,
    ) {
        token.execution_start = Instant::now();
        if let Err(res) = res {
            println!(
                "Failed to parse query \"{}\" with error:{:?}",
                token.raw_sql, res
            )
        }
    }

    fn on_execute_completed<'a>(
        &self,
        token: &mut Self::RequestToken<'a>,
        res: &Vec<Result<query::QueryOutput, query::QueryError>>,
    ) {
        log::info!(
            "Query \"{}\". Total: {}s, Parse: {}s, Exec: {}s",
            token.raw_sql,
            token.parse_start.elapsed().as_secs_f32(),
            (token.execution_start - token.parse_start).as_secs_f32(),
            token.execution_start.elapsed().as_secs_f32()
        );
    }
}
