use query::{Query, QueryRequest};

use crate::{lexer::Lexer, queries::query::parse_query};
pub use common::{ExpectExprErr, ParseError};

#[doc(hidden)]
pub mod common;
#[doc(hidden)]
pub mod lexer;
#[doc(hidden)]
pub mod queries;

pub fn parse_query_request<'a>(source: &'a str) -> Result<QueryRequest, ParseError<'a>> {
    let queries = source
        .split(';')
        .filter(|x| !x.is_empty())
        .map(|x| {
            let tokens = Lexer::new(x);
            Ok(parse_query(tokens)?)
        })
        .collect::<Result<Vec<Query>, ParseError>>()?;

    Ok(QueryRequest::new(queries))
}
