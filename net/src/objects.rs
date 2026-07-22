use std::collections::HashMap;

use parser::{ExpectExprErr, ParseError};
use query::{QueryError, QueryOutput};
use serde::{Deserialize, Serialize};
use storage::common_types::Schema;

/// Output of `/v1/overview` route.
///
/// Contains data about schemas inside database tables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    schemas: HashMap<String, Schema>,
}
impl Overview {
    pub fn schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }
}
impl Overview {
    pub fn new(schemas: HashMap<String, Schema>) -> Self {
        Self { schemas }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseState {
    Healthy,
    Degraded,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    current_heap_size: usize,
}
impl MemoryMetrics {
    pub fn current_heap_size(&self) -> usize {
        self.current_heap_size
    }
}
impl MemoryMetrics {
    pub fn new(current_heap_size: usize) -> Self {
        Self { current_heap_size }
    }
}
/// Output of `/health` route.
///
/// Contains general information about database state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    /// Current database state
    state: DatabaseState,
    /// Current utc time in rfc3339 format
    time: String,
    memory_metrics: MemoryMetrics,
}
impl Health {
    pub fn state(&self) -> DatabaseState {
        self.state
    }

    pub fn memory_metrics(&self) -> MemoryMetrics {
        self.memory_metrics
    }
}

impl Health {
    pub fn new(state: DatabaseState, time: String, memory_metrics: MemoryMetrics) -> Self {
        Self {
            state,
            time,
            memory_metrics,
        }
    }
}
/// Output of `/v1/query` route.
///
/// Contains execution results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryOutput {
    output: Vec<Result<QueryOutput, QueryError>>,
}

impl SqlQueryOutput {
    pub fn new(output: Vec<Result<QueryOutput, QueryError>>) -> Self {
        Self { output }
    }

    pub fn output(&self) -> &[Result<QueryOutput, QueryError>] {
        &self.output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "payload")]
pub enum ParseErrorDTO {
    UnknownInstruction,
    UnclosedBracket(char),
    UnappropriateKeyword,
    ExpectedExpr(ExpectExprErrDTO),
    FieldNumberMismatch {
        expected: usize,
        provided: usize,
    },
    UnknownModifier {
        modifier: String,
    },
    UnexpectedSymbol {
        expected: String,
        given: String,
    },
    /// Unexpected end of file
    UnexpectedEof,
    /// Unexpected start of file
    UnexpectedSof,
    UnknownDataType,
    UnknownPattern,
    WrongPattern,
    Other {
        message: String,
    },
    IdentStartsWithNumber,
}
#[cfg(feature = "server")]
impl axum::response::IntoResponse for ParseErrorDTO {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::BAD_REQUEST, axum::Json(self)).into_response()
    }
}
impl From<ParseError<'_>> for ParseErrorDTO {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::UnknownInstruction => ParseErrorDTO::UnknownInstruction,
            ParseError::UnclosedBracket(c) => ParseErrorDTO::UnclosedBracket(c),
            ParseError::UnappropriateKeyword => ParseErrorDTO::UnappropriateKeyword,
            ParseError::ExpectedExpr(expect_expr_err) => {
                ParseErrorDTO::ExpectedExpr(expect_expr_err.into())
            }
            ParseError::FieldNumberMismatch { expected, provided } => {
                ParseErrorDTO::FieldNumberMismatch { expected, provided }
            }
            ParseError::UnknownModifier { modifier } => ParseErrorDTO::UnknownModifier {
                modifier: modifier.to_string(),
            },
            ParseError::UnexpectedSymbol { expected, given } => ParseErrorDTO::UnexpectedSymbol {
                expected: expected.to_string(),
                given: given.to_string(),
            },
            ParseError::UnexpectedEof => ParseErrorDTO::UnexpectedEof,
            ParseError::UnexpectedSof => ParseErrorDTO::UnexpectedSof,
            ParseError::UnknownDataType => ParseErrorDTO::UnknownDataType,
            ParseError::UnknownPattern => ParseErrorDTO::UnknownPattern,
            ParseError::WrongPattern => ParseErrorDTO::WrongPattern,
            ParseError::Other { message } => ParseErrorDTO::Other {
                message: message.to_string(),
            },
            ParseError::IdentStartsWithNumber => ParseErrorDTO::IdentStartsWithNumber,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "payload")]
pub enum ExpectExprErrDTO {
    Before { symbol: String },
    After { symbol: String },
    BeforeAfter { symbol: String },
}

impl<'a> From<ExpectExprErr<'a>> for ExpectExprErrDTO {
    fn from(value: ExpectExprErr) -> Self {
        match value {
            ExpectExprErr::Before { symbol } => ExpectExprErrDTO::Before {
                symbol: symbol.to_string(),
            },
            ExpectExprErr::After { symbol } => ExpectExprErrDTO::After {
                symbol: symbol.to_string(),
            },
            ExpectExprErr::BeforeAfter { symbol } => ExpectExprErrDTO::BeforeAfter {
                symbol: symbol.to_string(),
            },
        }
    }
}
