use parser::ParseError;
use query::{QueryError, QueryOutput, QueryRequest};
use storage::db::Database;

pub trait MetricsCollector {
    /// For data needed between different stages of execution
    type RequestToken<'a>;

    fn on_parse_start<'a>(&self, raw_sql: &'a str) -> Self::RequestToken<'a>;
    fn on_parse_completed<'a>(
        &self,
        token: &mut Self::RequestToken<'a>,
        res: &Result<QueryRequest, ParseError<'a>>,
    ) {
    }
    fn on_execute_completed<'a>(
        &self,
        token: &mut Self::RequestToken<'a>,
        res: &Vec<Result<QueryOutput, QueryError>>,
    ) {
    }
}
impl MetricsCollector for () {
    type RequestToken<'a> = ();

    fn on_parse_start<'a>(&self, _raw_sql: &'a str) -> Self::RequestToken<'a> {
        ()
    }
}

#[derive(Debug)]
pub struct DatabaseContext<M = ()>
where
    M: MetricsCollector,
{
    db: Database,
    metrics: M,
}
impl DatabaseContext<()> {
    pub fn new(db: Database) -> Self {
        Self { db, metrics: () }
    }
}
impl<M: MetricsCollector> DatabaseContext<M> {
    pub fn new_with_metrics(db: Database, metrics: M) -> Self {
        Self { db, metrics }
    }
    pub fn execute<'a>(
        &self,
        query: &'a str,
    ) -> Result<Vec<Result<QueryOutput, QueryError>>, ParseError<'a>> {
        let owned = query.trim_matches('\"');
        let mut token = self.metrics.on_parse_start(owned);
        let res_query_req = parser::parse_query_request(&owned);
        self.metrics.on_parse_completed(&mut token, &res_query_req);
        let res = res_query_req?.execute(&self.db);
        self.metrics.on_execute_completed(&mut token, &res);
        Ok(res)
    }

    pub fn database(&self) -> &Database {
        &self.db
    }

    pub fn metrics(&self) -> &M {
        &self.metrics
    }
}
