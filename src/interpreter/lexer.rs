use miette::Diagnostic;
use paste::paste;
use thiserror::Error;

use super::{
    num::{from_slice_radix, ParseIntError},
    token::{token, FullUnit, Token, Unit},
    unit_prefix::UnitPrefix,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Diagnostic)]
pub enum LexErrorKind {
    #[error("Unexpected character")]
    UnexpectedCharacter,
    #[error("Invalid digit")]
    InvalidDigit(#[source] super::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Diagnostic)]
#[error("{}", kind)]
pub struct LexError {
    kind: LexErrorKind,
    #[label = "here"]
    loc: usize,
}

impl LexError {
    pub fn new(kind: LexErrorKind, loc: usize) -> Self {
        Self { kind, loc }
    }

    #[inline]
    pub fn loc(&self) -> usize {
        self.loc
    }
}

#[inline]
fn parse_nr<const RADIX: u32>(
    s: &[u8],
    invalid_digit: fn(&u8) -> bool,
) -> Result<(u64, &[u8]), ParseIntError> {
    let end = s.iter().position(invalid_digit).unwrap_or(s.len());
    let (num, rest) = s.split_at(end);
    let val = from_slice_radix::<RADIX>(num)?;
    Ok((val, rest))
}

#[inline]
fn parse_bin_nr(s: &[u8]) -> Result<(u64, &[u8]), ParseIntError> {
    parse_nr::<2>(s, |c| !matches!(c, b'0' | b'1'))
}

#[inline]
fn parse_oct_nr(s: &[u8]) -> Result<(u64, &[u8]), ParseIntError> {
    parse_nr::<8>(s, |c| !matches!(c, b'0'..=b'7'))
}

#[inline]
fn parse_dec_nr(s: &[u8]) -> Result<(u64, &[u8]), ParseIntError> {
    parse_nr::<10>(s, |c| !c.is_ascii_digit())
}

#[inline]
fn parse_hex_nr(s: &[u8]) -> Result<(u64, &[u8]), ParseIntError> {
    parse_nr::<16>(s, |c| !c.is_ascii_hexdigit())
}

pub struct Lexer<'a> {
    input: Option<&'a [u8]>,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut input = input.as_bytes();
        let old_len = input.len();
        input = input.trim_ascii_start();

        Self {
            input: Some(input),
            current: old_len - input.len(),
        }
    }

    #[inline]
    fn trim_whitespace(&mut self) {
        if let Some(mut input) = self.input {
            let old_len = input.len();
            input = input.trim_ascii_start();
            self.input = Some(input);
            self.current += old_len - input.len();
        }
    }

