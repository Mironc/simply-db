use query::{
    Query,
    expr::{ComparisonOp, Expr, LiteralValue},
    queries::delete::{DeleteQuery, DeleteRows, DropTable, TruncateTable},
};

use crate::{
    ParseError,
    common::Parser,
    lexer::Lexer,
    queries::query::{parse_delete_query, parse_drop_query, parse_truncate_query},
};

#[test]
fn truncate() {
    let lexer = Lexer::new("TRUNCATE TABLE table");
    let parser = Parser::new(lexer).unwrap();

    let parsed_query = parse_truncate_query(parser).unwrap();
    let cmp_query = TruncateTable::new("table".to_string());
    assert_eq!(
        parsed_query,
        Query::Delete(DeleteQuery::TruncateTable(cmp_query))
    );
}

#[test]
fn truncate_ident_starts_with_digit_error() {
    let lexer = Lexer::new("TRUNCATE TABLE 123table");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_truncate_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "123table"
        }
    );
}

#[test]
fn truncate_missing_table_name_error() {
    let lexer = Lexer::new("TRUNCATE TABLE");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_truncate_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "Eof"
        }
    );
}

#[test]
fn truncate_missing_keyword_error() {
    let test_cases = vec![("TRUNCATE table", "TABLE"), ("TABLE table", "TRUNCATE")];
    for (case, missing) in test_cases {
        let lexer = Lexer::new(case);
        let parser = Parser::new(lexer).unwrap();

        let error = parse_truncate_query(parser).unwrap_err();
        if !matches!(
            error,
            ParseError::UnexpectedSymbol { expected: sym, .. }
            if sym == missing
        ) {
            panic!("Expected other error type");
        }
    }
}

#[test]
fn drop() {
    let lexer = Lexer::new("DROP TABLE table");
    let parser = Parser::new(lexer).unwrap();

    let parsed_query = parse_drop_query(parser).unwrap();
    let cmp_query = DropTable::new("table".to_string());
    assert_eq!(
        parsed_query,
        Query::Delete(DeleteQuery::DropTable(cmp_query))
    );
}

#[test]
fn drop_ident_starts_with_digit_error() {
    let lexer = Lexer::new("DROP TABLE 123table");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_drop_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "123table"
        }
    );
}

#[test]
fn drop_missing_table_name_error() {
    let lexer = Lexer::new("DROP TABLE");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_drop_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "Eof"
        }
    );
}

#[test]
fn drop_missing_keyword_error() {
    let test_cases = vec![("DROP table", "TABLE"), ("TABLE table", "DROP")];
    for (case, missing) in test_cases {
        let lexer = Lexer::new(case);
        let parser = Parser::new(lexer).unwrap();

        let error = parse_drop_query(parser).unwrap_err();
        if !matches!(
            error,
            ParseError::UnexpectedSymbol { expected: sym, .. }
            if sym == missing
        ) {
            panic!("Expected other error type")
        }
    }
}

#[test]
fn delete() {
    let lexer = Lexer::new("DELETE FROM table WHERE is_active == FALSE");
    let parser = Parser::new(lexer).unwrap();

    let parsed_query = parse_delete_query(parser).unwrap();
    let cmp_query = DeleteRows::new(
        "table".to_string(),
        Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(false)),
        ))),
    );
    assert_eq!(
        parsed_query,
        Query::Delete(DeleteQuery::DeleteRows(cmp_query))
    );
}

#[test]
fn delete_ident_starts_with_digit_error() {
    let lexer = Lexer::new("DELETE FROM 123table WHERE is_active == FALSE");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_delete_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "123table"
        }
    );
}

#[test]
fn delete_missing_keyword_error() {
    let test_cases = vec![
        ("FROM table WHERE is_active == FALSE", "DELETE"),
        ("DELETE table WHERE is_active == FALSE", "FROM"),
        ("DELETE FROM table  is_active == FALSE", "WHERE"),
    ];
    for (case, missing) in test_cases {
        let lexer = Lexer::new(case);
        let parser = Parser::new(lexer).unwrap();

        let error = parse_delete_query(parser).unwrap_err();
        if !matches!(
            error,
            ParseError::UnexpectedSymbol { expected: sym, .. }
            if sym == missing
        ) {
            panic!("Expected other error type")
        }
    }
}

#[test]
fn delete_missing_table_name_error() {
    let lexer = Lexer::new("DELETE FROM WHERE is_active == FALSE");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_delete_query(parser).unwrap_err();
    assert_eq!(
        error,
        ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: "WHERE"
        }
    );
}

#[test]
fn delete_missing_expr_error() {
    let lexer = Lexer::new("DELETE FROM table WHERE ");
    let parser = Parser::new(lexer).unwrap();

    let error = parse_delete_query(parser).unwrap_err();
    assert_eq!(error, ParseError::UnexpectedEof);
}
