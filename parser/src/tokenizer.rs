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
/// Turns string into vector of tokens
pub fn tokenize<'a>(source: &'a str) -> Result<Vec<TokenValue<'a>>, ParseError<'a>> {
    let source = source.trim();
    let bytes = source.as_bytes();
    let mut tokens = Vec::with_capacity(50);
    tokens.push(TokenValue::SOF);
    let mut i = 0;
    while i < bytes.len() {
        let b = *unsafe { bytes.get_unchecked(i) };
        // Skip whitespaces
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // Text literal, includes any utf-8 characters
        if b == b'\'' {
            i += 1;
            let start = i;
            // Skip until end or another apostrophe
            while i < bytes.len() && *unsafe { bytes.get_unchecked(i) } != b'\'' {
                i += 1;
            }
            // Check if literal ends with apostrophe
            if i < bytes.len() && *unsafe { bytes.get_unchecked(i) } == b'\'' {
                let literal = unsafe { str::from_utf8_unchecked(&bytes[start..i]) };
                tokens.push(TokenValue::TextLiteral(literal));
                i += 1;
                continue;
            } else {
                return Err(ParseError::UnclosedBracket('\''));
            }
        }

        // Handle special characters (non-alphanumeric, excluding underscores).
        // We only check ASCII since all valid delimiters and signs are ASCII.
        if !b.is_ascii_alphanumeric() && b != b'_' {
            let word = if i + 1 < bytes.len() {
                [b, *unsafe { bytes.get_unchecked(i + 1) }]
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
                tokens.push(token);
                i += size;
                continue;
            }
        }
        // Identifiers
        let start = i;
        while i < source.len() {
            let (codepoint, size) = next_utf8_code_point(bytes, i);
            let char = unsafe { char::from_u32_unchecked(codepoint) };
            if is_valid_identifier_char(char) {
                i += size;
            } else if char.is_ascii() {
                break;
            } else {
                return Err(ParseError::UnsupportedCharacter { character: char });
            }
        }
        if start == i {
            let (codepoint, _) = next_utf8_code_point(bytes, i);
            let unknown_char = unsafe { char::from_u32_unchecked(codepoint) };

            return Err(ParseError::UnsupportedCharacter {
                character: unknown_char,
            });
        }
        if let Some(keyword) = Keyword::from_str(&source[start..i]) {
            tokens.push(TokenValue::Keyword(keyword));
        } else {
            tokens.push(TokenValue::Ident(&source[start..i]));
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
    fn unsupported_characters() {
        let string = "~`@#^&{}|?\\¢£¤¥¦§¨©«¬®¯°±²³´¶·¸¹º»¼½¾¿×÷±∓√∛∜∝∞∟∠∡∢∣∤∥∦∧∨∩∪∴∵∶∷∸∹∺∻∼∽∾∿≀≁≂≃≄≅≆≇≈≉≊≋≌≍≎≏≐≑≒≓≔≕≖≗≘≙≚≛≜≝≞≟≠≡≢≣≤≥≦≧≨≩≪≫≬≭≮≯≰≱≲≳≴≵≶≷≸≹≺≻≼≽≾≿⊀⊁⊂⊃⊄⊅⊆⊇⊈⊉⊊⊋⊌⊍⊎⊏⊐⊑⊒⊓⊔⊕⊖⊗⊘⊙⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊥⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊰⊱⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⊿⋀⋁⋂⋃⋄⋅⋆⋇⋈⋉⋊⋋⋌⋍⋎⋏⋐⋑⋒⋓⋔⋕⋖⋗⋘⋙⋚⋛⋜⋝⋞⋟⋠⋡⋢⋣⋤⋥⋦⋧⋨⋩⋪⋫⋬⋭⋮⋯⋰⋱";
        for c in string.chars() {
            let as_str = format!("{}", c);
            let tokenized = tokenize(&as_str);
            assert_eq!(
                tokenized.unwrap_err(),
                ParseError::UnsupportedCharacter { character: c }
            )
        }
    }
}
