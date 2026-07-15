use std::collections::HashMap;

use storage::{
    common_types::{DataType, DataValue, FieldType, ScalarType, ScalarValue, Schema, SchemaValue},
    db::Database,
    row::Row,
    table::Table,
};

use crate::{
    expr::{ArithmeticOp, ComparisonOp, Expr, ExprError, LiteralValue, LogicOp},
    queries::select::{Projection, SelectError, SelectQuery},
};

fn setup_database() -> Database {
    let db = Database::new();
    // Create first row
    let mut data = HashMap::new();
    data.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));
    data.insert(
        "name".to_string(),
        DataValue::Scalar(ScalarValue::Text("Alice".to_string())),
    );
    data.insert(
        "is_active".to_string(),
        DataValue::Scalar(ScalarValue::Bool(true)),
    );
    let type_value = SchemaValue::new(data);
    let row1 = Row::new(type_value);

    // Create second row
    let mut data = HashMap::new();
    data.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(25)));
    data.insert(
        "name".to_string(),
        DataValue::Scalar(ScalarValue::Text("Bob".to_string())),
    );
    data.insert(
        "is_active".to_string(),
        DataValue::Scalar(ScalarValue::Bool(false)),
    );
    let type_value = SchemaValue::new(data);
    let row2 = Row::new(type_value);

    let mut field_types = Vec::new();
    field_types.push((
        "age".to_string(),
        FieldType::new(DataType::Scalar(ScalarType::Int), vec![]),
    ));
    field_types.push((
        "name".to_string(),
        FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
    ));
    field_types.push((
        "is_active".to_string(),
        FieldType::new(DataType::Scalar(ScalarType::Bool), vec![]),
    ));
    let schema = Schema::new(field_types);
    // Create table
    let table = Table::new(schema);

    // Insert into database
    db.insert_table("test_table".to_string(), table).unwrap();
    let table = db.get_table("test_table").unwrap();
    table.insert_row(row1).unwrap();
    table.insert_row(row2).unwrap();

    db
}
/// Simple way to create DataValue from scalar
macro_rules! scalar {
    ($variant:ident($variant_value:expr)) => {
        DataValue::Scalar(ScalarValue::$variant($variant_value.into()))
    };
}
#[test]
fn execute_valid() {
    let db = setup_database();
    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("name".to_string()),
            Expr::Field("name".to_string()),
        )))),
    );

    let result = filter.execute(&db);
    assert!(result.is_ok());
    let indices = result.unwrap();
    assert_eq!(
        indices,
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true)),
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false)),],
        ]
    );
}

#[test]
fn unknown_field() {
    let db = setup_database();
    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Field("nonexistent_field".to_string())),
    );

    let result = filter.execute(&db);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        SelectError::ExprErr(ExprError::UnknownField { .. })
    ));
}

#[test]
fn not_applicable() {
    let db = setup_database();

    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("age".to_string()),
            Expr::Field("name".to_string()),
        )))),
    );

    let result = filter.execute(&db);
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        SelectError::ExprErr(ExprError::NotApplicable)
    ));
}
#[test]
fn bad_expr() {
    let db = setup_database();
    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Literal(LiteralValue::Text("Bad expr".to_owned()))),
    );
    let result = filter.execute(&db);
    assert_eq!(result, Err(SelectError::BadExpr));
}

#[test]
fn logical_and() {
    let db = setup_database();

    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Logical(Box::new(LogicOp::And(
            Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("name".to_string()),
                Expr::Literal(LiteralValue::Text("Alice".to_string())),
            ))),
            Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(30)),
            ))),
        )))),
    );

    let result = filter.execute(&db);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );
}

#[test]
fn logical_or() {
    let db = setup_database();

    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Logical(Box::new(LogicOp::Or(
            Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("name".to_string()),
                Expr::Literal(LiteralValue::Text("Alice".to_string())),
            ))),
            Expr::Comparison(Box::new(ComparisonOp::Eq(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(25)),
            ))),
        )))),
    );

    let result = filter.execute(&db);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );
}

