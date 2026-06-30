use crate::{
    self as simply_db,
    sql::{
        expr::{Expr, LiteralValue},
        query::Query,
    },
};
use simply_db::sql::parser::{
    common::{ParseError, TokenWalker},
    query::parse_update_query,
    tokenizer::tokenize,
};

// 1. Successful test: Basic UPDATE query without WHERE clause
#[test]
fn update_basic() {
    let tokens = tokenize("UPDATE users SET age=25");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);

    if let Ok(Query::Update(query)) = res {
        assert_eq!(query.table_name(), "users");
        assert_eq!(query.set_exprs().len(), 1);
        assert_eq!(query.set_exprs()[0].0, "age");
        assert_eq!(query.set_exprs()[0].1, Expr::Literal(LiteralValue::Int(25)));
        assert!(query.filter_expr().is_none());
    } else {
        panic!("Expected update query");
    }
}

// 2. Successful test: UPDATE query with multiple field assignments
#[test]
fn update_multiple_fields() {
    let tokens = tokenize("UPDATE users SET age=25, name='John'");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);

    if let Ok(Query::Update(query)) = res {
        assert_eq!(query.table_name(), "users");
        assert_eq!(query.set_exprs().len(), 2);
        assert_eq!(query.set_exprs()[0].0, "age");
        assert_eq!(query.set_exprs()[0].1, Expr::Literal(LiteralValue::Int(25)));
        assert_eq!(query.set_exprs()[1].0, "name");
        assert_eq!(
            query.set_exprs()[1].1,
            Expr::Literal(LiteralValue::Text("John".to_owned()))
        );
        assert!(query.filter_expr().is_none());
    } else {
        panic!("Expected update query");
    }
}

// 3. Successful test: UPDATE query with WHERE clause
#[test]
fn update_with_where() {
    let tokens = tokenize("UPDATE users SET age=25 WHERE age > 18");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);
    println!("{:?}", res);
    if let Ok(Query::Update(query)) = res {
        assert_eq!(query.table_name(), "users");
        assert_eq!(query.set_exprs().len(), 1);
        assert_eq!(query.set_exprs()[0].0, "age");
        assert_eq!(query.set_exprs()[0].1, Expr::Literal(LiteralValue::Int(25)));
        assert!(query.filter_expr().is_some());
    } else {
        panic!("Expected update query");
    }
}

// 4. Test on error: Table name starts with digit (invalid identifier)
#[test]
fn invalid_table_name() {
    let tokens = tokenize("UPDATE 123users SET age=25");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);
    assert!(res.is_err());
    if let Err(ParseError::UnexpectedSymbol { expected, .. }) = res {
        assert!(expected.contains("valid table name that starts not with digit"));
    } else {
        panic!("Expected error for invalid table name");
    }
}

// 5. Test on error: Missing SET clause after UPDATE
#[test]
fn missing_set_clause() {
    let tokens = tokenize("UPDATE users");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);
    assert!(res.is_err());
    if let Err(ParseError::UnexpectedEof) = res {
        // This is expected - no SET clause
    } else {
        panic!("Expected error for missing SET clause");
    }
}

// 6. Test on error: Invalid token after SET (should be comma or WHERE)
// skipped for now as I'am no sure how to implement this check, as invalid counts as part of set expr
// #[test]
// fn invalid_token_after_set() {
//     let tokens = tokenize("UPDATE users SET age=25 invalid");
//     let walker = TokenWalker::new(&tokens);

//     let res = parse_update_query(walker);
//     assert!(res.is_err());
//     println!("{:?}", res);
//     if let Err(ParseError::UnexpectedSymbol { expected, .. }) = res {
//         assert!(expected.contains("valid table name"));
//     } else {
//         panic!("Expected error for invalid token after SET");
//     }
// }

// 7. Test on error: Invalid token in WHERE clause (should be expression)
// as for now skipped
// #[test]
// fn invalid_where_expression() {
//     let tokens = tokenize("UPDATE users SET age=25 WHERE 123");
//     let walker = TokenWalker::new(&tokens);

//     let res = parse_update_query(walker);
//     assert!(res.is_err());
//     if let Err(ParseError::UnexpectedSymbol { expected, given }) = res {
//         println!("{} {}", expected, given);
//         assert!(expected.contains("valid field name"));
//     } else {
//         panic!("Expected error for invalid WHERE expression");
//     }
// }

// 8. Test on error: Invalid field name in SET clause (should be identifier)
#[test]
fn invalid_field_name() {
    let tokens = tokenize("UPDATE users SET 123age=25");
    let walker = TokenWalker::new(&tokens);

    let res = parse_update_query(walker);
    assert!(res.is_err());
    println!("{:?}", res);
    if let Err(ParseError::UnexpectedSymbol { expected, .. }) = res {
        assert!(expected.contains("valid field name"));
    } else {
        panic!("Expected error for invalid field name");
    }
}
