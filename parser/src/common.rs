use storage::{
    common_types::{DataValue, ScalarValue},
    scalar,
};

use crate::lexer::{Delimiter, Keyword, Lexer, Sign, TokenValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectExprErr<'a> {
    Before { symbol: &'a str },
    After { symbol: &'a str },
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
    UnexpectedValue {
        expected: &'a str,
    },
    /// Unexpected end of file
    UnexpectedEof,
    /// Unexpected start of file
    UnexpectedSof,
    UnknownDataType,
    Other {
        message: &'a str,
    },
    UnsupportedCharacter {
        character: char,
    },
    IdentStartsWithNumber,
}
pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: TokenValue<'a>,
    next: TokenValue<'a>,
    current_position: usize,
}
impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> ParseResult<'a, Self> {
        let mut parser = Self {
            lexer,
            current: TokenValue::SOF,
            next: TokenValue::SOF,
            current_position: 0,
        };
        parser.advance()?;
        Ok(parser)
    }
    pub fn position(&self) -> usize {
        self.current_position
    }
    #[inline]
    pub fn advance(&mut self) -> ParseResult<'a, ()> {
        self.current = self.next;
        self.current_position = self.lexer.position();
        self.next = self.lexer.next_token()?;
        Ok(())
    }

    /// Goes to the next token and compares it to the `expect_token`
    ///
    /// # Errors:
    /// - ParseError::UnexpectedEof, if there's no tokens left
    /// - ParseError::UnexpectedSymbol, if token doesn't match `expect_token`
    #[inline]
    pub fn expect_next_token(&mut self, expected: TokenValue<'a>) -> ParseResult<'a, ()> {
        let token = self.consume()?;
        if token == expected {
            Ok(())
        } else {
            if self.current == TokenValue::EOF {
                return Err(ParseError::UnexpectedEof);
            }
            Err(ParseError::UnexpectedSymbol {
                expected: expected.as_str(),
                given: self.current.as_str(),
            })
        }
    }

    #[inline]
    pub fn consume(&mut self) -> ParseResult<'a, TokenValue<'a>> {
        self.advance()?;
        Ok(self.current)
    }

    pub fn current_token(&self) -> TokenValue<'a> {
        self.current
    }

    pub fn peek_next(&self) -> TokenValue<'a> {
        self.next
    }

    pub fn lexer(&self) -> Lexer<'a> {
        self.lexer
    }
}
/// Parses literals including numbers, strings, nulls and bools.
pub fn parse_literal<'a>(walker: &mut Parser<'a>) -> ParseResult<'a, DataValue> {
    let token = walker.peek_next();
    match token {
        TokenValue::Ident(_) => parse_number_literal(walker),
        TokenValue::Sign(_) => parse_number_literal(walker),
        TokenValue::Delimiter(delim) => match delim {
            Delimiter::RoundOpen => todo!("Maybe structures in the future"),
            Delimiter::BlockOpen => todo!("Maybe arrays in the future"),
            Delimiter::Dot => parse_number_literal(walker),
            _ => Err(ParseError::UnknownDataType),
        },
        TokenValue::Keyword(_) => parse_bool_null_literal(walker),
        TokenValue::SOF => Err(ParseError::UnexpectedSof),
        TokenValue::EOF => Err(ParseError::UnexpectedEof),
        TokenValue::TextLiteral(value) => {
            let text = (*value).to_owned();
            walker.advance()?;
            Ok(DataValue::Scalar(ScalarValue::Text(text)))
        }
    }
}

pub fn parse_bool_null_literal<'a>(walker: &mut Parser<'a>) -> ParseResult<'a, DataValue> {
    let token = walker.consume()?;
    if let TokenValue::Keyword(k) = token {
        Ok(match k {
            Keyword::Null => DataValue::Null,
            Keyword::False => scalar!(Bool(false)),
            Keyword::True => scalar!(Bool(true)),
            _ => return Err(ParseError::UnappropriateKeyword),
        })
    } else {
        Err(ParseError::UnexpectedSymbol {
            expected: "NULL, TRUE, FALSE",
            given: token.as_str(),
        })
    }
}
/// Expects walker's pointer be beside literal symbol.
pub fn parse_number_literal<'a>(walker: &mut Parser<'a>) -> ParseResult<'a, DataValue> {
    let mut negative = false;
    let mut token = walker.consume()?;
    if let TokenValue::Sign(Sign::Minus) = token {
        negative = true;
        token = walker.consume()?;
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
    } else if let TokenValue::EOF = token {
        return Err(ParseError::Other {
            message: "Expected number literal after '-' sign",
        });
    } else {
        return Err(ParseError::UnknownDataType);
    };
    let mut partial = None;
    if let TokenValue::Delimiter(Delimiter::Dot) = walker.peek_next() {
        walker.advance()?; // Consume dot
        if let TokenValue::Ident(word) = walker.consume()? {
            match str::parse::<i32>(word) {
                Ok(n) => partial = Some(n),
                Err(_) => {
                    return Err(ParseError::UnexpectedSymbol {
                        expected: "number literal",
                        given: word,
                    });
                }
            };
        } else {
            return Err(ParseError::Other {
                message: "After dot expected fractional part of number",
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

pub fn parse_field_name<'a>(walker: &mut Parser<'a>) -> ParseResult<'a, String> {
    let token = walker.consume()?;
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

    use super::*;
    #[test]
    fn string_literal_parsing() {
        // Test with spaces and symbols
        let lexer = Lexer::new("' hello *,.)(;:<>[]}{-=+!@#$%^&№@'");
        println!("{:?}", lexer);
        let mut walker = Parser::new(lexer).unwrap();

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
        let lexer = Lexer::new(" 123");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Int(123))));

        // Test float parsing
        let lexer = Lexer::new("123.45");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Float(123.45))));

        // Test negative integer
        let lexer = Lexer::new(" -13");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Int(-13))));

        // Test negative float
        let lexer = Lexer::new("-31.75");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Float(-31.75))));
    }
    #[test]
    fn null_literal_parsing() {
        let lexer = Lexer::new(" NULL ");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(DataValue::Null));
    }
    #[test]
    fn bool_literal_parsing() {
        let lexer = Lexer::new(" TRUE ");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Bool(true))));

        let lexer = Lexer::new(" FALSE ");
        let mut walker = Parser::new(lexer).unwrap();
        let result = parse_literal(&mut walker);
        assert_eq!(result, Ok(scalar!(Bool(false))));
    }

    #[test]
    fn bad_number_literal_parsing() {
        // Test negative sign without digits
        let lexer = Lexer::new("-");
        let mut walker = Parser::new(lexer).unwrap();
        println!("{:?}", walker.peek_next());
        let result = parse_literal(&mut walker);
        assert_eq!(
            result,
            Err(ParseError::Other {
                message: "Expected number literal after '-' sign"
            })
        );

        // Test decimal point without integer part
        let lexer = Lexer::new(".2123");
        let mut walker = Parser::new(lexer).unwrap();
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
        let lexer = Lexer::new("123.45.67");
        let mut walker = Parser::new(lexer).unwrap();
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
