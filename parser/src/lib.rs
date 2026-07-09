use query::{Query, QueryRequest};

use crate::{common::ParseError, queries::query::parse_query, tokenizer::tokenize};

#[doc(hidden)]
pub mod common;
#[doc(hidden)]
pub mod queries;
#[doc(hidden)]
pub mod tokenizer;

pub fn parse_query_request<'a>(source: &'a str) -> Result<QueryRequest, ParseError<'a>> {
    let queries = source
        .split(';')
        .filter(|x| !x.is_empty())
        .map(|x| {
            let tokens = tokenize(x)?;
            Ok(parse_query(tokens)?)
        })
        .collect::<Result<Vec<Query>, ParseError>>()?;

    Ok(QueryRequest::new(queries))
}
