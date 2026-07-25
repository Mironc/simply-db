use std::fmt::Display;

use crate::common::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValue<'a> {
    Ident(&'a str),
    Sign(Sign),
    Delimiter(Delimiter),
    Keyword(Keyword),
    TextLiteral(&'a str),
    /// Start of the file
    SOF,
}
impl<'a> Display for TokenValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl<'a> TokenValue<'a> {
    pub fn starts_with_digit(&self) -> bool {
        match self {
            Self::Ident(ident) => ident
                .chars()
                .nth(0)
                .expect("Ident with size of 0, while calling TokenValue::starts_with_digit()")
                .is_numeric(),
            _ => false,
        }
    }
    pub fn as_str(&self) -> &'a str {
        match *self {
            TokenValue::Ident(w) => w,
            TokenValue::Sign(sign) => sign.as_str(),
            TokenValue::Delimiter(delimiter) => delimiter.as_str(),
            TokenValue::Keyword(k) => k.as_str(),
            TokenValue::SOF => "Sof",
            TokenValue::TextLiteral(l) => l,
        }
    }
    pub fn is_ident(&self) -> bool {
        matches!(self, TokenValue::Ident(_))
    }
    pub fn is_keyword(&self) -> bool {
        matches!(self, TokenValue::Keyword(_))
    }
    pub fn is_sof(&self) -> bool {
        matches!(self, TokenValue::SOF)
    }
}

macro_rules! implement_keywords {
    ($name:ident, $(($variant:ident,$symbol:literal)),+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name{
            $(
            #[doc = concat!("Represents keyword:",stringify!($symbol))]
            $variant,
            )+
        }
        impl $name{
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($symbol => Some($name::$variant),)+
                    _ => None
                }
            }
            pub fn as_str(&self) -> &'static str{
                match self {
                    $($name::$variant => $symbol,)+
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    $($name::$variant => $symbol,)+
                };
                f.write_str(s)
            }
        }
    };
}
implement_keywords!(
    Keyword,
    (Select, "SELECT"),
    (From, "FROM"),
    (Where, "WHERE"),
    (Group, "GROUP"),
    (By, "BY"),
    (If, "IF"),
    (Order, "ORDER"),
    (Distinct, "DISTINCT"),
    (As, "AS"),
    (Limit, "LIMIT"),
    (Insert, "INSERT"),
    (Into, "INTO"),
    (Values, "VALUES"),
    (Update, "UPDATE"),
    (Set, "SET"),
    (Delete, "DELETE"),
    (Create, "CREATE"),
    (Drop, "DROP"),
    (Truncate, "TRUNCATE"),
    (Using, "USING"),
    (And, "AND"),
    (Or, "OR"),
    (Not, "NOT"),
    (In, "IN"),
    (Is, "IS"),
    (Null, "NULL"),
    (Exists, "EXISTS"),
    (Case, "CASE"),
    (When, "WHEN"),
    (Then, "THEN"),
    (Else, "ELSE"),
    (End, "END"),
    (All, "ALL"),
    (Primary, "PRIMARY"),
    (Key, "KEY"),
    (Foreign, "FOREIGN"),
    (References, "REFERENCES"),
    (Unique, "UNIQUE"),
    (AutoIncrement, "AUTOINCREMENT"),
    (Check, "CHECK"),
    (Default, "DEFAULT"),
    (Index, "INDEX"),
    (View, "VIEW"),
    (Trigger, "TRIGGER"),
    (Database, "DATABASE"),
    (Table, "TABLE"),
    (Column, "COLUMN"),
    (False, "FALSE"),
    (True, "TRUE")
);

