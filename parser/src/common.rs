use storage::{
    common_types::{DataValue, ScalarValue},
    scalar,
};

use crate::tokenizer::{Delimiter, Sign, TokenValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectExprErr<'a> {
    Before { symbol: &'a str },
    After { symbol: &'a str },
    BeforeAfter { symbol: &'a str },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError<'a> {
    UnknownInstruction,
    UnclosedBracket(char),
    UnappropriateKeyword,
    ExpectedExpr(ExpectExprErr<'a>),
    FieldNumberMismatch {
        expected: usize,
        provided: usize,
    },
    UnknownModifier {
        modifier: &'a str,
    },
    UnexpectedSymbol {
        expected: &'a str,
        given: &'a str,
    },
    /// Unexpected end of file
    UnexpectedEof,
    /// Unexpected start of file
    UnexpectedSof,
    UnknownDataType,
    UnknownPattern,
    WrongPattern,
    Other {
        message: &'a str,
    },
    IdentStartsWithNumber,
}
pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

#[derive(Debug, Clone)]
pub struct TokenWalker<'a, 'b> {
    tokens: &'b [TokenValue<'a>],
    position: usize,
}
impl<'a, 'b> TokenWalker<'a, 'b> {
    #[inline]
    pub fn new(tokens: &'b [TokenValue<'a>]) -> Self {
        let position = 0;
        Self { tokens, position }
    }
    /// Clones walker without cloning tokens
    #[inline]
    pub fn clone_simple(&self) -> Self {
        Self {
            tokens: self.tokens,
            position: self.position,
        }
    }
    /// Clones walker without cloning tokens and with given startpos
    #[inline]
    pub fn clone_with_pos(&self, position: usize) -> Self {
        Self {
            tokens: self.tokens,
            position,
        }
    }
    /// Gives n-th token
    pub fn peek_n(&self, n: usize) -> Option<&TokenValue<'a>> {
        self.tokens.get(self.position + n)
    }

    #[inline]
    pub fn next(&mut self) -> Option<&TokenValue<'a>> {
        self.position += 1;
        self.tokens.get(self.position)
    }

    #[inline]
    pub fn peek_next(&self) -> Option<&TokenValue<'a>> {
        self.tokens.get(self.position + 1)
    }

    #[inline]
    pub fn current_token(&self) -> Option<&TokenValue<'a>> {
        self.tokens.get(self.position)
    }

    #[inline]
    pub fn skip(&mut self, n: usize) {
        self.position += n
    }
    pub fn skip_until(&mut self, value: &TokenValue) -> Option<()> {
        while let Some(token) = self.next() {
            if token == value {
                return Some(());
            }
        }
        None
    }
    /// Goes to the next non-blank token and compares it to the `expect_token`
    ///
    /// Returns Err
    #[inline]
    pub fn expect_next_token(&mut self, expect_token: &'a TokenValue) -> ParseResult<'a, ()> {
        let token = self.next().ok_or(ParseError::UnexpectedEof)?;
        if token != expect_token {
            return Err(ParseError::UnexpectedSymbol {
                expected: expect_token.as_str(),
                given: token.as_str(),
            });
        }
        Ok(())
    }
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }
    #[inline]
    pub fn set_position(&mut self, new_position: usize) {
        self.position = new_position;
    }
    #[inline]
    pub fn tokens(&self) -> &[TokenValue<'a>] {
        &self.tokens
    }
}