#[test]
fn logical_not() {
    let db = setup_database();

    let filter = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Logical(Box::new(LogicOp::Not(Expr::Comparison(
            Box::new(ComparisonOp::Eq(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(30)),
            )),
        ))))),
    );

    let result = filter.execute(&db);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false)),
        ]]
    );
}

#[test]
fn all_integer_comparisons() {
    let db = setup_database();

    // Test Eq (age == 30)
    let filter_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(30)),
        )))),
    );
    let result_eq = filter_eq.execute(&db);
    assert!(result_eq.is_ok());
    assert_eq!(
        result_eq.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );

    // Test NotEq (age != 30)
    let filter_not_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::NotEq(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(30)),
        )))),
    );
    let result_not_eq = filter_not_eq.execute(&db);
    assert!(result_not_eq.is_ok());
    assert_eq!(
        result_not_eq.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test Less (age < 30)
    let filter_less = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Less(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(30)),
        )))),
    );
    let result_less = filter_less.execute(&db);
    assert!(result_less.is_ok());
    assert_eq!(
        result_less.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test LessEq (age <= 30)
    let filter_less_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::LessEq(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(30)),
        )))),
    );
    let result_less_eq = filter_less_eq.execute(&db);
    assert!(result_less_eq.is_ok());
    assert_eq!(
        result_less_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );

    // Test Greater (age > 25)
    let filter_greater = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Greater(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(25)),
        )))),
    );
    let result_greater = filter_greater.execute(&db);
    assert!(result_greater.is_ok());
    assert_eq!(
        result_greater.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );

    // Test GreaterEq (age >= 25)
    let filter_greater_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::GreaterEq(
            Expr::Field("age".to_string()),
            Expr::Literal(LiteralValue::Int(25)),
        )))),
    );
    let result_greater_eq = filter_greater_eq.execute(&db);
    assert!(result_greater_eq.is_ok());
    assert_eq!(
        result_greater_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );
}

#[test]
fn all_text_comparisons() {
    let db = setup_database();

    // Test Eq (name == "Alice")
    let filter_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Alice".to_string())),
        )))),
    );
    let result_eq = filter_eq.execute(&db);
    assert!(result_eq.is_ok());
    assert_eq!(
        result_eq.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true)),
        ]]
    );

    // Test NotEq (name != "Alice")
    let filter_not_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::NotEq(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Alice".to_string())),
        )))),
    );
    let result_not_eq = filter_not_eq.execute(&db);
    assert!(result_not_eq.is_ok());
    assert_eq!(
        result_not_eq.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test Less (name < "Bob")
    let filter_less = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Less(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Bob".to_string())),
        )))),
    );
    let result_less = filter_less.execute(&db);
    assert!(result_less.is_ok());
    assert_eq!(
        result_less.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );

    // Test LessEq (name <= "Bob")
    let filter_less_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::LessEq(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Bob".to_string())),
        )))),
    );
    let result_less_eq = filter_less_eq.execute(&db);
    assert!(result_less_eq.is_ok());
    assert_eq!(
        result_less_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );

    // Test Greater (name > "Alice")
    let filter_greater = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Greater(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Alice".to_string())),
        )))),
    );
    let result_greater = filter_greater.execute(&db);
    assert!(result_greater.is_ok());
    assert_eq!(
        result_greater.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test GreaterEq (name >= "Alice")
    let filter_greater_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::GreaterEq(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Alice".to_string())),
        )))),
    );
    let result_greater_eq = filter_greater_eq.execute(&db);
    assert!(result_greater_eq.is_ok());
    assert_eq!(
        result_greater_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );
}

