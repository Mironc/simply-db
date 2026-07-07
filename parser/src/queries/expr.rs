use std::collections::HashMap;

use query::expr::{ArithmeticOp, ComparisonOp, Expr, LiteralValue, LogicOp};

use crate::common::{
    ExpectExprErr, ParseError, ParseResult, TokenWalker, parse_bool_null_literal, parse_field_name,
    parse_number_literal, parse_string_literal,
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
///
/// Cleans input to exclude excessive non-meaningful tokens.
/// Creates memoization table.
pub fn parse_expr<'a, 'b>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
) -> ExprParseResult<'a, Expr> {
    let mut memo = Cache::default();
    internal_parse_expr(walker, end, &mut memo)
}
/// From low priority to high priority
const ORDER: [for<'a> fn(
    &mut TokenWalker<'a, '_>,
    usize,
    &mut Cache<'a>,
) -> ParseResult<'a, Expr>; 16] = [
    parse_or,
    parse_and,
    parse_not,
    parse_cmp_less,
    parse_cmp_less_eq,
    parse_cmp_greater,
    parse_cmp_greater_eq,
    parse_cmp_eq,
    parse_cmp_neq,
    parse_add,
    parse_sub,
    parse_mul,
    parse_div,
    parse_mod,
    parse_brackets,
    parse_literal_or_field,
];
/// Internal backtrack-recursive parsing function
pub fn internal_parse_expr<'a>(
    walker: &mut TokenWalker<'a, '_>,
    mut end: usize,
    memo: &mut Cache<'a>,
) -> ExprParseResult<'a, Expr> {
    let tokens = walker.tokens();
    while end != 0 {
        if tokens[end - 1] != TokenValue::Blank {
            break;
        }
        end -= 1;
    }
    //println!("{:?}", &tokens[walker.position()..end]);
    let start = walker.position();
    if let Some(err) = memo.get(&(start, end)) {
        return Err(err.clone());
    }
    let res = try_parse_multiple(walker, end, memo, &ORDER);
    if let Err(err) = &res {
        memo.insert((start, end), err.clone());
    }
    res
}
/// Main function in expression parsing.
/// Applies different patterns and if there's at least one success returns it
pub fn try_parse_multiple<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
    memo: &mut Cache<'a>,
    parse: &[impl Fn(&mut TokenWalker<'a, '_>, usize, &mut Cache<'a>) -> ExprParseResult<'a, Expr>],
) -> ExprParseResult<'a, Expr> {
    let mut most_meaningful_err = None;
    let mut max_consumed = 0;
    for p_fn in parse.iter() {
        let mut walker_copy = walker.clone_simple();
        match p_fn(&mut walker_copy, end, memo) {
            Ok(expr) => {
                // Ensures that all tokens were taken
                if walker_copy.position() == end - 1 {
                    walker.set_position(walker_copy.position());
                    return Ok(expr);
                }
            }
            Err(e) => {
                let current_pos = walker_copy.position();
                if current_pos > max_consumed
                    && e != ParseError::WrongPattern
                    && e != ParseError::UnexpectedEof
                {
                    max_consumed = current_pos;
                    most_meaningful_err = Some(e);
                }
            }
        }
    }
    if let Some(e) = most_meaningful_err {
        return Err(e);
    }
    Err(ParseError::UnknownPattern)
}
///
pub fn parse_brackets<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
    memo: &mut Cache<'a>,
) -> ExprParseResult<'a, Expr> {
    if end - walker.position() < 3 {
        return Err(ParseError::WrongPattern);
    }
    let mut open = 0;
    let mut closed = 0;
    // Initially points to SOF
    let mut first_bracket = 0;
    while walker.position() < end {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if token == &TokenValue::Delimiter(Delimiter::RoundOpen) {
            let pos = walker.position();
            if pos >= end {
                continue;
            }
            let pos = walker.position();
            open += 1;
            if first_bracket == 0 {
                first_bracket = pos;
            }
        } else if token == &TokenValue::Delimiter(Delimiter::RoundClose) {
            let pos = walker.position();
            if pos >= end {
                continue;
            }
            if open == 0 {
                return Err(ParseError::UnclosedBracket(')'));
            }
            open -= 1;
            closed += 1;
            if open == 0 && closed > 0 {
                break;
            }
        }
    }
    // All brackets are closed and atleast one pair of bracket was seen
    if open == 0 && closed != 0 {
        let mut expr = walker.clone_with_pos(first_bracket);
        internal_parse_expr(&mut expr, walker.position(), memo)
    } else {
        if open != 0 {
            Err(ParseError::UnclosedBracket('('))
        } else {
            Err(ParseError::WrongPattern)
        }
    }
}