pub fn parse_literal<'a>(walker: &mut TokenWalker<'a, '_>) -> ParseResult<'a, DataValue> {
    let token = walker.peek_next().ok_or(ParseError::UnexpectedEof)?;
    match token {
        TokenValue::Ident(_) => parse_number_literal(walker),
        TokenValue::Sign(_) => parse_number_literal(walker),
        TokenValue::Delimiter(delim) => match delim {
            Delimiter::CurlyOpen => todo!("IDK"),
            Delimiter::RoundOpen => todo!("Maybe structures in the future"),
            Delimiter::BlockOpen => todo!("Maybe arrays in the future"),
            Delimiter::Dot => parse_number_literal(walker),
            _ => Err(ParseError::UnknownDataType),
        },
        TokenValue::Keyword(_) => parse_bool_null_literal(walker),
        TokenValue::SOF => Err(ParseError::UnexpectedSof),
        TokenValue::TextLiteral(value) => {
            let text = (*value).to_owned();
            walker.skip(1);
            Ok(DataValue::Scalar(ScalarValue::Text(text)))
        }
    }
}
pub fn parse_bool_null_literal<'a>(walker: &mut TokenWalker<'a, '_>) -> ParseResult<'a, DataValue> {
    let token = walker.next().ok_or(ParseError::UnexpectedEof)?;
    if let TokenValue::Keyword(k) = token {
        Ok(match *k {
            "NULL" => DataValue::Null,
            "FALSE" => scalar!(Bool(false)),
            "TRUE" => scalar!(Bool(true)),
            _ => return Err(ParseError::UnappropriateKeyword),
        })
    } else {
        Err(ParseError::UnexpectedSymbol {
            expected: "NULL, TRUE, FALSE",
            given: token.as_str(),
        })
    }
}
/// Expects walker's pointer be beside literal symbol
pub fn parse_number_literal<'a>(walker: &mut TokenWalker<'a, '_>) -> ParseResult<'a, DataValue> {
    let mut negative = false;
    let mut token = walker.next().ok_or(ParseError::UnexpectedEof)?;
    if let TokenValue::Sign(Sign::Minus) = token {
        negative = true;
        token = walker.next().ok_or(ParseError::Other {
            message: "Expected number literal after '-' sign",
        })?;
    }
    let whole_part = if let TokenValue::Ident(word) = token {
        match str::parse::<i32>(word) {
            Ok(n) => n,
            Err(_) => {
                return Err(ParseError::Other {
                    message: "Expected number literal",
                });
            }
        }
    } else if let TokenValue::Delimiter(Delimiter::Dot) = token {
        return Err(ParseError::Other {
            message: "Missing whole part of a number",
        });
    } else {
        return Err(ParseError::UnknownDataType);
    };
    let mut partial = None;
    if let Some(TokenValue::Delimiter(Delimiter::Dot)) = walker.peek_next() {
        walker.skip(1);
        if let TokenValue::Ident(word) = walker.next().ok_or(ParseError::UnexpectedEof)? {
            match str::parse::<i32>(word) {
                Ok(n) => partial = Some(n),
                Err(_) => {
                    return Err(ParseError::UnexpectedSymbol {
                        expected: "number literal",
                        given: *word,
                    });
                }
            };
        } else {
            return Err(ParseError::Other {
                message: "After dot expected fractional part of number".into(),
            });
        }
    }
    if let Some(partial) = partial {
        let val = (whole_part as f32)
            + ((partial as f32) / i32::pow(10, partial.checked_ilog10().unwrap() + 1) as f32);
        Ok(scalar!(Float(if negative { -val } else { val })))
    } else {
        Ok(scalar!(Int(if negative {
            -whole_part
        } else {
            whole_part
        })))
    }
}

pub fn parse_field_name<'a>(walker: &mut TokenWalker<'a, '_>) -> ParseResult<'a, String> {
    let token = walker.next().ok_or(ParseError::UnexpectedEof)?;
    if token.is_ident() && !token.starts_with_digit() {
        Ok(token.as_str().to_owned())
    } else {
        Err(ParseError::UnexpectedSymbol {
            expected: "valid field name that starts not with digit",
            given: token.as_str(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate as parser;
    use parser::tokenizer::tokenize;

    use super::*;
    #[test]
    fn string_literal_parsing() {
        // Test with spaces and symbols
        let tokens = tokenize("' hello *,.)(;:<>[]}{-=+!@#$%^&№@'").unwrap();
        println!("{:?}", tokens);
        let mut walker = TokenWalker::new(&tokens);

        let result = parse_literal(&mut walker);
        assert!(result.is_ok());

        let data = result.unwrap();
        if let DataValue::Scalar(ScalarValue::Text(s)) = data {
            assert_eq!(s, " hello *,.)(;:<>[]}{-=+!@#$%^&№@");
        } else {
            panic!("Parsed value was not a Text scalar!");
        }
    }
    #[test]
    fn number_literal_parsing() {
        // Test integer parsing
        let token = tokenize(" 123").unwrap();
        let mut walker = TokenWalker::new(&token);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Int(123))));

        // Test float parsing
        let tokens = tokenize("123.45").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Float(123.45))));

        // Test negative integer
        let tokens = tokenize(" -13").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Int(-13))));

        // Test negative float
        let tokens = tokenize("-31.75").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Float(-31.75))));
    }
    #[test]
    fn null_literal_parsing() {
        let tokens = tokenize(" NULL ").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(DataValue::Null));
    }
    #[test]
    fn bool_literal_parsing() {
        let tokens = tokenize(" TRUE ").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Bool(true))));

        let tokens = tokenize(" FALSE ").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Bool(false))));
    }

    #[test]
    fn bad_number_literal_parsing() {
        // Test negative sign without digits
        let tokens = tokenize("-").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(
            result,
            Err(ParseError::Other {
                message: "Expected number literal after '-' sign".into()
            })
        );

        // Test decimal point without integer part
        let mut tokens = tokenize(".2123").unwrap();
        let mut walker = TokenWalker::new(&mut tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(
            result,
            Err(ParseError::Other {
                message: "Missing whole part of a number".into()
            })
        );

        // Test idk.
        // I mean that's not particularly an error on the level of literal parsing
        // That would result in an error in consequtive parsing
        let tokens = tokenize("123.45.67").unwrap();
        let mut walker = TokenWalker::new(&tokens);
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Float(123.45))));
        let result = parse_literal(&mut walker);
        assert_eq!(
            result,
            Err(ParseError::Other {
                message: "Missing whole part of a number".into()
            })
        );
    }
}
