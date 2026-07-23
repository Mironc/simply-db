use query::{Query, queries::create_table::CreateTable};
use storage::{
    common_types::ScalarType,
    schema::{FieldModifier, FieldType, Schema},
};
use structures::VecMap;

use crate::{
    common::{ParseError, TokenWalker},
    queries::query::parse_create_query,
    tokenizer::tokenize,
};

#[test]
fn create_table_success() {
    let tokens =
        tokenize("CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    let mut row_type = VecMap::new();
    row_type.insert(
        "id".to_owned(),
        FieldType::new(ScalarType::Int, vec![FieldModifier::PrimaryKey]),
    );
    row_type.insert(
        "name".to_owned(),
        FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]),
    );
    let cmp_query = CreateTable::new("users".to_owned(), Schema::new(row_type), true);

    assert_eq!(query, Ok(Query::CreateTable(cmp_query)))
}
#[test]
fn create_table_no_modifiers() {
    let tokens = tokenize("CREATE TABLE IF NOT EXISTS users (id INT, name TEXT)").unwrap();

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    let mut row_type = VecMap::new();
    row_type.insert("id".to_owned(), FieldType::new(ScalarType::Int, Vec::new()));
    row_type.insert(
        "name".to_owned(),
        FieldType::new(ScalarType::Text, Vec::new()),
    );
    let cmp_query = CreateTable::new("users".to_owned(), Schema::new(row_type), true);

    assert_eq!(query, Ok(Query::CreateTable(cmp_query)))
}
#[test]
fn create_table_unknown_modifier() {
    let tokens = tokenize("CREATE TABLE IF NOT EXISTS users (id INT baba)").unwrap();

    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);

    assert_eq!(query, Err(ParseError::UnknownModifier { modifier: "baba" }))
}
#[test]
fn create_table_unexpected_token() {
    let tokens =
        tokenize("CREATE TABLE IF NOT EXISTS%users (id INT PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();
    println!("{:?}", tokens);
    let walker = TokenWalker::new(&tokens);
    let query = parse_create_query(walker);
    println!("{:?}", query);
    assert!(matches!(query, Err(ParseError::Other { message: _ })))
}