    #[inline]
    const fn span(&self, len: usize) -> std::ops::Range<usize> {
        self.current..(self.current + len)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        use LexError as LE;
        use LexErrorKind as LEK;

        self.trim_whitespace();

        let input = self.input?;
        if input.is_empty() {
            self.input = None;
            return Some(Ok(token!(Eof, self.span(0))));
        }

        macro_rules! tok {
            ($kind:ident, $len:literal) => {
                token!($kind, self.span($len))
            };
            ($kind:ident($($val:expr),+), $len:expr) => {
                token!($kind($($val),+), self.span($len))
            };
        }

        macro_rules! unit {
            ($prefix:expr, $unit:expr, $len:expr) => {
                token!(Unit(FullUnit($prefix, $unit)), self.span($len))
            };
            ($unit:expr, $len:expr) => {
                unit!(UnitPrefix::None, $unit, $len)
            };
        }

        macro_rules! parse_as {
            ($rad:ident, $input:ident, $offset:literal) => {{
                paste! {
                    let (val, rest) = match [<parse_ $rad _nr>]($input) {
                        Ok(val) => val,
                        Err(e) => return Some(Err(LE::new(LEK::InvalidDigit(e),self.current + $offset))),
                    };
                    let len = input.len() - rest.len();
                    (tok!(Integer(val), len), rest)
                }
            }};
            ($rad:ident, $input:ident) => {
                parse_as!($rad, $input, 0)
            };
        }

        macro_rules! parse_unit {
            ($input:ident, $prefix:expr, $len:literal) => {{
                match $input {
                    [b'b', rest @ ..] => (unit!($prefix, Unit::Bit, $len + 1), rest),
                    [b'B', rest @ ..] => (unit!($prefix, Unit::Byte, $len + 1), rest),
                    _ => return Some(Err(LE::new(LEK::UnexpectedCharacter, self.current))),
                }
            }};
        }

        let (token, rest) = match input {
            // Single character tokens
            [b'-', rest @ ..] => (tok!(Minus, 1), rest),
            [b'+', rest @ ..] => (tok!(Plus, 1), rest),
            [b'*', rest @ ..] => (tok!(Star, 1), rest),
            [b'/', rest @ ..] => (tok!(Slash, 1), rest),
            [b'(', rest @ ..] => (tok!(LeftParen, 1), rest),
            [b')', rest @ ..] => (tok!(RightParen, 1), rest),
            [b'b', rest @ ..] => (unit!(Unit::Bit, 1), rest),
            [b'B', rest @ ..] => (unit!(Unit::Byte, 1), rest),
            // Keywords
            [b'a', b's', rest @ ..] => (tok!(As, 2), rest),
            // Literals
            [b'0', c, rest @ ..] => match c {
                b'b' => parse_as!(bin, rest, 2),
                b'o' => parse_as!(oct, rest, 2),
                b'x' => parse_as!(hex, rest, 2),
                _ => parse_as!(dec, input),
            },
            [b'0'..=b'9', ..] => parse_as!(dec, input),
            [b'k' | b'K', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Kibi, 2),
            [b'm' | b'M', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Mebi, 2),
            [b'g' | b'G', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Gibi, 2),
            [b't' | b'T', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Tebi, 2),
            [b'p' | b'P', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Pebi, 2),
            [b'e' | b'E', b'i' | b'I', rest @ ..] => parse_unit!(rest, UnitPrefix::Exbi, 2),
            [b'k' | b'K', rest @ ..] => parse_unit!(rest, UnitPrefix::Kilo, 1),
            [b'm' | b'M', rest @ ..] => parse_unit!(rest, UnitPrefix::Mega, 1),
            [b'g' | b'G', rest @ ..] => parse_unit!(rest, UnitPrefix::Giga, 1),
            [b't' | b'T', rest @ ..] => parse_unit!(rest, UnitPrefix::Tera, 1),
            [b'p' | b'P', rest @ ..] => parse_unit!(rest, UnitPrefix::Peta, 1),
            [b'e' | b'E', rest @ ..] => parse_unit!(rest, UnitPrefix::Exa, 1),
            _ => return Some(Err(LE::new(LEK::UnexpectedCharacter, self.current))),
        };

        self.current += token.len();
        self.input = Some(rest);

        Some(Ok(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bin_nr() {
        let input = b"101010";
        let (val, rest) = parse_bin_nr(input).unwrap();
        assert_eq!(val, 42);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_oct_nr() {
        let input = b"52";
        let (val, rest) = parse_oct_nr(input).unwrap();
        assert_eq!(val, 42);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_dec_nr() {
        let input = b"42";
        let (val, rest) = parse_dec_nr(input).unwrap();
        assert_eq!(val, 42);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_hex_nr() {
        let input = b"2a";
        let (val, rest) = parse_hex_nr(input).unwrap();
        assert_eq!(val, 42);
        assert!(rest.is_empty());
    }

    macro_rules! lex {
        ($s:expr) => {
            Lexer::new($s).collect::<Result<Vec<_>, _>>()
        };
    }

    #[test]
    fn test_lexer() {
        let tokens = lex!("42 + 42").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(42), 0..2),
                token!(Plus, 3..4),
                token!(Integer(42), 5..7),
                token!(Eof, 7..7),
            ]
        );
    }

    #[test]
    fn test_lexer_unit_prefix() {
        let tokens = lex!("42Kib").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(42), 0..2),
                token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Bit)), 2..5),
                token!(Eof, 5..5),
            ]
        );
    }

    #[test]
    fn test_lexer_unit() {
        let tokens = lex!("42B").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(42), 0..2),
                token!(Unit(FullUnit(UnitPrefix::None, Unit::Byte)), 2..3),
                token!(Eof, 3..3),
            ]
        );
    }

    #[test]
    fn test_lexer_unit_prefix_and_unit() {
        let tokens = lex!("42KiB").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(42), 0..2),
                token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Byte)), 2..5),
                token!(Eof, 5..5),
            ]
        );
    }

    #[test]
    fn test_lexer_unit_and_unit_prefix() {
        let tokens = lex!("42KB").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(42), 0..2),
                token!(Unit(FullUnit(UnitPrefix::Kilo, Unit::Byte)), 2..4),
                token!(Eof, 4..4),
            ]
        );
    }

    #[test]
    fn test_lexer_long_expression() {
        let tokens = lex!("12KiB / 02MiB * 42Gib -(42TiB + 42PiB)+ 42EiB").unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Integer(12), 0..2),
                token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Byte)), 2..5),
                token!(Slash, 6..7),
                token!(Integer(2), 8..10),
                token!(Unit(FullUnit(UnitPrefix::Mebi, Unit::Byte)), 10..13),
                token!(Star, 14..15),
                token!(Integer(42), 16..18),
                token!(Unit(FullUnit(UnitPrefix::Gibi, Unit::Bit)), 18..21),
                token!(Minus, 22..23),
                token!(LeftParen, 23..24),
                token!(Integer(42), 24..26),
                token!(Unit(FullUnit(UnitPrefix::Tebi, Unit::Byte)), 26..29),
                token!(Plus, 30..31),
                token!(Integer(42), 32..34),
                token!(Unit(FullUnit(UnitPrefix::Pebi, Unit::Byte)), 34..37),
                token!(RightParen, 37..38),
                token!(Plus, 38..39),
                token!(Integer(42), 40..42),
                token!(Unit(FullUnit(UnitPrefix::Exbi, Unit::Byte)), 42..45),
                token!(Eof, 45..45),
            ]
        );
    }

    #[test]
    fn test_lexer_single_digit() {
        let tokens = lex!("0").unwrap();
        assert_eq!(tokens, vec![token!(Integer(0), 0..1), token!(Eof, 1..1),]);
    }

    #[test]
    fn test_lexer_invalid_input() {
        use LexError as LE;
        use LexErrorKind as LEK;

        let res = lex!("42 + 42x").unwrap_err();
        assert_eq!(res, LE::new(LEK::UnexpectedCharacter, 7));

        let res = lex!("0x").unwrap_err();
        assert_eq!(res, LE::new(LEK::InvalidDigit(ParseIntError::Empty), 2));

        let res = lex!("0b").unwrap_err();
        assert_eq!(res, LE::new(LEK::InvalidDigit(ParseIntError::Empty), 2));

        let res = lex!("0o").unwrap_err();
        assert_eq!(res, LE::new(LEK::InvalidDigit(ParseIntError::Empty), 2));

        let res = lex!("0xg").unwrap_err();
        assert_eq!(res, LE::new(LEK::InvalidDigit(ParseIntError::Empty), 2));

        let res = lex!("0a").unwrap_err();
        assert_eq!(res, LE::new(LEK::UnexpectedCharacter, 1));

        let res = lex!("ak").unwrap_err();
        assert_eq!(res, LE::new(LEK::UnexpectedCharacter, 0));
    }
}