#[test]
fn all_boolean_comparisons() {
    let db = setup_database();

    // Test Eq (is_active == 1)
    let filter_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(true)),
        )))),
    );
    let result_eq = filter_eq.execute(&db);
    assert!(result_eq.is_ok());
    assert_eq!(
        result_eq.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );

    // Test NotEq (is_active != 1)
    let filter_not_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::NotEq(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(true)),
        )))),
    );
    let result_not_eq = filter_not_eq.execute(&db);
    assert!(result_not_eq.is_ok());
    assert_eq!(
        result_not_eq.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test Less (is_active < 1)
    let filter_less = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Less(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(true)),
        )))),
    );
    let result_less = filter_less.execute(&db);
    assert!(result_less.is_ok());
    assert_eq!(
        result_less.unwrap(),
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false))
        ]]
    );

    // Test LessEq (is_active <= 1)
    let filter_less_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::LessEq(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(true)),
        )))),
    );
    let result_less_eq = filter_less_eq.execute(&db);
    assert!(result_less_eq.is_ok());
    assert_eq!(
        result_less_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );

    // Test Greater (is_active > 0)
    let filter_greater = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Greater(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(false)),
        )))),
    );
    let result_greater = filter_greater.execute(&db);
    assert!(result_greater.is_ok());
    assert_eq!(
        result_greater.unwrap(),
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true))
        ]]
    );

    // Test GreaterEq (is_active >= 0)
    let filter_greater_eq = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::GreaterEq(
            Expr::Field("is_active".to_string()),
            Expr::Literal(LiteralValue::Bool(false)),
        )))),
    );
    let result_greater_eq = filter_greater_eq.execute(&db);
    assert!(result_greater_eq.is_ok());
    assert_eq!(
        result_greater_eq.unwrap(),
        vec![
            vec![
                scalar!(Int(30)),
                scalar!(Text("Alice")),
                scalar!(Bool(true))
            ],
            vec![scalar!(Int(25)), scalar!(Text("Bob")), scalar!(Bool(false))]
        ]
    );
}

// Test arithmetic operations
#[test]
fn all_arithmetic_operations() {
    let db = setup_database();

    // Test Add (age + 5) > 10
    let filter_add = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Greater(
            Expr::Arithmetic(Box::new(ArithmeticOp::Add(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(5)),
            ))),
            Expr::Literal(LiteralValue::Int(10)),
        )))),
    );
    let result_add = filter_add.execute(&db);
    assert!(result_add.is_ok());

    // Test Subtract (age - 5) < 5
    let filter_subtract = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Less(
            Expr::Arithmetic(Box::new(ArithmeticOp::Subtract(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(5)),
            ))),
            Expr::Literal(LiteralValue::Int(5)),
        )))),
    );
    let result_subtract = filter_subtract.execute(&db);
    assert!(result_subtract.is_ok());

    // Test Multiply (age * 2) > 20
    let filter_multiply = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Greater(
            Expr::Arithmetic(Box::new(ArithmeticOp::Multiply(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(2)),
            ))),
            Expr::Literal(LiteralValue::Int(20)),
        )))),
    );
    let result_multiply = filter_multiply.execute(&db);
    assert!(result_multiply.is_ok());

    // Test Divide (age / 2) >= 10
    let filter_divide = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::GreaterEq(
            Expr::Arithmetic(Box::new(ArithmeticOp::Divide(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(2)),
            ))),
            Expr::Literal(LiteralValue::Int(10)),
        )))),
    );
    let result_divide = filter_divide.execute(&db);
    assert!(result_divide.is_ok());

    // Test Modulo (age % 5) == 0
    let filter_modulo = SelectQuery::new(
        "test_table".to_string(),
        Projection::Row,
        Some(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Arithmetic(Box::new(ArithmeticOp::Modulo(
                Expr::Field("age".to_string()),
                Expr::Literal(LiteralValue::Int(5)),
            ))),
            Expr::Literal(LiteralValue::Int(0)),
        )))),
    );
    let result_modulo = filter_modulo.execute(&db);
    assert!(result_modulo.is_ok());
}
