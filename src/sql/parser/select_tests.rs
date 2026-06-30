use crate::{self as simply_db, sql::query::Query};
use simply_db::{
    queries::select::Projection,
    sql::parser::{
        common::{ParseError, TokenWalker},
        query::parse_select_query,
        tokenizer::tokenize,
    },
};

// 1. Успешный тест: Базовый SELECT * FROM table
#[test]
fn select_asterisk() {
    let tokens = tokenize("SELECT * FROM users");
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

// 2. Успешный тест: SELECT * FROM table WHERE condition
#[test]
fn select_with_where() {
    let tokens = tokenize("SELECT * FROM products WHERE price");
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        assert_eq!(query.table_name(), "products");
        assert!(
            query.filter_expr().is_some(),
            "WHERE выражение должно быть распарсено"
        );
    } else {
        panic!("Expected select query")
    }
}

// 3. Тест на ошибку: Пропущен токен SELECT в начале
#[test]
fn missing_select_keyword() {
    let tokens = tokenize("NOT_SELECT *");
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert!(
        matches!(res, Err(ParseError::UnexpectedSymbol { .. })),
        "Должна вернуться ошибка UnexpectedSymbol, так как запрос не начинается с SELECT"
    );
}

// 4. Тест на ошибку: Имя таблицы начинается с цифры
#[test]
fn invalid_table_name_digit() {
    let tokens = tokenize("SELECT * FROM 123users");
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert!(res.is_err());
    if let Err(ParseError::UnexpectedSymbol { expected, .. }) = res {
        assert!(expected.contains("valid table name"));
    } else {
        panic!("Ожидалась ошибка валидации имени таблицы");
    }
}

// 5. Тест на ошибку: Незакрытая скобка внутри проекций полей
#[test]
fn unclosed_bracket_in_projection() {
    let tokens = tokenize("SELECT (id,)) FROM");
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);
    assert_eq!(res, Err(ParseError::UnclosedBracket(')')));
}

// 6. Тест для проверки твоего бага (Парсинг списка полей)
// Оставляю его, чтобы ты проверил фикс. Сейчас он, скорее всего, упадет,
// так как "age" не запишется в вектор expressions.
#[test]
fn multiple_expressions_projection() {
    let tokens = tokenize("SELECT id, age, is_active FROM users");
    println!("{:?}", tokens);
    let walker = TokenWalker::new(&tokens);

    let res = parse_select_query(walker);

    if let Query::Select(query) = res.unwrap() {
        if let Projection::Expr(exprs) = query.projection() {
            assert_eq!(
                exprs.len(),
                3,
                "Ожидалось 2 выражения в проекции (id и age)"
            );
        } else {
            panic!("Ожидалась проекция типа Projection::Expr");
        }
    } else {
        panic!("Expected select query")
    }
}
#[test]
fn select_with_complex_where() {
    let tokens = tokenize("SELECT age FROM users WHERE age > 18");
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
