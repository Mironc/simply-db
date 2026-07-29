use std::collections::HashMap;

use query::expr::{ArithmeticOp, ComparisonOp, Expr, LiteralValue, LogicOp};

use crate::common::{
    ExpectExprErr, ParseError, Parser, parse_bool_null_literal, parse_field_name,
    parse_number_literal,
};

use crate::lexer::{Delimiter, Keyword, Sign, TokenValue};

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
pub fn parse_expr<'a, 'b>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let expr = parse_or(parser, end)?;
    if parser.position() != end {
        match parser.peek_next() {
            TokenValue::Ident(_)
            | TokenValue::TextLiteral(_)
            | TokenValue::Keyword(_)
            | TokenValue::Delimiter(Delimiter::RoundOpen) => {
                if parser.position() != end - 1 {
                    return Err(ParseError::UnexpectedSymbol {
                        expected: "operator or end of expression",
                        given: parser.peek_next().as_str(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(expr)
}

pub fn parse_or<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_and(parser, end)?;

    while parser.lexer().position() < end {
        if let TokenValue::Keyword(k) = parser.peek_next() {
            if k == Keyword::Or {
                parser.advance()?; // Skip OR keyword
                let right = parse_and(parser, end)?;
                left = Expr::Logical(Box::new(LogicOp::Or(left, right)));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_and<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_not(parser, end)?;

    while parser.lexer().position() < end {
        if let TokenValue::Keyword(k) = parser.peek_next() {
            if k == Keyword::And {
                parser.advance()?; // Skip AND keyword
                let right = parse_not(parser, end)?;
                left = Expr::Logical(Box::new(LogicOp::And(left, right)));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

pub fn parse_not<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    if let TokenValue::Keyword(k) = parser.peek_next() {
        if k == Keyword::Not {
            parser.advance()?; // Skip NOT keyword
            let right = parse_not(parser, end)?;
            return Ok(Expr::Logical(Box::new(LogicOp::Not(right))));
        }
    }
    parse_comparison(parser, end)
}

pub fn parse_comparison<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let left = parse_add_sub(parser, end)?;

    if parser.lexer().position() < end {
        if let TokenValue::Sign(s) = parser.peek_next() {
            if matches!(
                s,
                Sign::Less | Sign::LessEq | Sign::Greater | Sign::GreaterEq | Sign::Eq | Sign::Neq
            ) {
                parser.advance()?; // Skip sign
                let right = parse_add_sub(parser, end)?;
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

pub fn parse_add_sub<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_mul_div_mod(parser, end)?;

    while parser.lexer().position() < end {
        if let TokenValue::Sign(s) = parser.peek_next() {
            if matches!(s, Sign::Plus | Sign::Minus) {
                parser.advance()?; // Skip sign
                let right = parse_mul_div_mod(parser, end).map_err(|e| {
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

pub fn parse_mul_div_mod<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    let mut left = parse_primary(parser, end)?;

    while parser.lexer().position() < end {
        if let TokenValue::Sign(s) = parser.peek_next() {
            if matches!(s, Sign::Asterisk | Sign::Slash | Sign::Percent) {
                parser.advance()?; // Skip sign
                let right = parse_primary(parser, end).map_err(|e| {
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
pub fn parse_primary<'a>(parser: &mut Parser<'a>, end: usize) -> ExprParseResult<'a, Expr> {
    if let TokenValue::Delimiter(Delimiter::RoundOpen) = parser.peek_next() {
        parser.advance()?; // Skip parenthesis
        let expr = parse_or(parser, end)?;
        let closing = parser.consume()?; // Consume closing parenth
        if closing != TokenValue::Delimiter(Delimiter::RoundClose) {
            return Err(ParseError::UnclosedBracket(')'));
        }
        return Ok(expr);
    }

    parse_literal_or_field(parser, end)
}

/// I guess it's pretty self-explanatory
pub fn parse_literal_or_field<'a>(parser: &mut Parser<'a>, _: usize) -> ExprParseResult<'a, Expr> {
    let token = parser.peek_next();
    match token {
        TokenValue::Ident(ident) => {
            if ident.chars().all(char::is_numeric) {
                return Ok(Expr::Literal(
                    LiteralValue::from_value(parse_number_literal(parser)?)
                        .expect("Got non-number output from parse_number_literal() function"),
                ));
            }
            Ok(Expr::Field(parse_field_access(parser)?))
        }
        TokenValue::Delimiter(delimiter) => match delimiter {
            _ => Err(ParseError::UnexpectedSymbol {
                expected: "literal or field access",
                given: token.as_str(),
            }),
        },
        TokenValue::Sign(sign) => match sign {
            Sign::Minus => Ok(Expr::Literal(
                LiteralValue::from_value(parse_number_literal(parser)?)
                    .expect("Got non-number output from parse_number_literal() function"),
            )),
            s => match s {
                Sign::Plus | Sign::Asterisk | Sign::Slash | Sign::Percent => {
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
            if keyword == Keyword::And || keyword == Keyword::Or {
                return Err(ParseError::ExpectedExpr(ExpectExprErr::Before {
                    symbol: token.as_str(),
                }));
            }
            Ok(Expr::Literal(
                LiteralValue::from_value(parse_bool_null_literal(parser)?).expect(
                    "Got non-bool, non-null output from parse_bool_null_literal() function",
                ),
            ))
        }
        TokenValue::TextLiteral(text) => {
            let lit_value = LiteralValue::Text((*text).to_owned());
            parser.advance()?; // Consume literal
            Ok(Expr::Literal(lit_value))
        }
        TokenValue::EOF => Err(ParseError::UnexpectedEof),
        _ => Err(ParseError::UnexpectedSymbol {
            expected: "literal or field access",
            given: token.as_str(),
        }),
    }
}

#[inline]
pub fn parse_field_access<'a>(walker: &mut Parser<'a>) -> ExprParseResult<'a, String> {
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
                &mut simply_db::sql::parser::common::Parser::new(&tokens),
                tokens.len(),
            )
        }};
    }
}
