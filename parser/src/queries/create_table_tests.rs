use query::{Query, queries::create_table::CreateTable};
use storage::{
    common_types::{FieldType, Schema},
    scalar_type,
};

use crate::{
    common::{ParseError, TokenWalker},
    queries::query::parse_create_query,
    tokenizer::tokenize,
};

#[test]
fn create_table_success() {
    let tokens =
        tokenize("CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name TEXT NOT NULL)");

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    let mut row_type = Vec::new();
    row_type.push(("id".to_owned(), FieldType::new(scalar_type!(Int), false)));
    row_type.push(("name".to_owned(), FieldType::new(scalar_type!(Text), false)));
    let cmp_query = CreateTable::new("users".to_owned(), Schema::new(row_type), true);

    assert_eq!(query, Ok(Query::CreateTable(cmp_query)))
}
#[test]
fn create_table_no_modifiers() {
    let tokens = tokenize("CREATE TABLE IF NOT EXISTS users (id INT, name TEXT)");

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    let mut row_type = Vec::new();
    row_type.push(("id".to_owned(), FieldType::new(scalar_type!(Int), true)));
    row_type.push(("name".to_owned(), FieldType::new(scalar_type!(Text), true)));
    let cmp_query = CreateTable::new("users".to_owned(), Schema::new(row_type), true);

    assert_eq!(query, Ok(Query::CreateTable(cmp_query)))
}
#[test]
fn create_table_unknown_modifier() {
    let tokens = tokenize("CREATE TABLE IF NOT EXISTS users (id INT baba)");

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    assert_eq!(query, Err(ParseError::UnknownModifier { modifier: "baba" }))
}
#[test]
fn create_table_unexpected_token() {
    let tokens =
        tokenize("CREATE TABLE IF NOT EXISTS%users (id INT PRIMARY KEY, name TEXT NOT NULL)");
    println!("{:?}", tokens);
    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);
    println!("{:?}", query);
    assert!(matches!(query, Err(ParseError::Other { message: _ })))
}
