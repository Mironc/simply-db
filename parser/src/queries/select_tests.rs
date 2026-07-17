use query::{Query, queries::select::Projection};

use crate::{
    common::{ParseError, TokenWalker},
    queries::query::parse_select_query,
    tokenizer::tokenize,
};

#[test]
fn select_asterisk() {
    let tokens = tokenize("SELECT * FROM users").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        assert_eq!(query.table_name(), "users");
        assert!(matches!(query.projection(), Projection::Row));
        assert!(query.filter_expr().is_none());
    } else {
        panic!("Expected select query")
    }
}

#[test]
fn select_with_where() {
    let tokens = tokenize("SELECT * FROM products WHERE price").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        assert_eq!(query.table_name(), "products");
        assert!(
            query.filter_expr().is_some(),
            "WHERE expr should be created"
        );
    } else {
        panic!("Expected select query")
    }
}

#[test]
fn missing_select_keyword() {
    let tokens = tokenize("NOT_SELECT *").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert!(
        matches!(res, Err(ParseError::UnexpectedSymbol { .. })),
        "Expected error"
    );
}

#[test]
fn invalid_table_name_digit() {
    let tokens = tokenize("SELECT * FROM 123users").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert!(res.is_err());
    if let Err(ParseError::UnexpectedSymbol { expected, .. }) = res {
        assert!(expected.contains("valid table name"));
    } else {
        panic!("Expected error");
    }
}

#[test]
fn unclosed_bracket_in_projection() {
    let tokens = tokenize("SELECT (id,)) FROM").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert_eq!(res, Err(ParseError::UnclosedBracket(')')));
}

#[test]
fn multiple_expressions_projection() {
    let tokens = tokenize("SELECT id, age, is_active FROM users").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        if let Projection::Expr(exprs) = query.projection() {
            assert_eq!(
                exprs.len(),
                3,
                "expected 3 expressions in projections (id, age and is_active)"
            );
        } else {
            panic!("Expected projection variant Projection::Expr");
        }
    } else {
        panic!("Expected select query")
    }
}
#[test]
fn select_with_complex_where() {
    let tokens = tokenize("SELECT age FROM users WHERE age > 18").unwrap();
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Ok(Query::Select(query)) = res {
        assert_eq!(query.table_name(), "users");
        assert!(matches!(query.projection(), Projection::Expr(_)));

        assert!(
            query.filter_expr().is_some(),
            "filter expr shouldn't be empty"
        );
    } else {
        panic!("expected select query");
    }
}
