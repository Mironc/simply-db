use query::{Query, queries::delete::DeleteQuery};

use crate::{ParseError, lexer::Lexer, queries::query::parse_query};

#[test]
fn insert() {
    let lexer = Lexer::new("INSERT INTO table (int,string) VALUES (0,'Steve')");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::Insert(_)));
}

#[test]
fn insert_error() {
    let lexer = Lexer::new("INSERT INTO table (int,string) VALUES (0,'Steve)");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn create_table() {
    let lexer = Lexer::new("CREATE TABLE table (field1 INT,field2 TEXT)");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::CreateTable(_)));
}

#[test]
fn create_table_error() {
    let lexer = Lexer::new("CREATE TABLE table (field1 INT,field2 TEXT");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn select() {
    let lexer = Lexer::new("SELECT * FROM table");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::Select(_)));
}

#[test]
fn select_error() {
    let lexer = Lexer::new("SELECT * FROM");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn update() {
    let lexer = Lexer::new("UPDATE table SET field1=0");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::Update(_)));
}

#[test]
fn update_error() {
    let lexer = Lexer::new("UPDATE table SET field1");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn drop() {
    let lexer = Lexer::new("DROP TABLE table");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::Delete(DeleteQuery::DropTable(_))));
}

#[test]
fn drop_error() {
    let lexer = Lexer::new("DROP TABLE");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn truncate() {
    let lexer = Lexer::new("TRUNCATE TABLE table");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(
        query,
        Query::Delete(DeleteQuery::TruncateTable(_))
    ))
}

#[test]
fn truncate_error() {
    let lexer = Lexer::new("TRUNCATE TABLE");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn delete() {
    let lexer = Lexer::new("DELETE FROM table WHERE id == 0");
    let query = parse_query(lexer).unwrap();

    assert!(matches!(query, Query::Delete(DeleteQuery::DeleteRows(_))))
}

#[test]
fn delete_error() {
    let lexer = Lexer::new("DELETE FROM table WHERE ");
    assert!(parse_query(lexer).is_err());
}

#[test]
fn unknown_instruction() {
    let lexer = Lexer::new("UNKNOWN INSTRUCTION");
    let error = parse_query(lexer).unwrap_err();
    assert_eq!(error, ParseError::UnknownInstruction);
}
