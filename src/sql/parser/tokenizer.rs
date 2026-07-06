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
///
/// '''
///
/// '''
// pub fn tokenize<'a>(string: &'a str) -> Vec<TokenValue<'a>> {
//     let string = string.trim();
//     let split = string.split(" ");
//     let mut tokens = vec![TokenValue::SOF];
//     let mut char_map = Vec::new();
//     for s in split {
//         for (byte_pos, _) in s.char_indices() {
//             char_map.push(byte_pos);
//         }
//         char_map.push(s.len());

//         let mut pos = 0;
//         let mut i = 0;

//         while i < (char_map.len() - 1) {
//             let char_i = char_map[i];
//             let char_pos = char_map[pos];

//             if i + 2 < char_map.len() {
//                 let char_i_2 = char_map[i + 2];
//                 if let Some(sign) = Sign::from_str(&s[char_i..char_i_2]) {
//                     if lookup_keyword(&s[char_pos..char_i]) && char_i > char_pos {
//                         tokens.push(TokenValue::Keyword(&s[char_pos..char_i]));
//                     } else if char_i > char_pos {
//                         tokens.push(TokenValue::Ident(&s[char_pos..char_i]));
//                     }
//                     i += 2;
//                     pos = i;
//                     tokens.push(TokenValue::Sign(sign));
//                     continue;
//                 }
//             }

//             let char_i_1 = char_map[i + 1];
//             if let Some(sign) = Sign::from_str(&s[char_i..char_i_1]) {
//                 if lookup_keyword(&s[char_pos..char_i]) && char_i > char_pos {
//                     tokens.push(TokenValue::Keyword(&s[char_pos..char_i]));
//                 } else if char_i > char_pos {
//                     tokens.push(TokenValue::Ident(&s[char_pos..char_i]));
//                 }
//                 i += 1;
//                 pos = i;
//                 tokens.push(TokenValue::Sign(sign));
//                 continue;
//             }

//             if let Some(delim) = Delimiter::from_str(&s[char_i..char_i_1]) {
//                 if lookup_keyword(&s[char_pos..char_i]) && char_i > char_pos {
//                     tokens.push(TokenValue::Keyword(&s[char_pos..char_i]));
//                 } else if char_i > char_pos {
//                     tokens.push(TokenValue::Ident(&s[char_pos..char_i]));
//                 }
//                 i += 1;
//                 pos = i;
//                 tokens.push(TokenValue::Delimiter(delim));
//                 continue;
//             }

//             i += 1;
//         }

//         if pos < (char_map.len() - 1) {
//             let char_pos = char_map[pos];
//             let char_end = char_map[char_map.len() - 1];

//             if char_end > char_pos {
//                 if lookup_keyword(&s[char_pos..char_end]) {
//                     tokens.push(TokenValue::Keyword(&s[char_pos..char_end]));
//                 } else {
//                     tokens.push(TokenValue::Ident(&s[char_pos..char_end]));
//                 }
//             }
//         }
//         tokens.push(TokenValue::Blank);
//         char_map.clear();
//     }

//     if !tokens.is_empty() {
//         tokens.remove(tokens.len() - 1);
//     }
//     tokens
// }
//tokenize zeco copy
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
    use crate as simply_db;
    use simply_db::sql::parser::tokenizer::{TokenValue, tokenize};

    /// Easier TokenValue creation
    macro_rules! token {
        (Ident($value:expr)) => {
            simply_db::sql::parser::tokenizer::TokenValue::Ident($value.into())
        };
        (Keyword($value:expr)) => {
            simply_db::sql::parser::tokenizer::TokenValue::Keyword($value.into())
        };
        (Delimiter($value:ident)) => {
            simply_db::sql::parser::tokenizer::TokenValue::Delimiter(
                simply_db::sql::parser::tokenizer::Delimiter::$value,
            )
        };
        (Sign($value:ident)) => {
            simply_db::sql::parser::tokenizer::TokenValue::Sign(
                simply_db::sql::parser::tokenizer::Sign::$value,
            )
        };
        () => {
            simply_db::sql::parser::tokenizer::TokenValue::Blank
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
