use crate as simply_db;
use simply_db::common_types::SchemaValue;
use simply_db::common_types::{DataValue, ScalarValue};
use simply_db::row::Row;
use simply_db::sql::parser::common::TokenWalker;
use simply_db::sql::parser::expr::parse_expr;
use simply_db::sql::parser::tokenizer::{TokenValue, tokenize};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
// Helper macro to parse a string cleanly in tests
macro_rules! parse {
    ($input:expr) => {{
        let mut tokens = tokenize($input);
        tokens.push(TokenValue::Blank);
        parse_expr(&mut TokenWalker::new(&tokens), tokens.len())
    }};
}
fn null_row() -> Row {
    Row::new(SchemaValue::new(HashMap::default()))
}
#[test]
fn arithmetic() {
    let expr = parse!("5+3");

    let expected = DataValue::Scalar(ScalarValue::Int(8));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn arithmetic_precedence() {
    let expr = parse!("2+(5+3)*4+3");

    let expected = DataValue::Scalar(ScalarValue::Int(37));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn arithmetic_with_parentheses() {
    let expr = parse!("(5 + 3) * 4");

    let expected = DataValue::Scalar(ScalarValue::Int(32));
    println!("{:?}", expr);
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn boolean_operations() {
    let expr = parse!("NOT TRUE");

    let expected = DataValue::Scalar(ScalarValue::Bool(false));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn boolean_and() {
    let expr = parse!(" TRUE AND  TRUE ");

    let expected = DataValue::Scalar(ScalarValue::Bool(true));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn boolean_or() {
    let expr = parse!("TRUE OR FALSE");

    let expected = DataValue::Scalar(ScalarValue::Bool(true));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn field_references() {
    let expr = parse!("id");

    let expected = DataValue::Scalar(ScalarValue::Int(1));
    let mut row_data = HashMap::new();
    row_data.insert("id".to_string(), DataValue::Scalar(ScalarValue::Int(1)));
    let row = Row::new(SchemaValue::new(row_data));
    assert_eq!(expr.unwrap().execute(&row).unwrap().deref(), &expected);
}

#[test]
fn literals() {
    let expr = parse!("42");

    let expected = DataValue::Scalar(ScalarValue::Int(42));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn modulo_operation() {
    let expr = parse!("10 % 3 ");

    let expected = DataValue::Scalar(ScalarValue::Int(1));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn division_operation() {
    let expr = parse!("10 / 2");

    let expected = DataValue::Scalar(ScalarValue::Int(5));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn multiplication_operation() {
    let expr = parse!("5 * 6");

    let expected = DataValue::Scalar(ScalarValue::Int(30));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn complex_boolean_expression() {
    let expr = parse!("5 + 4 < 10 AND 1 > 50");

    let expected = DataValue::Scalar(ScalarValue::Bool(false));
    println!("{:?}", expr);
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn nested_boolean_expression() {
    let expr = parse!("NOT (TRUE AND FALSE) OR (5 > 3)");

    let expected = DataValue::Scalar(ScalarValue::Bool(true));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn mixed_operations() {
    let expr = parse!("(5 + 3) * 4 + 2 > 20 AND 10 % 3 == 1");

    let expected = DataValue::Scalar(ScalarValue::Bool(true));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn arithmetic_with_boolean_comparison() {
    let expr = parse!("5 + 3 >= 8 AND 10 / 2 == 5");

    let expected = DataValue::Scalar(ScalarValue::Bool(true));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}

#[test]
fn nested_arithmetic_with_parentheses() {
    let expr = parse!("(5 + 3) * (4 + 2) > 50");
    let expected = DataValue::Scalar(ScalarValue::Bool(false));
    assert_eq!(
        expr.unwrap().execute(&null_row()).unwrap().deref(),
        &expected
    );
}
#[test]
fn syntax_errors() {
    assert!(parse!("5 +").is_err());
    assert!(parse!("* 5").is_err());
    assert!(parse!("(5 + 3").is_err()); // Missing closing bracket
    assert!(parse!("5 5").is_err()); // Missing operator
}
#[test]
fn test_bench_func() {
    let complex = parse!(
        "(((1 + 2) * (3 - 4)) / (5 % 6) == -1 OR id > 10) AND (status == 'active' OR NOT (val <= 100 AND price * 2 > 50))"
    );
    assert!(complex.is_ok());
}
use rstest::rstest;

/// First one is literal, second one is how it named in test
struct Op(&'static str, &'static str);

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}
const PLUS: Op = Op("+", "add");
const MINUS: Op = Op("-", "sub");
const MULTIPLY: Op = Op("*", "mul");
const DIVIDE: Op = Op("/", "div");
const MODULO: Op = Op("%", "mod");
const EQUAL: Op = Op("==", "eq");
const NEQUAL: Op = Op("!=", "neq");
const LESS: Op = Op("<", "less");
const LESSEQ: Op = Op("<=", "less_eq");
const GREATER: Op = Op(">", "greater");
const GREATEREQ: Op = Op(">=", "greater_eq");
#[rstest]
fn combinatoric_arithmetic(
    #[values(PLUS, MINUS, MULTIPLY, DIVIDE, MODULO)] op1: Op,
    #[values(PLUS, MINUS, MULTIPLY, DIVIDE, MODULO)] op2: Op,
    #[values(LESS, LESSEQ, GREATER, GREATEREQ, EQUAL, NEQUAL)] cmp: Op,
) {
    use boa_engine::{Context, Source};
    let values = vec!["2", "3", "5"];

    let mut js_context = Context::default();
    let expression = format!(
        "({} {} {}) {} {} {} 10",
        values[0], op1.0, values[1], op2.0, values[2], cmp.0
    );

    let js_source = Source::from_bytes(expression.as_bytes());
    let js_res = js_context.eval(js_source).unwrap();
    let expected_bool = js_res.as_boolean().unwrap();

    let expr = parse!(&expression).unwrap();
    let row = null_row();
    let your_res = expr.execute(&row);

    if let Ok(DataValue::Scalar(ScalarValue::Bool(b))) = your_res.as_deref() {
        assert_eq!(*b, expected_bool, "Bad priorities: {}", expression);
    } else {
        panic!("Expected bool")
    }
}

const AND: Op = Op("AND", "AND");
const OR: Op = Op("OR", "OR");
const NOT: Op = Op("NOT", "NOT");
const FALSE: Op = Op("FALSE", "FALSE");
const TRUE: Op = Op("TRUE", "TRUE");
const NOTHING: Op = Op("", "");
#[rstest]
fn combinatoric_boolean(
    #[values(NOTHING, NOT)] log1: Op,
    #[values(TRUE, FALSE)] value1: Op,
    #[values(AND, OR)] log2: Op,
    #[values(TRUE, FALSE)] value2: Op,
    #[values(AND, OR)] log3: Op,
    #[values(TRUE, FALSE)] value3: Op,
) {
    use boa_engine::{Context, Source};
    let mut js_context = Context::default();

    let expression = format!(
        "{} {} {} ({} {} {})",
        log1.0, value1.0, log2.0, value2.0, log3.0, value3.0
    );
    let js_expression = expression
        .replace("NOT", "!")
        .replace("AND", "&&")
        .replace("OR", "||")
        .replace("TRUE", "true")
        .replace("FALSE", "false");
    let js_source = Source::from_bytes(js_expression.as_bytes());
    let js_res = js_context.eval(js_source).unwrap();
    let expected_bool = js_res.as_boolean().unwrap();

    let expr = parse!(&expression).unwrap();
    let row = null_row();
    let your_res = expr.execute(&row);

    if let Ok(DataValue::Scalar(ScalarValue::Bool(b))) = your_res.as_deref() {
        assert_eq!(*b, expected_bool, "Bad priorities: {}", expression);
    } else {
        panic!("Expected bool")
    }
}