macro_rules! implement_special_character {
    ($name:ident, $(($variant:ident,$symbol:literal)),+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name{
            $(
            #[doc = concat!("Represents symbol:",stringify!($symbol))]
            $variant,
            )+
        }
        impl $name{
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($symbol => Some($name::$variant),)+
                    _ => None
                }
            }
            pub fn as_str(&self) -> &'static str{
                match self {
                    $($name::$variant => $symbol,)+
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    $($name::$variant => $symbol,)+
                };
                f.write_str(s)
            }
        }
    };
}

implement_special_character!(
    Delimiter,
    (CurlyOpen, "{"),
    (CurlyClose, "}"),
    (RoundOpen, "("),
    (RoundClose, ")"),
    (BlockOpen, "["),
    (BlockClose, "]"),
    //(Apostrophe, "'"), Reserved for TextLiteral
    (Comma, ","),
    (Dot, "."),
    (Semicolon, ";"),
    (Backtick, "`"),
    (DoubleQuote, "\""),
    (Colon, ":")
);

implement_special_character!(
    Sign,
    (Eq, "=="),
    (Neq, "!="),
    (Less, "<"),
    (LessEq, "<="),
    (Greater, ">"),
    (GreaterEq, ">="),
    (Plus, "+"),
    (Minus, "-"),
    (Asterisk, "*"),
    (Slash, "/"),
    (Set, "="),
    (Percent, "%"),
    (Tilda, "~"),
    (ExclamationMark, "!"),
    (At, "@"),
    (Hash, "#"),
    (Question, "?"),
    (Dollar, "$"),
    (Caret, "^"),
    (Ampersand, "&"),
    (Number, "№"),
    //(Underscore, "_"), it's a valid ident character, so its better to skip
    (Pipe, "|")
);

