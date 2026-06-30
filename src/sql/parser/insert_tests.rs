use crate::{self as simply_db, sql::query::Query};
use simply_db::{
    common_types::SchemaValue,
    queries::insert::InsertQuery,
    sql::{
        parser::common::{ParseError, ParseResult, TokenWalker},
        parser::query::{parse_insert_data, parse_insert_fields, parse_insert_query, parse_query},
        parser::tokenizer::tokenize,
    },
};

use simply_db::{hashmap, scalar};

#[test]
fn insert_fields_success() {
    // Test with one field
    let tokens = tokenize("(field1)");
    let mut walker = TokenWalker::new(&tokens);
    let result = parse_insert_fields(&mut walker);
    assert_eq!(result, Ok(vec!["field1".to_string()]));
    // Test with multiple fields
    let tokens = tokenize("(field1, field2, field3)");
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
    let tokens = tokenize("( )");
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
fn insert_data_success() -> ParseResult<()> {
    // Parsing insert data with mulptiple fields
    let tokens = tokenize("('test' , '1')");
    let mut walker = TokenWalker::new(&tokens);

    let result = parse_insert_data(&mut walker);
    assert!(result.is_ok(), "Data parsing failed: {:?}", result.err());

    let data = result?;
    // Asserting only 2 rows were parsed successfully in this minimal simulation.
    assert_eq!(data.len(), 2);

    Ok(())
}

#[test]
fn insert_row_count_mismatch() {
    let tokens = tokenize(" INSERT  INTO table1  (f1,f2) VALUES  ('text')");
    let walker = TokenWalker::new(&tokens);

    let insert_query = parse_insert_query(walker);
    assert!(insert_query.is_err());

    assert_eq!(
        insert_query,
        Err(ParseError::Other {
            message:
                "number of fields not matches with number of values in row. Expected:2, Provided:1"
                    .into()
        })
    );
}
#[test]
fn insert_query() {
    // Test insert query with multiple fields one row
    let tokens = tokenize("INSERT INTO table (int, string) VALUES (100, 'text' )");
    let insert_query = parse_query(tokens);
    let values = hashmap!(
            "int".to_owned()=> scalar!(Int(100)),
            "string".to_owned()=> scalar!(Text("text"))
    );
    let cmp_query = InsertQuery::new("table".to_owned(), vec![SchemaValue::new(values)]);
    assert_eq!(insert_query, Ok(Query::Insert(cmp_query)));

    // Test insert query with multiple fields with multiple rows
    let tokens =
        tokenize("INSERT INTO table (int, string) VALUES (100, 'text'), (50, 't'),(17, 'Steve')");
    let walker = TokenWalker::new(&tokens);
    let insert_query = parse_insert_query(walker);
    let mut rows = Vec::new();

    let fields = hashmap!(
        "int".to_owned() => scalar!(Int(100)),
        "string".to_owned() => scalar!(Text("text"))
    );
    rows.push(SchemaValue::new(fields.clone()));

    let fields = hashmap!(
            "int".to_owned() => scalar!(Int(50)),
            "string".to_owned() => scalar!(Text("t"))
    );
    rows.push(SchemaValue::new(fields.clone()));

    let fields = hashmap!(
        "int".to_owned() => scalar!(Int(17)),
        "string".to_owned() => scalar!(Text("Steve"))
    );
    rows.push(SchemaValue::new(fields));

    let cmp_query = InsertQuery::new("table".to_owned(), rows);
    assert_eq!(insert_query, Ok(Query::Insert(cmp_query)));
}