pub fn parse_not<'a>(
    walker: &mut TokenWalker<'a, '_>,
    end: usize,
    memo: &mut Cache<'a>,
) -> ExprParseResult<'a, Expr> {
    // Comparing with 3 because it look like this (NOT, blank, expr)
    if end - walker.position() < 3 {
        return Err(ParseError::WrongPattern);
    }
    while walker.position() < end {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if token == &TokenValue::Keyword("NOT".into()) {
            let pos = walker.position();
            if pos >= end {
                continue;
            }
            if let Ok(expr) = internal_parse_expr(walker, end, memo) {
                return Ok(Expr::Logical(Box::new(LogicOp::Not(expr))));
            }
        }
    }
    Err(ParseError::WrongPattern)
}
//316 lines into just 50, how neat
macro_rules! parse_binary {
    ($fn_name:ident,$expr_enum_variant:ident,$construct_variant:expr,$match_token:expr) => {
        #[doc = concat!("Finds every entry of `", stringify!($match_token), "` and tries to parse until success.")]
        ///
        /// If goes to end and nothing succeeded returns `Err`
        ///
        /// Doc was created automatically.
        pub fn $fn_name<'a>(
            walker: &mut TokenWalker<'a,'_>,
            end: usize,
            memo: &mut Cache<'a>,
        ) -> ExprParseResult<'a, Expr> {
            let start = walker.position();
            // comparing with 3 because it looks like this (Expr, Matching Symb, Expr)
            if end - start < 3{
                return Err(ParseError::WrongPattern)
            }
            let mut most_meaningful_err = None;
            let mut max_consumed = start;
            while walker.position() < end {
                let token = if let Some(token) = walker.next_meaningful(){
                    token
                }
                else{
                    break;
                };
                if token == &$match_token {
                    let pos = walker.position();
                    if pos >= end {
                        continue;
                    }
                    let mut lhs = walker.clone_with_pos(start);
                    let mut rhs = walker.clone_simple();
                    let lhs_expr = internal_parse_expr(&mut lhs, walker.position(), memo);
                    let rhs_expr = internal_parse_expr(&mut rhs, end, memo);
                    walker.set_position(rhs.position());
                    let current_pos = walker.position();
                    match (lhs_expr, rhs_expr){
                        (Ok(lhs), Ok(rhs)) =>{
                            return Ok(Expr::$expr_enum_variant(Box::new($construct_variant(
                                lhs, rhs,
                            ))));
                        }
                        (Err(_), Err(_)) => {
                            if current_pos > max_consumed {
                                max_consumed = current_pos;
                                most_meaningful_err = Some(ParseError::ExpectedExpr(ExpectExprErr::BeforeAfter{symbol:$match_token.as_str()}));
                            }
                        }
                        (Err(_), _) => {
                            if current_pos > max_consumed  {
                                max_consumed = current_pos;
                                most_meaningful_err = Some(ParseError::ExpectedExpr(ExpectExprErr::Before{symbol:$match_token.as_str()}));
                            }
                        }
                        (_, Err(_)) => {
                            if current_pos > max_consumed  {
                                max_consumed = current_pos;
                                most_meaningful_err = Some(ParseError::ExpectedExpr(ExpectExprErr::After{symbol:$match_token.as_str()}));
                            }
                        }
                    }

                }
            }
            if let Some(e) = most_meaningful_err {
                return Err(e);
            }
            Err(ParseError::WrongPattern)
        }
    };
}

parse_binary!(
    parse_add,
    Arithmetic,
    ArithmeticOp::Add,
    TokenValue::Sign(Sign::Plus)
);
parse_binary!(
    parse_sub,
    Arithmetic,
    ArithmeticOp::Subtract,
    TokenValue::Sign(Sign::Minus)
);
parse_binary!(
    parse_div,
    Arithmetic,
    ArithmeticOp::Divide,
    TokenValue::Sign(Sign::Slash)
);
parse_binary!(
    parse_mul,
    Arithmetic,
    ArithmeticOp::Multiply,
    TokenValue::Sign(Sign::Asterisk)
);
parse_binary!(
    parse_mod,
    Arithmetic,
    ArithmeticOp::Modulo,
    TokenValue::Sign(Sign::Percent)
);
parse_binary!(
    parse_cmp_less,
    Comparison,
    ComparisonOp::Less,
    TokenValue::Sign(Sign::Less)
);
parse_binary!(
    parse_cmp_less_eq,
    Comparison,
    ComparisonOp::LessEq,
    TokenValue::Sign(Sign::LessEq)
);
parse_binary!(
    parse_cmp_greater,
    Comparison,
    ComparisonOp::Greater,
    TokenValue::Sign(Sign::Greater)
);
parse_binary!(
    parse_cmp_greater_eq,
    Comparison,
    ComparisonOp::GreaterEq,
    TokenValue::Sign(Sign::GreaterEq)
);
parse_binary!(
    parse_cmp_eq,
    Comparison,
    ComparisonOp::Eq,
    TokenValue::Sign(Sign::Eq)
);
parse_binary!(
    parse_cmp_neq,
    Comparison,
    ComparisonOp::NotEq,
    TokenValue::Sign(Sign::Neq)
);
parse_binary!(
    parse_and,
    Logical,
    LogicOp::And,
    TokenValue::Keyword("AND".into())
);
parse_binary!(
    parse_or,
    Logical,
    LogicOp::Or,
    TokenValue::Keyword("OR".into())
);

/// I guess it's pretty self-explanatory
pub fn parse_literal_or_field<'a>(
    walker: &mut TokenWalker<'a, '_>,
    _: usize,
    _: &mut Cache<'a>,
) -> ExprParseResult<'a, Expr> {
    let token = walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnexpectedEof)?;
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
            Delimiter::Apostrophe => Ok(Expr::Literal(
                LiteralValue::from_value(parse_string_literal(walker)?)
                    .expect("Got non-string output from parse_string_literal() function"),
            )),
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
            _ => Err(ParseError::UnexpectedSymbol {
                expected: "literal or field access",
                given: token.as_str(),
            }),
        },
        TokenValue::Keyword(_) => Ok(Expr::Literal(
            LiteralValue::from_value(parse_bool_null_literal(walker)?)
                .expect("Got non-bool, non-null output from parse_bool_null_literal() function"),
        )),
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