/// Turns string into vector of tokens
pub fn tokenize<'a>(source: &'a str) -> Result<Vec<TokenValue<'a>>, ParseError<'a>> {
    let source = source.trim();
    let mut char_ind = source.char_indices().peekable();
    let mut tokens = Vec::with_capacity(50);
    tokens.push(TokenValue::SOF);
    while let Some(&(ind, char)) = char_ind.peek() {
        if char.is_whitespace() {
            char_ind.next();
            continue;
        }
        if char == '\'' {
            char_ind.next();
            let start = ind + char.len_utf8();
            let mut end = start;
            while let Some(&(ind, c)) = char_ind.peek()
                && c != '\''
            {
                end = ind + c.len_utf8();
                char_ind.next();
            }
            if let Some((_, c)) = char_ind.next()
                && c == '\''
            {
                tokens.push(TokenValue::TextLiteral(&source[start..end]));
                continue;
            } else {
                return Err(ParseError::UnclosedBracket('\''));
            }
        }
        if !char.is_alphanumeric() && !char.is_whitespace() && char != '_' {
            let start_ind = ind;
            char_ind.next();
            if let Some(&(end_ind, c)) = char_ind.peek() {
                if let Some(token) = Sign::from_str(&source[start_ind..end_ind + c.len_utf8()]) {
                    tokens.push(TokenValue::Sign(token));
                    char_ind.next();
                    continue;
                }
            }
            if let Some(token) = Sign::from_str(&source[start_ind..start_ind + char.len_utf8()]) {
                tokens.push(TokenValue::Sign(token));
                continue;
            }
            if let Some(token) =
                Delimiter::from_str(&source[start_ind..start_ind + char.len_utf8()])
            {
                tokens.push(TokenValue::Delimiter(token));
                continue;
            }
        }
        let start = ind;
        let mut end = ind;
        while let Some(&(ind, c)) = char_ind.peek()
            && (c.is_alphanumeric() || c == '_')
        {
            end = ind + c.len_utf8();
            char_ind.next();
        }
        if let Some(keyword) = Keyword::from_str(&source[start..end]) {
            tokens.push(TokenValue::Keyword(keyword));
        } else {
            tokens.push(TokenValue::Ident(&source[start..end]));
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::{self as parser, common::ParseError};
    use parser::tokenizer::{TokenValue, tokenize};

    /// Easier TokenValue creation
    macro_rules! token {
        (Ident($value:expr)) => {
            parser::tokenizer::TokenValue::Ident($value.into())
        };
        (Keyword($value:ident)) => {
            parser::tokenizer::TokenValue::Keyword(parser::tokenizer::Keyword::$value)
        };
        (Delimiter($value:ident)) => {
            parser::tokenizer::TokenValue::Delimiter(parser::tokenizer::Delimiter::$value)
        };
        (Sign($value:ident)) => {
            parser::tokenizer::TokenValue::Sign(parser::tokenizer::Sign::$value)
        };
        (TextLiteral($value:literal)) => {
            parser::tokenizer::TokenValue::TextLiteral($value)
        };
    }

    #[test]
    fn success() {
        let string = "SELECT price FROM Prices WHERE price < 100";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![
                TokenValue::SOF,
                token!(Keyword(Select)),
                token!(Ident("price")),
                token!(Keyword(From)),
                token!(Ident("Prices")),
                token!(Keyword(Where)),
                token!(Ident("price")),
                token!(Sign(Less)),
                token!(Ident("100"))
            ]
        );

        let string = "SELECT price FROM Prices WHERE price <= 100";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![
                TokenValue::SOF,
                token!(Keyword(Select)),
                token!(Ident("price")),
                token!(Keyword(From)),
                token!(Ident("Prices")),
                token!(Keyword(Where)),
                token!(Ident("price")),
                token!(Sign(LessEq)),
                token!(Ident("100")),
            ]
        );

        let string = "SELECT price FROM Prices WHERE (price >= 100)";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![
                TokenValue::SOF,
                token!(Keyword(Select)),
                token!(Ident("price")),
                token!(Keyword(From)),
                token!(Ident("Prices")),
                token!(Keyword(Where)),
                token!(Delimiter(RoundOpen)),
                token!(Ident("price")),
                token!(Sign(GreaterEq)),
                token!(Ident("100")),
                token!(Delimiter(RoundClose)),
            ]
        );

        let string = "INSERT INTO Items (price,name) VALUES (50,'Egg')";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![
                TokenValue::SOF,
                token!(Keyword(Insert)),
                token!(Keyword(Into)),
                token!(Ident("Items")),
                token!(Delimiter(RoundOpen)),
                token!(Ident("price")),
                token!(Delimiter(Comma)),
                token!(Ident("name")),
                token!(Delimiter(RoundClose)),
                token!(Keyword(Values)),
                token!(Delimiter(RoundOpen)),
                token!(Ident("50")),
                token!(Delimiter(Comma)),
                token!(TextLiteral("Egg")),
                token!(Delimiter(RoundClose)),
            ]
        );
    }
    #[test]
    fn unclosed_text_literal() {
        let string = "' unclosed";
        let tokenized = tokenize(string);
        assert_eq!(tokenized, Err(ParseError::UnclosedBracket('\'')))
    }
    #[test]
    fn multiple_blanks() {
        let string = "'hello  '";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![TokenValue::SOF, token!(TextLiteral("hello  ")),]
        );
    }

    #[test]
    fn short_identifiers() {
        let string = "u s c";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized.unwrap(),
            vec![
                TokenValue::SOF,
                token!(Ident("u")),
                token!(Ident("s")),
                token!(Ident("c"))
            ]
        );
    }

    #[test]
    fn snake_case_ident() {
        let string = "is_active how_to_come_up_with_good_ident";
        let tokenized = tokenize(string).unwrap();
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Ident("is_active")),
                token!(Ident("how_to_come_up_with_good_ident")),
            ]
        );
    }

    #[test]
    fn all_special_characters() {
        let string = "~`!@#$%^&*()-+={[}]|:;<,>.?/\"";
        let tokenized = tokenize(string).unwrap();
        // -1 to remove SOF token
        assert_eq!(tokenized.len() - 1, string.len())
    }
}
