use std::{borrow::Cow, ops::Deref};

use storage::{
    common_types::{DataValue, ScalarValue},
    row::Row,
};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// Field not found,
    UnknownField { field: String },
    /// Operator encounter wrong data type for some operation
    NotApplicable,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Text(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Null,
}

impl LiteralValue {
    pub fn value(&self) -> DataValue {
        match self {
            LiteralValue::Text(t) => DataValue::Scalar(ScalarValue::Text(t.clone())),
            LiteralValue::Int(i) => DataValue::Scalar(ScalarValue::Int(*i)),
            LiteralValue::Float(f) => DataValue::Scalar(ScalarValue::Float(*f)),
            LiteralValue::Bool(b) => DataValue::Scalar(ScalarValue::Bool(*b)),
            LiteralValue::Null => DataValue::Null,
        }
    }
    pub fn from_value(value: DataValue) -> Option<Self> {
        match value {
            DataValue::Scalar(scalar_value) => Some(match scalar_value {
                ScalarValue::Int(i) => Self::Int(i),
                ScalarValue::Float(f) => Self::Float(f),
                ScalarValue::Bool(b) => Self::Bool(b),
                ScalarValue::Text(t) => Self::Text(t),
            }),
            DataValue::Struct(_) => None,
            DataValue::Null => Some(Self::Null),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum LogicOp {
    And(Expr, Expr),
    Or(Expr, Expr),
    Not(Expr),
}
impl LogicOp {
    pub fn execute<'a>(&'a self, row: &Row) -> Result<Cow<'a, DataValue>, ExprError> {
        match self {
            LogicOp::And(op, op1) => {
                let left = op.execute(row)?;
                let right = op1.execute(row)?;
                if let (
                    DataValue::Scalar(ScalarValue::Bool(l)),
                    DataValue::Scalar(ScalarValue::Bool(r)),
                ) = (left.deref(), right.deref())
                {
                    let value = DataValue::Scalar(ScalarValue::Bool(*l && *r));
                    Ok(Cow::Owned(value))
                } else {
                    Err(ExprError::NotApplicable)
                }
            }
            LogicOp::Or(op, op1) => {
                let left = op.execute(row)?;
                let right = op1.execute(row)?;
                if let (
                    DataValue::Scalar(ScalarValue::Bool(l)),
                    DataValue::Scalar(ScalarValue::Bool(r)),
                ) = (left.deref(), right.deref())
                {
                    let value = DataValue::Scalar(ScalarValue::Bool(*l || *r));
                    Ok(Cow::Owned(value))
                } else {
                    Err(ExprError::NotApplicable)
                }
            }
            LogicOp::Not(op) => {
                let val = op.execute(row)?;
                if let DataValue::Scalar(ScalarValue::Bool(val)) = val.deref() {
                    let value = DataValue::Scalar(ScalarValue::Bool(!val));
                    return Ok(Cow::Owned(value));
                }
                Err(ExprError::NotApplicable)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Less(Expr, Expr),
    LessEq(Expr, Expr),
    Greater(Expr, Expr),
    GreaterEq(Expr, Expr),
    Eq(Expr, Expr),
    NotEq(Expr, Expr),
}
impl ComparisonOp {
    pub fn execute<'a>(&'a self, row: &'a Row) -> Result<Cow<'a, DataValue>, ExprError> {
        let (left, right) = self.exprs();
        let left = left.execute(row)?;
        let right = right.execute(row)?;
        if let (DataValue::Scalar(ScalarValue::Bool(l)), DataValue::Scalar(ScalarValue::Bool(r))) =
            (left.deref(), right.deref())
        {
            let greater = l > r;
            let less = l < r;
            let eq = l == r;
            return Ok(Cow::Owned(DataValue::Scalar(ScalarValue::Bool(
                match self {
                    ComparisonOp::Eq(_, _) => eq,
                    ComparisonOp::NotEq(_, _) => !eq,
                    ComparisonOp::Less(_, _) => less,
                    ComparisonOp::LessEq(_, _) => !greater,
                    ComparisonOp::Greater(_, _) => greater,
                    ComparisonOp::GreaterEq(_, _) => !less,
                },
            ))));
        }
        if let (DataValue::Scalar(ScalarValue::Text(l)), DataValue::Scalar(ScalarValue::Text(r))) =
            (left.deref(), right.deref())
        {
            let greater = l > r;
            let less = l < r;
            let eq = l == r;
            return Ok(Cow::Owned(DataValue::Scalar(ScalarValue::Bool(
                match self {
                    ComparisonOp::Eq(_, _) => eq,
                    ComparisonOp::NotEq(_, _) => !eq,
                    ComparisonOp::Less(_, _) => less,
                    ComparisonOp::LessEq(_, _) => !greater,
                    ComparisonOp::Greater(_, _) => greater,
                    ComparisonOp::GreaterEq(_, _) => !less,
                },
            ))));
        }
        if let (DataValue::Scalar(ScalarValue::Int(l)), DataValue::Scalar(ScalarValue::Int(r))) =
            (left.deref(), right.deref())
        {
            let greater = l > r;
            let less = l < r;
            let eq = l == r;
            return Ok(Cow::Owned(DataValue::Scalar(ScalarValue::Bool(
                match self {
                    ComparisonOp::Eq(_, _) => eq,
                    ComparisonOp::NotEq(_, _) => !eq,
                    ComparisonOp::Less(_, _) => less,
                    ComparisonOp::LessEq(_, _) => !greater,
                    ComparisonOp::Greater(_, _) => greater,
                    ComparisonOp::GreaterEq(_, _) => !less,
                },
            ))));
        }
        if let (
            DataValue::Scalar(ScalarValue::Float(l)),
            DataValue::Scalar(ScalarValue::Float(r)),
        ) = (left.deref(), right.deref())
        {
            let greater = l > r;
            let less = l < r;
            let eq = l == r;
            return Ok(Cow::Owned(DataValue::Scalar(ScalarValue::Bool(
                match self {
                    ComparisonOp::Eq(_, _) => eq,
                    ComparisonOp::NotEq(_, _) => !eq,
                    ComparisonOp::Less(_, _) => less,
                    ComparisonOp::LessEq(_, _) => !greater,
                    ComparisonOp::Greater(_, _) => greater,
                    ComparisonOp::GreaterEq(_, _) => !less,
                },
            ))));
        }
        Err(ExprError::NotApplicable)
    }
    pub fn exprs(&self) -> (&Expr, &Expr) {
        match self {
            ComparisonOp::Less(expr, expr1)
            | ComparisonOp::LessEq(expr, expr1)
            | ComparisonOp::Greater(expr, expr1)
            | ComparisonOp::GreaterEq(expr, expr1)
            | ComparisonOp::Eq(expr, expr1)
            | ComparisonOp::NotEq(expr, expr1) => (expr, expr1),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOp {
    Add(Expr, Expr),
    Subtract(Expr, Expr),
    Multiply(Expr, Expr),
    Divide(Expr, Expr),
    Modulo(Expr, Expr),
}

impl ArithmeticOp {
    pub fn execute<'a>(&'a self, row: &'a Row) -> Result<Cow<'a, DataValue>, ExprError> {
        let (lhs, rhs) = self.exprs();
        match (lhs.execute(row)?.deref(), rhs.execute(row)?.deref()) {
            (DataValue::Struct(_), _) => Err(ExprError::NotApplicable),
            (_, DataValue::Struct(_)) => Err(ExprError::NotApplicable),
            (DataValue::Scalar(rhs), DataValue::Scalar(lhs)) => Ok(Cow::Owned(DataValue::Scalar(
                match self {
                    ArithmeticOp::Add(_, _) => ScalarValue::add(rhs, lhs),
                    ArithmeticOp::Subtract(_, _) => ScalarValue::subtract(rhs, lhs),
                    ArithmeticOp::Multiply(_, _) => ScalarValue::multiply(rhs, lhs),
                    ArithmeticOp::Divide(_, _) => ScalarValue::divide(rhs, lhs),
                    ArithmeticOp::Modulo(_, _) => ScalarValue::modulo(rhs, lhs),
                }
                .ok_or(ExprError::NotApplicable)?,
            ))),
            _ => Ok(Cow::Owned(DataValue::Null)),
        }
    }
    pub fn exprs(&self) -> (&Expr, &Expr) {
        match self {
            ArithmeticOp::Add(expr, expr1)
            | ArithmeticOp::Multiply(expr, expr1)
            | ArithmeticOp::Subtract(expr, expr1)
            | ArithmeticOp::Divide(expr, expr1)
            | ArithmeticOp::Modulo(expr, expr1) => (expr, expr1),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Field(String),
    Literal(LiteralValue),
    Logical(Box<LogicOp>),
    Comparison(Box<ComparisonOp>),
    Arithmetic(Box<ArithmeticOp>),
}
impl Expr {
    pub fn execute<'a>(&'a self, row: &'a Row) -> Result<Cow<'a, DataValue>, ExprError> {
        Ok(match self {
            Expr::Field(field_name) => {
                Cow::Borrowed(row.data().fields().get(field_name.as_str()).ok_or(
                    ExprError::UnknownField {
                        field: field_name.to_owned(),
                    },
                )?)
            }
            Expr::Logical(logical_op) => logical_op.execute(row)?,
            Expr::Comparison(comparison_op) => comparison_op.execute(row)?,
            Expr::Literal(literal_value) => Cow::Owned(literal_value.value()),
            Expr::Arithmetic(operator) => operator.execute(row)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use storage::{common_types::SchemaValue, row::Row, scalar};

    use crate::expr::{ArithmeticOp, ComparisonOp, Expr, ExprError, LiteralValue, LogicOp};

    fn empty_row() -> Row {
        Row::new(SchemaValue::new(HashMap::new()))
    }
    /// Makes tests cleaner by removing boilerplate
    macro_rules! eq_assert {
        ($lhs:expr,$rhs:expr) => {
            assert_eq!($lhs.as_deref(), Ok(&$rhs))
        };
    }

    #[test]
    fn literal_from_value() {
        assert_eq!(
            LiteralValue::from_value(scalar!(Int(5))),
            Some(LiteralValue::Int(5))
        );
        assert_eq!(
            LiteralValue::from_value(scalar!(Float(2.0))),
            Some(LiteralValue::Float(2.0))
        );
        assert_eq!(
            LiteralValue::from_value(scalar!(Text("Text"))),
            Some(LiteralValue::Text("Text".to_owned()))
        );
        assert_eq!(
            LiteralValue::from_value(scalar!(Bool(true))),
            Some(LiteralValue::Bool(true))
        );
    }

    enum ArOp {
        Add,
        Sub,
        Mod,
        Mul,
        Div,
    }
    #[rstest::rstest]
    fn int_arithmetics(
        #[values(-100, 0, 42)] lhs: i32,
        #[values(-5, 1, 20)] rhs: i32,
        #[values(ArOp::Add, ArOp::Sub, ArOp::Mod, ArOp::Mul, ArOp::Div)] op: ArOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Int(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Int(rhs));
        let expr = Expr::Arithmetic(Box::new(match op {
            ArOp::Add => ArithmeticOp::Add(literal_lhs, literal_rhs),
            ArOp::Sub => ArithmeticOp::Subtract(literal_lhs, literal_rhs),
            ArOp::Mod => ArithmeticOp::Modulo(literal_lhs, literal_rhs),
            ArOp::Mul => ArithmeticOp::Multiply(literal_lhs, literal_rhs),
            ArOp::Div => ArithmeticOp::Divide(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Int(match op {
            ArOp::Add => lhs + rhs,
            ArOp::Sub => lhs - rhs,
            ArOp::Mod => lhs % rhs,
            ArOp::Mul => lhs * rhs,
            ArOp::Div => lhs / rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
    #[rstest::rstest]
    fn float_arithmetics(
        #[values(-100.0, 0.0, 42.0)] lhs: f32,
        #[values(-5.0, 1.0, 20.0)] rhs: f32,
        #[values(ArOp::Add, ArOp::Sub, ArOp::Mod, ArOp::Mul, ArOp::Div)] op: ArOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Float(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Float(rhs));
        let expr = Expr::Arithmetic(Box::new(match op {
            ArOp::Add => ArithmeticOp::Add(literal_lhs, literal_rhs),
            ArOp::Sub => ArithmeticOp::Subtract(literal_lhs, literal_rhs),
            ArOp::Mod => ArithmeticOp::Modulo(literal_lhs, literal_rhs),
            ArOp::Mul => ArithmeticOp::Multiply(literal_lhs, literal_rhs),
            ArOp::Div => ArithmeticOp::Divide(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Float(match op {
            ArOp::Add => lhs + rhs,
            ArOp::Sub => lhs - rhs,
            ArOp::Mod => lhs % rhs,
            ArOp::Mul => lhs * rhs,
            ArOp::Div => lhs / rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
    #[rstest::rstest]
    fn text_arithmetics_notapplicable(
        #[values("Not")] lhs: String,
        #[values("Applicable")] rhs: String,
        #[values(ArOp::Add, ArOp::Sub, ArOp::Mod, ArOp::Mul, ArOp::Div)] op: ArOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Text(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Text(rhs));
        let expr = Expr::Arithmetic(Box::new(match op {
            ArOp::Add => ArithmeticOp::Add(literal_lhs, literal_rhs),
            ArOp::Sub => ArithmeticOp::Subtract(literal_lhs, literal_rhs),
            ArOp::Mod => ArithmeticOp::Modulo(literal_lhs, literal_rhs),
            ArOp::Mul => ArithmeticOp::Multiply(literal_lhs, literal_rhs),
            ArOp::Div => ArithmeticOp::Divide(literal_lhs, literal_rhs),
        }));
        assert_eq!(expr.execute(&empty_row()), Err(ExprError::NotApplicable));
    }
    enum CmpOp {
        Less,
        LessEq,
        GreaterEq,
        Greater,
        Eq,
        NotEq,
    }
    #[rstest::rstest]
    fn text_comparison(
        #[values("eq", "abc")] lhs: String, // equality and lexicographical comparison
        #[values("eq", "abd")] rhs: String,
        #[values(
            CmpOp::Eq,
            CmpOp::NotEq,
            CmpOp::Less,
            CmpOp::LessEq,
            CmpOp::Greater,
            CmpOp::GreaterEq
        )]
        op: CmpOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Text(lhs.clone()));
        let literal_rhs = Expr::Literal(LiteralValue::Text(rhs.clone()));
        let expr = Expr::Comparison(Box::new(match op {
            CmpOp::Eq => ComparisonOp::Eq(literal_lhs, literal_rhs),
            CmpOp::NotEq => ComparisonOp::NotEq(literal_lhs, literal_rhs),
            CmpOp::Less => ComparisonOp::Less(literal_lhs, literal_rhs),
            CmpOp::LessEq => ComparisonOp::LessEq(literal_lhs, literal_rhs),
            CmpOp::GreaterEq => ComparisonOp::GreaterEq(literal_lhs, literal_rhs),
            CmpOp::Greater => ComparisonOp::Greater(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Bool(match op {
            CmpOp::Eq => lhs == rhs,
            CmpOp::NotEq => lhs != rhs,
            CmpOp::Less => lhs < rhs,
            CmpOp::LessEq => lhs <= rhs,
            CmpOp::GreaterEq => lhs >= rhs,
            CmpOp::Greater => lhs > rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }

    #[rstest::rstest]
    fn int_comparison(
        #[values(-100, 0, 42)] lhs: i32,
        #[values(-5, 0, 20)] rhs: i32,
        #[values(
            CmpOp::Less,
            CmpOp::LessEq,
            CmpOp::Greater,
            CmpOp::GreaterEq,
            CmpOp::Eq,
            CmpOp::NotEq
        )]
        op: CmpOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Int(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Int(rhs));
        let expr = Expr::Comparison(Box::new(match op {
            CmpOp::Eq => ComparisonOp::Eq(literal_lhs, literal_rhs),
            CmpOp::NotEq => ComparisonOp::NotEq(literal_lhs, literal_rhs),
            CmpOp::Less => ComparisonOp::Less(literal_lhs, literal_rhs),
            CmpOp::LessEq => ComparisonOp::LessEq(literal_lhs, literal_rhs),
            CmpOp::GreaterEq => ComparisonOp::GreaterEq(literal_lhs, literal_rhs),
            CmpOp::Greater => ComparisonOp::Greater(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Bool(match op {
            CmpOp::Eq => lhs == rhs,
            CmpOp::NotEq => lhs != rhs,
            CmpOp::Less => lhs < rhs,
            CmpOp::LessEq => lhs <= rhs,
            CmpOp::GreaterEq => lhs >= rhs,
            CmpOp::Greater => lhs > rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
    #[rstest::rstest]
    fn float_comparison(
        #[values(-100.0, 0.0, 42.0)] lhs: f32,
        #[values(-5.0, 0.0, 20.0)] rhs: f32,
        #[values(
            CmpOp::Less,
            CmpOp::LessEq,
            CmpOp::Greater,
            CmpOp::GreaterEq,
            CmpOp::Eq,
            CmpOp::NotEq
        )]
        op: CmpOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Float(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Float(rhs));
        let expr = Expr::Comparison(Box::new(match op {
            CmpOp::Eq => ComparisonOp::Eq(literal_lhs, literal_rhs),
            CmpOp::NotEq => ComparisonOp::NotEq(literal_lhs, literal_rhs),
            CmpOp::Less => ComparisonOp::Less(literal_lhs, literal_rhs),
            CmpOp::LessEq => ComparisonOp::LessEq(literal_lhs, literal_rhs),
            CmpOp::GreaterEq => ComparisonOp::GreaterEq(literal_lhs, literal_rhs),
            CmpOp::Greater => ComparisonOp::Greater(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Bool(match op {
            CmpOp::Eq => lhs == rhs,
            CmpOp::NotEq => lhs != rhs,
            CmpOp::Less => lhs < rhs,
            CmpOp::LessEq => lhs <= rhs,
            CmpOp::GreaterEq => lhs >= rhs,
            CmpOp::Greater => lhs > rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
    enum LogOp {
        Or,
        And,
    }
    #[rstest::rstest]
    fn logical_operations(
        #[values(true, false)] lhs: bool,
        #[values(true, false)] rhs: bool,
        #[values(LogOp::Or, LogOp::And)] op: LogOp,
    ) {
        let literal_lhs = Expr::Literal(LiteralValue::Bool(lhs));
        let literal_rhs = Expr::Literal(LiteralValue::Bool(rhs));
        let expr = Expr::Logical(Box::new(match op {
            LogOp::Or => LogicOp::Or(literal_lhs, literal_rhs),
            LogOp::And => LogicOp::And(literal_lhs, literal_rhs),
        }));
        let cmp = scalar!(Bool(match op {
            LogOp::Or => lhs || rhs,
            LogOp::And => lhs && rhs,
        }));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
    #[test]
    fn logical_not() {
        let literal = Expr::Literal(LiteralValue::Bool(true));
        let expr = Expr::Logical(Box::new(LogicOp::Not(literal)));
        let cmp = scalar!(Bool(false));
        eq_assert!(expr.execute(&empty_row()), cmp);

        let literal = Expr::Literal(LiteralValue::Bool(false));
        let expr = Expr::Logical(Box::new(LogicOp::Not(literal)));
        let cmp = scalar!(Bool(true));
        eq_assert!(expr.execute(&empty_row()), cmp);
    }
}
