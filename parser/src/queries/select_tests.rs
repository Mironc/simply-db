use query::{
    Query,
    expr::{ComparisonOp, Expr, LiteralValue},
    queries::select::Projection,
};

use crate::{
    common::{ParseError, Parser},
    lexer::{Lexer, TokenValue},
    queries::query::parse_select_query,
};

fn collect_tokens_until_eof<'a>(
    mut lexer: Lexer<'a>,
) -> Result<Vec<(TokenValue<'a>, usize)>, ParseError<'a>> {
    let mut tokens = Vec::new();
    while let token = lexer.next_token()?
        && token != TokenValue::EOF
    {
        tokens.push((token, lexer.position()));
    }
    Ok(tokens)
}
#[test]
fn select_rows() {
    let lexer = Lexer::new("SELECT * FROM users");
    let walker = Parser::new(lexer).unwrap();

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
fn select_with_filter_and_projection_expr() {
    let lexer = Lexer::new("SELECT age FROM users WHERE age > 18");
    let walker = Parser::new(lexer.clone()).unwrap();

    let res = parse_select_query(walker);

    if let Ok(Query::Select(query)) = res {
        assert_eq!(query.table_name(), "users");
        assert_eq!(
            query.projection(),
            &Projection::Expr(vec![Expr::Field("age".to_owned())])
        );

        assert_eq!(
            query.filter_expr(),
            Some(&Expr::Comparison(Box::new(ComparisonOp::Greater(
                Expr::Field("age".to_owned()),
                Expr::Literal(LiteralValue::Int(18))
            ))))
        );
    } else {
        println!("{:?}", res);
        panic!("expected select query");
    }
}

#[test]
fn select_skip_take() {
    let lexer = Lexer::new("SELECT * FROM users TAKE 15 SKIP 30");
    let walker = Parser::new(lexer).unwrap();

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        assert_eq!(query.take(), Some(15));
        assert_eq!(query.skip(), Some(30));
    } else {
        panic!("Expected select query")
    }
}
#[test]
fn missing_keywords() {
    let test_cases = [("* FROM users"), ("SELECT * users")];
    for test in test_cases {
        let lexer = Lexer::new(test);
        let walker = Parser::new(lexer).unwrap();

        let res = parse_select_query(walker);
        assert!(
            matches!(res, Err(ParseError::UnexpectedSymbol { .. })),
            "{:?}",
            res
        );
    }
}
#[test]
fn missing_arguments() {
    let test_cases = [("SELECT * FROM users SKIP"), ("SELECT * FROM users TAKE")];
    for test in test_cases {
        let lexer = Lexer::new(test);
        let walker = Parser::new(lexer).unwrap();

        let res = parse_select_query(walker);
        assert!(
            matches!(
                res,
                Err(ParseError::UnexpectedValue {
                    expected: "Expected numeric argument"
                })
            ),
            "{:?}",
            res
        );
    }
}

#[test]
fn invalid_table_name_digit() {
    let lexer = Lexer::new("SELECT * FROM 123users");
    let walker = Parser::new(lexer).unwrap();

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
    let lexer = Lexer::new("SELECT (id,)) FROM");
    let walker = Parser::new(lexer).unwrap();

    let res = parse_select_query(walker);
    assert_eq!(res, Err(ParseError::UnclosedBracket(')')));
}

#[test]
fn multiple_expressions_projection() {
    let lexer = Lexer::new("SELECT id, age, is_active FROM users");
    let walker = Parser::new(lexer).unwrap();
    println!("{:?}", collect_tokens_until_eof(lexer.clone()));
    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        if let Projection::Expr(exprs) = query.projection() {
            assert_eq!(
                exprs,
                &vec![
                    Expr::Field("id".to_owned()),
                    Expr::Field("age".to_owned()),
                    Expr::Field("is_active".to_owned())
                ],
            );
        } else {
            panic!("Expected projection variant Projection::Expr");
        }
    } else {
        panic!("Expected select query")
    }
}
