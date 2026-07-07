use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValue<'a> {
    Ident(&'a str),
    Sign(Sign),
    Delimiter(Delimiter),
    Keyword(&'a str),
    /// New line or space
    Blank,
    /// Start of the file
    SOF,
}
impl<'a> Display for TokenValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenValue::Ident(w) => w,
                TokenValue::Sign(sign) => sign.as_str(),
                TokenValue::Delimiter(delimiter) => delimiter.as_str(),
                TokenValue::Keyword(k) => k,
                TokenValue::Blank => " ",
                TokenValue::SOF => "Sof",
            }
        )
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
            TokenValue::Keyword(k) => k,
            TokenValue::Blank => " ",
            TokenValue::SOF => "Sof",
        }
    }
    pub fn is_ident(&self) -> bool {
        matches!(self, TokenValue::Ident(_))
    }
    pub fn is_keyword(&self) -> bool {
        matches!(self, TokenValue::Keyword(_))
    }
    pub fn is_blank(&self) -> bool {
        matches!(self, TokenValue::Blank)
    }
    pub fn is_sof(&self) -> bool {
        matches!(self, TokenValue::SOF)
    }
}

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
    (Apostrophe, "'"),
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

macro_rules! lookup_fn {
    ($($branch:literal),+) => {
        #[inline(always)]
        fn lookup_keyword(value:&str) -> bool{
            match value{
                $($branch => true,)+
                _ => false
            }
        }
    };
}
lookup_fn!(
    "SELECT",
    "FROM",
    "WHERE",
    "GROUP",
    "BY",
    "IF",
    "ORDER",
    "DISTINCT",
    "AS",
    "LIMIT",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "DROP",
    "TRUNCATE",
    "USING",
    "AND",
    "OR",
    "NOT",
    "IN",
    "IS",
    "NULL",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "ALL",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "UNIQUE",
    "CHECK",
    "DEFAULT",
    "INDEX",
    "VIEW",
    "TRIGGER",
    "DATABASE",
    "TABLE",
    "COLUMN",
    "FALSE",
    "TRUE"
);

/// Turns string into vector of tokens
pub fn tokenize(source: &str) -> Vec<TokenValue<'_>> {
    let source = source.trim();
    let mut char_ind = source.char_indices().peekable();
    let mut tokens = Vec::with_capacity(50);
    tokens.push(TokenValue::SOF);
    while let Some(&(ind, char)) = char_ind.peek() {
        if char.is_whitespace() {
            tokens.push(TokenValue::Blank);
            char_ind.next();
            continue;
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
        if lookup_keyword(&source[start..end]) {
            tokens.push(TokenValue::Keyword(&source[start..end]));
        } else {
            tokens.push(TokenValue::Ident(&source[start..end]));
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use crate as parser;
    use parser::tokenizer::{TokenValue, tokenize};

    /// Easier TokenValue creation
    macro_rules! token {
        (Ident($value:expr)) => {
            parser::tokenizer::TokenValue::Ident($value.into())
        };
        (Keyword($value:expr)) => {
            parser::tokenizer::TokenValue::Keyword($value.into())
        };
        (Delimiter($value:ident)) => {
            parser::tokenizer::TokenValue::Delimiter(parser::tokenizer::Delimiter::$value)
        };
        (Sign($value:ident)) => {
            parser::tokenizer::TokenValue::Sign(parser::tokenizer::Sign::$value)
        };
        () => {
            parser::tokenizer::TokenValue::Blank
        };
    }

    #[test]
    fn success() {
        let string = "SELECT price FROM Prices WHERE price < 100";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Keyword("SELECT")),
                token!(),
                token!(Ident("price")),
                token!(),
                token!(Keyword("FROM")),
                token!(),
                token!(Ident("Prices")),
                token!(),
                token!(Keyword("WHERE")),
                token!(),
                token!(Ident("price")),
                token!(),
                token!(Sign(Less)),
                token!(),
                token!(Ident("100"))
            ]
        );

        let string = "SELECT price FROM Prices WHERE price <= 100";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Keyword("SELECT")),
                token!(),
                token!(Ident("price")),
                token!(),
                token!(Keyword("FROM")),
                token!(),
                token!(Ident("Prices")),
                token!(),
                token!(Keyword("WHERE")),
                token!(),
                token!(Ident("price")),
                token!(),
                token!(Sign(LessEq)),
                token!(),
                token!(Ident("100")),
            ]
        );

        let string = "SELECT price FROM Prices WHERE (price >= 100)";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Keyword("SELECT")),
                token!(),
                token!(Ident("price")),
                token!(),
                token!(Keyword("FROM")),
                token!(),
                token!(Ident("Prices")),
                token!(),
                token!(Keyword("WHERE")),
                token!(),
                token!(Delimiter(RoundOpen)),
                token!(Ident("price")),
                token!(),
                token!(Sign(GreaterEq)),
                token!(),
                token!(Ident("100")),
                token!(Delimiter(RoundClose)),
            ]
        );

        let string = "INSERT INTO Items (price,name) VALUES (50,'Egg')";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Keyword("INSERT")),
                token!(),
                token!(Keyword("INTO")),
                token!(),
                token!(Ident("Items")),
                token!(),
                token!(Delimiter(RoundOpen)),
                token!(Ident("price")),
                token!(Delimiter(Comma)),
                token!(Ident("name")),
                token!(Delimiter(RoundClose)),
                token!(),
                token!(Keyword("VALUES")),
                token!(),
                token!(Delimiter(RoundOpen)),
                token!(Ident("50")),
                token!(Delimiter(Comma)),
                token!(Delimiter(Apostrophe)),
                token!(Ident("Egg")),
                token!(Delimiter(Apostrophe)),
                token!(Delimiter(RoundClose)),
            ]
        );
    }
    #[test]
    fn multiple_blanks() {
        let string = "'hello  '";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Delimiter(Apostrophe)),
                token!(Ident("hello")),
                token!(),
                token!(),
                token!(Delimiter(Apostrophe)),
            ]
        );
    }

    #[test]
    fn short_identifiers() {
        let string = "u s c";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Ident("u")),
                token!(),
                token!(Ident("s")),
                token!(),
                token!(Ident("c"))
            ]
        );
    }

    #[test]
    fn snake_case_ident() {
        let string = "is_active how_to_come_up_with_good_ident";
        let tokenized = tokenize(string);
        assert_eq!(
            tokenized,
            vec![
                TokenValue::SOF,
                token!(Ident("is_active")),
                token!(),
                token!(Ident("how_to_come_up_with_good_ident")),
            ]
        );
    }

    #[test]
    fn all_special_characters() {
        let string = "~`!@#$%^&*()-+={[}]|:;'<,>.?/\"";
        let tokenized = tokenize(string);
        // -1 to remove
        assert_eq!(tokenized.len() - 1, string.len())
    }
}
