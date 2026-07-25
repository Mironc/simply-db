use std::collections::HashMap;

use query::expr::{ArithmeticOp, ComparisonOp, Expr, LiteralValue, LogicOp};

use crate::common::{
    ExpectExprErr, ParseError, TokenWalker, parse_bool_null_literal, parse_field_name,
    parse_number_literal,
};

use crate::tokenizer::{Delimiter, Sign, TokenValue};

/// This is for future optimization
pub type Prefix = HashMap<TokenValue<'static>, Vec<usize>>;
pub type Cache<'a> = HashMap<(usize, usize), ParseError<'a>, foldhash::fast::FixedState>;
pub type ExprParseResult<'a, E> = Result<E, ParseError<'a>>;
// impl From<&'static ParseError<'static>> for ExprParseResult<Expr> {
//     fn from(value: &'static ParseError<'static>) -> Self {
//         Err(Cow::Borrowed(value))
//     }
// }
/// Entry function for expression parsing.
pub fn parse_expr<'a, 'b>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    let expr = parse_or(walker, end)?;
    if let Some(next_token) = walker.peek_next() {
        match next_token {
            TokenValue::Ident(_)
            | TokenValue::TextLiteral(_)
            | TokenValue::Keyword(_)
            | TokenValue::Delimiter(Delimiter::RoundOpen) => {
                if walker.position() != end - 1 {
                    return Err(ParseError::UnexpectedSymbol {
                        expected: "operator or end of expression",
                        given: next_token.as_str(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(expr)
}

pub fn parse_or<'a>(walker: &mut TokenWalker<'a, '_>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_and(walker, end)?;

    while walker.position() < end {
        if let Some(TokenValue::Keyword(k)) = walker.peek_next() {
            if k == &"OR" {
                walker.next(); // skip OR keyword
                let right = parse_and(walker, end)?;
                left = Expr::Logical(Box::new(LogicOp::Or(left, right)));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_and<'a>(walker: &mut TokenWalker<'a, '_>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_not(walker, end)?;

    while walker.position() < end {
        if let Some(TokenValue::Keyword(k)) = walker.peek_next() {
            if k == &"AND" {
                walker.next();
                let right = parse_not(walker, end)?;
                left = Expr::Logical(Box::new(LogicOp::And(left, right)));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_not<'a>(walker: &mut TokenWalker<'a, '_>, end: usize) -> ExprParseResult<'a, Expr> {
    if let Some(TokenValue::Keyword(k)) = walker.peek_next() {
        if k == &"NOT" {
            walker.next();
            let right = parse_not(walker, end)?;
            return Ok(Expr::Logical(Box::new(LogicOp::Not(right))));
        }
    }
    parse_comparison(walker, end)
}

pub fn parse_comparison<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    let left = parse_add_sub(walker, end)?;

    if walker.position() < end {
        if let Some(TokenValue::Sign(s)) = walker.peek_next() {
            let s = *s;

            if matches!(
                s,
                Sign::Less | Sign::LessEq | Sign::Greater | Sign::GreaterEq | Sign::Eq | Sign::Neq
            ) {
                walker.next();
                let right = parse_add_sub(walker, end)?;
                return Ok(Expr::Comparison(Box::new(match s {
                    Sign::Less => ComparisonOp::Less(left, right),
                    Sign::LessEq => ComparisonOp::LessEq(left, right),
                    Sign::Greater => ComparisonOp::Greater(left, right),
                    Sign::GreaterEq => ComparisonOp::GreaterEq(left, right),
                    Sign::Eq => ComparisonOp::Eq(left, right),
                    Sign::Neq => ComparisonOp::NotEq(left, right),
                    _ => unreachable!(),
                })));
            }
        }
    }
    Ok(left)
}

pub fn parse_add_sub<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    let mut left = parse_mul_div_mod(walker, end)?;

    while walker.position() < end {
        if let Some(TokenValue::Sign(s)) = walker.peek_next() {
            let s = *s;
            if matches!(s, Sign::Plus | Sign::Minus) {
                walker.next();
                let right = parse_mul_div_mod(walker, end).map_err(|e| {
                    if let ParseError::UnexpectedEof = e {
                        ParseError::ExpectedExpr(ExpectExprErr::After { symbol: s.as_str() })
                    } else {
                        e
                    }
                })?;
                left = Expr::Arithmetic(Box::new(match s {
                    Sign::Plus => ArithmeticOp::Add(left, right),
                    Sign::Minus => ArithmeticOp::Subtract(left, right),
                    _ => unreachable!(),
                }));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_mul_div_mod<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    let mut left = parse_primary(walker, end)?;

    while walker.position() < end {
        if let Some(TokenValue::Sign(s)) = walker.peek_next() {
            let s = *s;
            if matches!(s, Sign::Asterisk | Sign::Slash | Sign::Percent) {
                walker.next();
                let right = parse_primary(walker, end).map_err(|e| {
                    if let ParseError::UnexpectedEof = e {
                        ParseError::ExpectedExpr(ExpectExprErr::After { symbol: s.as_str() })
                    } else {
                        e
                    }
                })?;
                left = Expr::Arithmetic(Box::new(match s {
                    Sign::Asterisk => ArithmeticOp::Multiply(left, right),
                    Sign::Slash => ArithmeticOp::Divide(left, right),
                    Sign::Percent => ArithmeticOp::Modulo(left, right),
                    _ => unreachable!(),
                }));
                continue;
            }
        }
        break;
    }
    Ok(left)
}
pub fn parse_primary<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    if let Some(TokenValue::Delimiter(Delimiter::RoundOpen)) = walker.peek_next() {
        walker.next();
        let expr = parse_or(walker, end)?;
        let closing = walker.next().ok_or(ParseError::UnclosedBracket(')'))?;
        if closing != &TokenValue::Delimiter(Delimiter::RoundClose) {
            return Err(ParseError::UnclosedBracket(')'));
        }
        return Ok(expr);
    }

    parse_literal_or_field(walker, end)
}

/// I guess it's pretty self-explanatory
pub fn parse_literal_or_field<'a>(
    walker: &mut TokenWalker<'a, '_>,
    _: usize,
) -> ExprParseResult<'a, Expr> {
    let token = walker.peek_next().ok_or(ParseError::UnexpectedEof)?;
    match token {
        TokenValue::Ident(ident) => {
            if ident.chars().all(char::is_numeric) {
                return Ok(Expr::Literal(
                    LiteralValue::from_value(parse_number_literal(walker)?)
                        .expect("Got non-number output from parse_number_literal() function"),
                ));
            }
            Ok(Expr::Field(parse_field_access(walker)?))
        }
        TokenValue::Delimiter(delimiter) => match delimiter {
            _ => Err(ParseError::UnexpectedSymbol {
                expected: "literal or field access",
                given: token.as_str(),
            }),
        },
        TokenValue::Sign(sign) => match sign {
            Sign::Minus => Ok(Expr::Literal(
                LiteralValue::from_value(parse_number_literal(walker)?)
                    .expect("Got non-number output from parse_number_literal() function"),
            )),
            s => match s {
                Sign::Plus | Sign::Minus | Sign::Asterisk | Sign::Slash | Sign::Percent => {
                    Err(ParseError::ExpectedExpr(ExpectExprErr::Before {
                        symbol: token.as_str(),
                    }))
                }
                _ => Err(ParseError::UnexpectedSymbol {
                    expected: "literal or field access",
                    given: token.as_str(),
                }),
            },
        },
        TokenValue::Keyword(keyword) => {
            if *keyword == "AND" || *keyword == "OR" {
                return Err(ParseError::ExpectedExpr(ExpectExprErr::Before {
                    symbol: token.as_str(),
                }));
            }
            Ok(Expr::Literal(
                LiteralValue::from_value(parse_bool_null_literal(walker)?).expect(
                    "Got non-bool, non-null output from parse_bool_null_literal() function",
                ),
            ))
        }
        TokenValue::TextLiteral(text) => {
            let lit_value = LiteralValue::Text((*text).to_owned());
            walker.skip(1);
            Ok(Expr::Literal(lit_value))
        }
        _ => Err(ParseError::UnexpectedSymbol {
            expected: "literal or field access",
            given: token.as_str(),
        }),
    }
}

#[inline]
pub fn parse_field_access<'a>(walker: &mut TokenWalker<'a, '_>) -> ExprParseResult<'a, String> {
    // Might add some logic
    parse_field_name(walker)
}

#[cfg(test)]
pub mod test_util {
    // Helper macro to parse a string cleanly in tests
    #[macro_export]
    macro_rules! parse_expr {
        ($input:expr) => {{
            let mut tokens = simply_db::sql::parser::tokenizer::tokenize($input);
            tokens.push(simply_db::sql::parser::tokenizer::TokenValue::Blank);
            simply_db::sql::parser::expr::parse_expr(
                &mut simply_db::sql::parser::common::TokenWalker::new(&tokens),
                tokens.len(),
            )
        }};
    }
}
