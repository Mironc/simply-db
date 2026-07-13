use serde::{Deserialize, Serialize};

/// Input data for `/v1/query` route.
///
/// Contains single or multiple SQL queries at once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQueryRequest {
    sql: String,
}
impl SqlQueryRequest {
    pub fn new(sql: String) -> Self {
        Self { sql }
    }

    pub fn sql(&self) -> &str {
        &self.sql
    }
}
