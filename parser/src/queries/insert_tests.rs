use query::{Query, queries::insert::InsertQuery};
use storage::{row::Row, scalar};

use crate::{
    common::{ParseError, TokenWalker},
    queries::query::{parse_insert_data, parse_insert_fields, parse_insert_query, parse_query},
    tokenizer::tokenize,
};

#[test]
fn insert_fields_success() {
    // Test with one field
    let tokens = tokenize("(field1)").unwrap();
    let mut walker = TokenWalker::new(&tokens);
    let result = parse_insert_fields(&mut walker);
    assert_eq!(result, Ok(vec!["field1".to_string()]));
    // Test with multiple fields
    let tokens = tokenize("(field1, field2, field3)").unwrap();
    let mut walker = TokenWalker::new(&tokens);
    let result = parse_insert_fields(&mut walker);
    assert_eq!(
        result,
        Ok(vec![
            "field1".to_string(),
            "field2".to_string(),
            "field3".to_string()
        ])
    );
}

#[test]
fn insert_fields_empty() {
    let tokens = tokenize("( )").unwrap();
    let mut walker = TokenWalker::new(&tokens);

    let result = parse_insert_fields(&mut walker);
    // Expecting failure because the implementation requires at least one field name after '('
    assert_eq!(
        result,
        Err(ParseError::UnexpectedSymbol {
            expected: "Expected field name".into(),
            given: ")".into()
        })
    )
}

#[test]
fn insert_data_success() {
    // Parsing insert data with mulptiple fields
    let tokens = tokenize("('test' , '1')").unwrap();
    let mut walker = TokenWalker::new(&tokens);

    let result = parse_insert_data(&mut walker);
    assert!(result.is_ok(), "Data parsing failed: {:?}", result.err());

    let data = result.unwrap();
    // Asserting only 2 rows were parsed successfully in this minimal simulation.
    assert_eq!(data.len(), 2);
}

#[test]
fn insert_row_count_mismatch() {
    let tokens = tokenize(" INSERT  INTO table1  (f1,f2) VALUES  ('text')").unwrap();
    let walker = TokenWalker::new(&tokens);

    let insert_query = parse_insert_query(walker);
    assert!(insert_query.is_err());

    assert_eq!(
        insert_query,
        Err(ParseError::FieldNumberMismatch {
            expected: 2,
            provided: 1
        })
    );
}
#[test]
fn insert_query() {
    // Test insert query with multiple fields one row
    let tokens = tokenize("INSERT INTO table (int, string) VALUES (100, 'text' )").unwrap();
    println!("{:?}", tokens);
    let insert_query = parse_query(tokens);
    let values = vec![scalar!(Int(100)), scalar!(Text("text"))];
    let cmp_query = InsertQuery::new(
        "table".to_owned(),
        vec!["int".to_owned(), "string".to_owned()],
        vec![Row::new(values)],
    );
    assert_eq!(insert_query, Ok(Query::Insert(cmp_query)));

    // Test insert query with multiple fields with multiple rows
    let tokens =
        tokenize("INSERT INTO table (int, string) VALUES (100, 'text'), (50, 't'),(17, 'Steve')")
            .unwrap();
    let walker = TokenWalker::new(&tokens);
    let insert_query = parse_insert_query(walker);
    let mut rows = Vec::new();

    let fields = vec![scalar!(Int(100)), scalar!(Text("text"))];
    rows.push(Row::new(fields.clone()));

    let fields = vec![scalar!(Int(50)), scalar!(Text("t"))];
    rows.push(Row::new(fields.clone()));

    let fields = vec![scalar!(Int(17)), scalar!(Text("Steve"))];
    rows.push(Row::new(fields));

    let cmp_query = InsertQuery::new(
        "table".to_owned(),
        vec!["int".to_owned(), "string".to_owned()],
        rows,
    );
    assert_eq!(insert_query, Ok(Query::Insert(cmp_query)));
}
