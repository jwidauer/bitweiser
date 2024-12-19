use paste::paste;
use thiserror::Error;

use crate::unit_prefix::UnitPrefix;

use super::{
    num::{from_slice_radix, ParseIntError},
    token::{token, Token, Unit},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TokenizerError {
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Unexpected character at index {0}")]
    UnexpectedCharacter(usize),
    #[error("Invalid digit at index {0}")]
    InvalidDigit(usize, #[source] super::num::ParseIntError),
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

pub(crate) fn tokenize(s: &str) -> Result<Vec<Token>, TokenizerError> {
    use UnitPrefix as UP;

    let mut tokens = vec![];
    let mut input = s.as_bytes();

    // Remove leading whitespace
    let old_len = input.len();
    input = input.trim_ascii_start();
    let mut current = old_len - input.len();

    if input.is_empty() {
        return Err(TokenizerError::UnexpectedEndOfInput);
    }

    macro_rules! tok {
        ($kind:ident, $len:literal) => {
            token!($kind, current..(current + $len))
        };
        ($kind:ident($($val:expr),+), $len:expr) => {
            token!($kind($($val),+), current..(current + $len))
        };
    }

    macro_rules! parse_as {
        ($rad:ident, $input:ident, $offset:expr) => {{
            paste! {
                let (val, rest) =
                    [<parse_ $rad _nr>]($input).map_err(|e| TokenizerError::InvalidDigit(current + $offset, e))?;
                let len = input.len() - rest.len();
                (tok!(Number(val), len), rest)
            }
        }};
        ($rad:ident, $input:ident) => {
            parse_as!($rad, $input, 0)
        };
    }

    macro_rules! parse_unit {
        ($input:ident, $prefix:expr, $len:literal) => {{
            match $input {
                [b'b', rest @ ..] => (tok!(Unit(Some($prefix), Unit::Bit), $len + 1), rest),
                [b'B', rest @ ..] => (tok!(Unit(Some($prefix), Unit::Byte), $len + 1), rest),
                _ => return Err(TokenizerError::UnexpectedCharacter(current)),
            }
        }};
    }

    while !input.is_empty() {
        let old_len = input.len();
        input = input.trim_ascii_start();
        current += old_len - input.len();

        let (token, rest) = match input {
            // Single character tokens
            [b'-', rest @ ..] => (tok!(Minus, 1), rest),
            [b'+', rest @ ..] => (tok!(Plus, 1), rest),
            [b'*', rest @ ..] => (tok!(Star, 1), rest),
            [b'/', rest @ ..] => (tok!(Slash, 1), rest),
            [b'(', rest @ ..] => (tok!(LeftParen, 1), rest),
            [b')', rest @ ..] => (tok!(RightParen, 1), rest),
            [b'b', rest @ ..] => (tok!(Unit(None, Unit::Bit), 1), rest),
            [b'B', rest @ ..] => (tok!(Unit(None, Unit::Byte), 1), rest),
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
            [b'k' | b'K', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Kibi, 2),
            [b'm' | b'M', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Mebi, 2),
            [b'g' | b'G', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Gibi, 2),
            [b't' | b'T', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Tebi, 2),
            [b'p' | b'P', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Pebi, 2),
            [b'e' | b'E', b'i' | b'I', rest @ ..] => parse_unit!(rest, UP::Exbi, 2),
            [b'k' | b'K', rest @ ..] => parse_unit!(rest, UP::Kilo, 1),
            [b'm' | b'M', rest @ ..] => parse_unit!(rest, UP::Mega, 1),
            [b'g' | b'G', rest @ ..] => parse_unit!(rest, UP::Giga, 1),
            [b't' | b'T', rest @ ..] => parse_unit!(rest, UP::Mega, 1),
            [b'p' | b'P', rest @ ..] => parse_unit!(rest, UP::Peta, 1),
            [b'e' | b'E', rest @ ..] => parse_unit!(rest, UP::Exa, 1),
            [] => continue,
            _ => return Err(TokenizerError::UnexpectedCharacter(current)),
        };

        current += token.len();
        tokens.push(token);

        input = rest;
    }
    tokens.push(token!(Eof, current..current));

    Ok(tokens)
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

    #[test]
    fn test_tokenize() {
        let input = "42 + 42";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(42), 0..2),
                token!(Plus, 3..4),
                token!(Number(42), 5..7),
                token!(Eof, 7..7),
            ]
        );
    }

    #[test]
    fn test_tokenize_unit_prefix() {
        let input = "42Kib";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(42), 0..2),
                token!(Unit(Some(UnitPrefix::Kibi), Unit::Bit), 2..5),
                token!(Eof, 5..5),
            ]
        );
    }

    #[test]
    fn test_tokenize_unit() {
        let input = "42B";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(42), 0..2),
                token!(Unit(None, Unit::Byte), 2..3),
                token!(Eof, 3..3),
            ]
        );
    }

    #[test]
    fn test_tokenize_unit_prefix_and_unit() {
        let input = "42KiB";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(42), 0..2),
                token!(Unit(Some(UnitPrefix::Kibi), Unit::Byte), 2..5),
                token!(Eof, 5..5),
            ]
        );
    }

    #[test]
    fn test_tokenize_unit_and_unit_prefix() {
        let input = "42KB";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(42), 0..2),
                token!(Unit(Some(UnitPrefix::Kilo), Unit::Byte), 2..4),
                token!(Eof, 4..4),
            ]
        );
    }

    #[test]
    fn test_tokenize_long_expression() {
        let input = "12KiB / 02MiB * 42Gib -(42TiB + 42PiB)+ 42EiB";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                token!(Number(12), 0..2),
                token!(Unit(Some(UnitPrefix::Kibi), Unit::Byte), 2..5),
                token!(Slash, 6..7),
                token!(Number(2), 8..10),
                token!(Unit(Some(UnitPrefix::Mebi), Unit::Byte), 10..13),
                token!(Star, 14..15),
                token!(Number(42), 16..18),
                token!(Unit(Some(UnitPrefix::Gibi), Unit::Bit), 18..21),
                token!(Minus, 22..23),
                token!(LeftParen, 23..24),
                token!(Number(42), 24..26),
                token!(Unit(Some(UnitPrefix::Tebi), Unit::Byte), 26..29),
                token!(Plus, 30..31),
                token!(Number(42), 32..34),
                token!(Unit(Some(UnitPrefix::Pebi), Unit::Byte), 34..37),
                token!(RightParen, 37..38),
                token!(Plus, 38..39),
                token!(Number(42), 40..42),
                token!(Unit(Some(UnitPrefix::Exbi), Unit::Byte), 42..45),
                token!(Eof, 45..45),
            ]
        );
    }

    #[test]
    fn test_tokenize_single_digit() {
        let tokens = tokenize("0").unwrap();
        assert_eq!(tokens, vec![token!(Number(0), 0..1), token!(Eof, 1..1),]);
    }

    #[test]
    fn test_tokenize_invalid_input() {
        let res = tokenize("42 + 42x").unwrap_err();
        assert_eq!(res, TokenizerError::UnexpectedCharacter(7));

        let res = tokenize("0x").unwrap_err();
        assert_eq!(res, TokenizerError::InvalidDigit(2, ParseIntError::Empty));

        let res = tokenize("0b").unwrap_err();
        assert_eq!(res, TokenizerError::InvalidDigit(2, ParseIntError::Empty));

        let res = tokenize("0o").unwrap_err();
        assert_eq!(res, TokenizerError::InvalidDigit(2, ParseIntError::Empty));

        let res = tokenize("0xg").unwrap_err();
        assert_eq!(res, TokenizerError::InvalidDigit(2, ParseIntError::Empty));

        let res = tokenize("0a").unwrap_err();
        assert_eq!(res, TokenizerError::UnexpectedCharacter(1));

        let res = tokenize("ak").unwrap_err();
        assert_eq!(res, TokenizerError::UnexpectedCharacter(0));

        let res = tokenize("").unwrap_err();
        assert_eq!(res, TokenizerError::UnexpectedEndOfInput);
    }
}
