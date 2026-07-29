use std::fmt::Display;

use crate::common::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenValue<'a> {
    Ident(&'a str),
    Sign(Sign),
    Delimiter(Delimiter),
    Keyword(Keyword),
    TextLiteral(&'a str),
    /// Start of the file
    SOF,
    /// End of the file
    EOF,
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
            TokenValue::EOF => "Eof",
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
    (Take, "TAKE"),
    (Skip, "SKIP"),
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
    (RoundOpen, "("),
    (RoundClose, ")"),
    (BlockOpen, "["),
    (BlockClose, "]"),
    //(Apostrophe, "'"), Reserved for TextLiteral
    (Comma, ","),
    (Dot, "."),
    (DoubleQuote, "\"")
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
    // (Underscore, "_"), it's a valid ident character, so its better to skip
    (Dollar, "$")
);

#[inline(always)]
fn next_utf8_code_point(bytes: &[u8], pos: usize) -> (u32, usize) {
    let b1 = *unsafe { bytes.get_unchecked(pos) };

    if b1 < 0x80 {
        return (b1 as u32, 1);
    }

    if b1 & 0xE0 == 0xC0 && pos + 1 < bytes.len() {
        let b2 = *unsafe { bytes.get_unchecked(pos + 1) };
        let code = ((b1 & 0x1F) as u32) << 6 | (b2 & 0x3F) as u32;
        return (code, 2);
    }

    if b1 & 0xF0 == 0xE0 && pos + 2 < bytes.len() {
        let b2 = *unsafe { bytes.get_unchecked(pos + 1) };
        let b3 = *unsafe { bytes.get_unchecked(pos + 2) };
        let code = ((b1 & 0x0F) as u32) << 12 | ((b2 & 0x3F) as u32) << 6 | (b3 & 0x3F) as u32;
        return (code, 3);
    }

    if b1 & 0xF8 == 0xF0 && pos + 3 < bytes.len() {
        let b2 = *unsafe { bytes.get_unchecked(pos + 1) };
        let b3 = *unsafe { bytes.get_unchecked(pos + 2) };
        let b4 = *unsafe { bytes.get_unchecked(pos + 3) };
        let code = ((b1 & 0x07) as u32) << 18
            | ((b2 & 0x3F) as u32) << 12
            | ((b3 & 0x3F) as u32) << 6
            | (b4 & 0x3F) as u32;
        return (code, 4);
    }

    (b1 as u32, 1)
}
#[inline(always)]
fn is_valid_identifier_char(c: char) -> bool {
    // ascii, should be alphanumeric or '_'
    if c.is_ascii() {
        return c.is_ascii_alphanumeric() || c == '_';
    }

    // Should be alphanumeric
    if !c.is_alphabetic() && !c.is_numeric() {
        return false;
    }

    // Remove unicode codepoint blocks (modifiers, fractions, superscripts, subscripts, rare signs)
    match c as u32 {
        // Latin-1 Supplement
        0x0080..=0x00FF => c.is_alphabetic() && (c as u32 != 0xAA) && (c as u32 != 0xBA),
        // Spacing Modifier Letters
        0x02B0..=0x02FF => false,
        // Phonetic Extensions
        0x1D00..=0x1D7F => false,
        // Superscripts and subscripts
        0x2070..=0x209F => false,
        // Number forms
        0x2150..=0x218F => false,
        _ => true,
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    source: &'a [u8],
    position: usize,
}
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position
    }

    #[inline(always)]
    pub fn next_token(&mut self) -> Result<TokenValue<'a>, ParseError<'a>> {
        while self.position < self.source.len() {
            let b = unsafe { *self.source.get_unchecked(self.position) };
            if b.is_ascii_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
        if self.position >= self.source.len() {
            return Ok(TokenValue::EOF);
        }

        let b = unsafe { *self.source.get_unchecked(self.position) };
        // Text literal, includes any utf-8 characters
        if b == b'\'' {
            self.position += 1;
            let start = self.position;
            // Skip until end or another apostrophe
            while self.position < self.source.len()
                && *unsafe { self.source.get_unchecked(self.position) } != b'\''
            {
                self.position += 1;
            }
            // Check if literal ends with apostrophe
            if self.position < self.source.len()
                && *unsafe { self.source.get_unchecked(self.position) } == b'\''
            {
                let literal =
                    unsafe { str::from_utf8_unchecked(&self.source[start..self.position]) };
                self.position += 1;
                return Ok(TokenValue::TextLiteral(literal));
            } else {
                return Err(ParseError::UnclosedBracket('\''));
            }
        }

        // Handle special characters (non-alphanumeric, excluding underscores).
        // We only check ASCII since all valid delimiters and signs are ASCII.
        if !b.is_ascii_alphanumeric() && b != b'_' {
            let word = if self.position + 1 < self.source.len() {
                [b, *unsafe { self.source.get_unchecked(self.position + 1) }]
            } else {
                [b, 0]
            };
            let token_match = match &word {
                b"==" => Some((TokenValue::Sign(Sign::Eq), 2)),
                b"!=" => Some((TokenValue::Sign(Sign::Neq), 2)),
                b"<=" => Some((TokenValue::Sign(Sign::LessEq), 2)),
                b">=" => Some((TokenValue::Sign(Sign::GreaterEq), 2)),
                _ => match b {
                    b'<' => Some((TokenValue::Sign(Sign::Less), 1)),
                    b'>' => Some((TokenValue::Sign(Sign::Greater), 1)),
                    b'+' => Some((TokenValue::Sign(Sign::Plus), 1)),
                    b'-' => Some((TokenValue::Sign(Sign::Minus), 1)),
                    b'*' => Some((TokenValue::Sign(Sign::Asterisk), 1)),
                    b'/' => Some((TokenValue::Sign(Sign::Slash), 1)),
                    b'=' => Some((TokenValue::Sign(Sign::Set), 1)),
                    b'%' => Some((TokenValue::Sign(Sign::Percent), 1)),
                    b'$' => Some((TokenValue::Sign(Sign::Dollar), 1)),
                    b'(' => Some((TokenValue::Delimiter(Delimiter::RoundOpen), 1)),
                    b')' => Some((TokenValue::Delimiter(Delimiter::RoundClose), 1)),
                    b'[' => Some((TokenValue::Delimiter(Delimiter::BlockOpen), 1)),
                    b']' => Some((TokenValue::Delimiter(Delimiter::BlockClose), 1)),
                    b',' => Some((TokenValue::Delimiter(Delimiter::Comma), 1)),
                    b'.' => Some((TokenValue::Delimiter(Delimiter::Dot), 1)),
                    b'"' => Some((TokenValue::Delimiter(Delimiter::DoubleQuote), 1)),
                    _ => None,
                },
            };
            if let Some((token, size)) = token_match {
                self.position += size;
                return Ok(token);
            }
        }
        // Identifiers
        let start = self.position;
        while self.position < self.source.len() {
            let byte = *unsafe { self.source.get_unchecked(self.position) };
            if byte.is_ascii() {
                if byte.is_ascii_alphanumeric() || byte == b'_' {
                    self.position += 1;
                } else {
                    break;
                }
            } else {
                let (codepoint, size) = next_utf8_code_point(self.source, self.position);
                let char = unsafe { char::from_u32_unchecked(codepoint) };
                if is_valid_identifier_char(char) {
                    self.position += size;
                } else {
                    return Err(ParseError::UnsupportedCharacter { character: char });
                }
            }
        }
        if start == self.position {
            let (codepoint, _) = next_utf8_code_point(self.source, self.position);
            let unknown_char = unsafe { char::from_u32_unchecked(codepoint) };

            return Err(ParseError::UnsupportedCharacter {
                character: unknown_char,
            });
        }
        let ident = unsafe { str::from_utf8_unchecked(&self.source[start..self.position]) };
        if let Some(keyword) = Keyword::from_str(ident) {
            return Ok(TokenValue::Keyword(keyword));
        } else {
            return Ok(TokenValue::Ident(ident));
        }
    }

    pub fn source(&self) -> &'a [u8] {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use crate::{self as parser, common::ParseError, lexer::Lexer};
    use parser::lexer::TokenValue;

    /// Easier TokenValue creation
    macro_rules! token {
        (Ident($value:expr)) => {
            parser::lexer::TokenValue::Ident($value.into())
        };
        (Keyword($value:ident)) => {
            parser::lexer::TokenValue::Keyword(parser::lexer::Keyword::$value)
        };
        (Delimiter($value:ident)) => {
            parser::lexer::TokenValue::Delimiter(parser::lexer::Delimiter::$value)
        };
        (Sign($value:ident)) => {
            parser::lexer::TokenValue::Sign(parser::lexer::Sign::$value)
        };
        (TextLiteral($value:literal)) => {
            parser::lexer::TokenValue::TextLiteral($value)
        };
    }
    fn collect_tokens_until_eof<'a>(
        mut lexer: Lexer<'a>,
    ) -> Result<Vec<TokenValue<'a>>, ParseError<'a>> {
        let mut tokens = Vec::new();
        while let token = lexer.next_token()?
            && token != TokenValue::EOF
        {
            tokens.push(token);
        }
        Ok(tokens)
    }
    #[test]
    fn success() {
        let string = "SELECT price FROM Prices WHERE price < 100";
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![
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
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![
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
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![
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
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![
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
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer);
        assert_eq!(tokens, Err(ParseError::UnclosedBracket('\'')))
    }
    #[test]
    fn multiple_blanks() {
        let string = "'hello  '";
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(tokens, vec![token!(TextLiteral("hello  ")),]);
    }

    #[test]
    fn short_identifiers() {
        let string = "u s c";
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![token!(Ident("u")), token!(Ident("s")), token!(Ident("c"))]
        );
    }

    #[test]
    fn snake_case_ident() {
        let string = "is_active how_to_come_up_with_good_ident";
        let lexer = Lexer::new(string);
        let tokens = collect_tokens_until_eof(lexer).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Ident("is_active")),
                token!(Ident("how_to_come_up_with_good_ident")),
            ]
        );
    }

    #[test]
    fn unsupported_characters() {
        let string = "~`@#^&{}|?\\¢£¤¥¦§¨©«¬®¯°±²³´¶·¸¹º»¼½¾¿×÷±∓√∛∜∝∞∟∠∡∢∣∤∥∦∧∨∩∪∴∵∶∷∸∹∺∻∼∽∾∿≀≁≂≃≄≅≆≇≈≉≊≋≌≍≎≏≐≑≒≓≔≕≖≗≘≙≚≛≜≝≞≟≠≡≢≣≤≥≦≧≨≩≪≫≬≭≮≯≰≱≲≳≴≵≶≷≸≹≺≻≼≽≾≿⊀⊁⊂⊃⊄⊅⊆⊇⊈⊉⊊⊋⊌⊍⊎⊏⊐⊑⊒⊓⊔⊕⊖⊗⊘⊙⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊥⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊰⊱⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⊿⋀⋁⋂⋃⋄⋅⋆⋇⋈⋉⋊⋋⋌⋍⋎⋏⋐⋑⋒⋓⋔⋕⋖⋗⋘⋙⋚⋛⋜⋝⋞⋟⋠⋡⋢⋣⋤⋥⋦⋧⋨⋩⋪⋫⋬⋭⋮⋯⋰⋱";
        for c in string.chars() {
            let as_str = format!("{}", c);
            let lexer = Lexer::new(&as_str);
            let tokens = collect_tokens_until_eof(lexer);
            assert_eq!(
                tokens.unwrap_err(),
                ParseError::UnsupportedCharacter { character: c }
            )
        }
    }
}
