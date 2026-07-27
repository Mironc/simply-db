use storage::{
    common_types::ScalarType,
    db::Database,
    row::Row,
    scalar,
    schema::{FieldType, Schema},
    table::Table,
};
use structures::VecMap;

use crate::{
    expr::{ComparisonOp, Expr, LiteralValue},
    queries::select::{Projection, SelectError, SelectQueryBuilder},
};

fn setup_database() -> Database {
    let db = Database::new();
    // Create first row
    let data = vec![
        scalar!(Int(30)),
        scalar!(Text("Alice".to_owned())),
        scalar!(Bool(true)),
    ];
    let row1 = Row::new(data);

    // Create second row
    let data = vec![
        scalar!(Int(25)),
        scalar!(Text("Bob".to_owned())),
        scalar!(Bool(false)),
    ];
    let row2 = Row::new(data);

    let mut field_types = VecMap::new();
    field_types.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
    field_types.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
    field_types.insert(
        "is_active".to_string(),
        FieldType::new(ScalarType::Bool, vec![]),
    );
    let schema = Schema::new(field_types);
    // Create table
    let table = Table::new(schema);

    // Insert into database
    db.insert_table("test_table".to_string(), table).unwrap();
    let table = db.get_table("test_table").unwrap();
    table
        .insert_row(
            &vec!["age".to_owned(), "name".to_owned(), "is_active".to_owned()],
            row1,
        )
        .unwrap();
    table
        .insert_row(
            &vec!["age".to_owned(), "name".to_owned(), "is_active".to_owned()],
            row2,
        )
        .unwrap();

    db
}

#[test]
fn no_filter_row_projection_success() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row).build();

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
fn filter_expr_row_projection_success() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("name".to_string()),
            Expr::Literal(LiteralValue::Text("Alice".to_owned())),
        ))))
        .build();

    let result = filter.execute(&db);
    assert!(result.is_ok());
    let indices = result.unwrap();
    assert_eq!(
        indices,
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true)),
        ],]
    );
}

#[test]
fn filter_fails_on_non_boolean_expression() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .filter_expr(Expr::Literal(LiteralValue::Text("Bad expr".to_owned())))
        .build();
    let result = filter.execute(&db);
    assert_eq!(result, Err(SelectError::BadExpr));
}

#[test]
fn filter_expr_error_propagation() {
    let db = setup_database();

    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("age".to_string()),
            Expr::Field("name".to_string()),
        ))))
        .build();

    let result = filter.execute(&db);
    assert!(result.is_err());
    assert!(matches!(result.err().unwrap(), SelectError::ExprErr(_)));
}

#[test]
fn projection_expr_success() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new(
        "test_table".to_string(),
        Projection::Expr(vec![Expr::Field("age".to_owned())]),
    )
    .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
        Expr::Field("name".to_string()),
        Expr::Literal(LiteralValue::Text("Bob".to_string())),
    ))))
    .build();

    let result = filter.execute(&db);
    assert!(result.is_ok());
    let indices = result.unwrap();
    assert_eq!(indices, vec![vec![scalar!(Int(25))],]);
}

#[test]
fn projection_expr_error_propagation() {
    let db = setup_database();

    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .filter_expr(Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("age".to_string()),
            Expr::Field("name".to_string()),
        ))))
        .build();

    let result = filter.execute(&db);
    assert!(result.is_err());
    assert!(matches!(result.err().unwrap(), SelectError::ExprErr(_)));
}

#[test]
fn limit_applied_success() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .take(1)
        .build();

    let result = filter.execute(&db);
    assert!(result.is_ok());
    let indices = result.unwrap();
    assert_eq!(
        indices,
        vec![vec![
            scalar!(Int(30)),
            scalar!(Text("Alice")),
            scalar!(Bool(true)),
        ],]
    );
}

#[test]
fn skip_applied_success() {
    let db = setup_database();
    let filter = SelectQueryBuilder::new("test_table".to_string(), Projection::Row)
        .skip(1)
        .build();

    let result = filter.execute(&db);
    assert!(result.is_ok());
    let indices = result.unwrap();
    assert_eq!(
        indices,
        vec![vec![
            scalar!(Int(25)),
            scalar!(Text("Bob")),
            scalar!(Bool(false)),
        ]]
    );
}
