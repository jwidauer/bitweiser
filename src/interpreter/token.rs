use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

use super::unit_prefix::UnitPrefix;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unit {
    Bit = 1,
    Byte = 8,
}

impl From<Unit> for u64 {
    fn from(unit: Unit) -> Self {
        unit as u64
    }
}

impl From<Unit> for f64 {
    fn from(unit: Unit) -> Self {
        unit as u64 as f64
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Bit => write!(f, "b"),
            Unit::Byte => write!(f, "B"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FullUnit(pub UnitPrefix, pub Unit);

impl FullUnit {
    pub fn new(prefix: UnitPrefix, unit: Unit) -> Self {
        Self(prefix, unit)
    }

    pub fn bit() -> Self {
        Self(UnitPrefix::None, Unit::Bit)
    }

    pub fn byte() -> Self {
        Self(UnitPrefix::None, Unit::Byte)
    }
}

impl From<FullUnit> for u64 {
    fn from(unit: FullUnit) -> Self {
        u64::from(unit.0) * u64::from(unit.1)
    }
}

impl PartialOrd for FullUnit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FullUnit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (u64::from(self.0) * self.1 as u64).cmp(&(u64::from(other.0) * other.1 as u64))
    }
}

impl Display for FullUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    // Single character tokens
    Minus,
    Plus,
    Star,
    Slash,
    LeftParen,
    RightParen,

    // Literals
    Unit(FullUnit),
    Integer(u64),

    // Keywords
    As,

    // End of file
    Eof,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Unit(unit) => write!(f, "{}", unit),
            TokenKind::Integer(num) => write!(f, "{}", num),
            TokenKind::As => write!(f, "as"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
    loc: Range<usize>,
}

macro_rules! token {
    ($kind:ident, $loc:expr) => {
        $crate::interpreter::token::Token::new($crate::interpreter::token::TokenKind::$kind, $loc)
    };
    ($kind:ident($($val:expr),+), $loc:expr) => {
        $crate::interpreter::token::Token::new(
            $crate::interpreter::token::TokenKind::$kind($($val),+),
            $loc,
        )
    };
}

pub(super) use token;

impl Token {
    pub fn new(kind: TokenKind, loc: Range<usize>) -> Self {
        Self { kind, loc }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn loc(&self) -> Range<usize> {
        self.loc.clone()
    }

    pub fn len(&self) -> usize {
        self.loc.len()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display_token() {
        let token = token!(Integer(42), 0..2);
        assert_eq!(format!("{}", token), "42");
    }

    #[test]
    fn test_display_token_kind() {
        assert_eq!(format!("{}", TokenKind::Minus), "-");
        assert_eq!(format!("{}", TokenKind::Plus), "+");
        assert_eq!(format!("{}", TokenKind::Star), "*");
        assert_eq!(format!("{}", TokenKind::Slash), "/");
        assert_eq!(format!("{}", TokenKind::LeftParen), "(");
        assert_eq!(format!("{}", TokenKind::RightParen), ")");
        assert_eq!(
            format!(
                "{}",
                TokenKind::Unit(FullUnit(UnitPrefix::Kilo, Unit::Byte))
            ),
            "kB"
        );
        assert_eq!(format!("{}", TokenKind::Integer(42)), "42");
        assert_eq!(format!("{}", TokenKind::As), "as");
        assert_eq!(format!("{}", TokenKind::Eof), "EOF");
    }

    #[test]
    fn test_display_unit() {
        assert_eq!(format!("{}", Unit::Bit), "b");
        assert_eq!(format!("{}", Unit::Byte), "B");
    }

    #[test]
    fn test_display_full_unit() {
        assert_eq!(format!("{}", FullUnit(UnitPrefix::Kilo, Unit::Byte)), "kB");
    }

    #[test]
    fn test_display_unit_prefix() {
        assert_eq!(format!("{}", UnitPrefix::None), "");
        assert_eq!(format!("{}", UnitPrefix::Kilo), "k");
        assert_eq!(format!("{}", UnitPrefix::Mega), "M");
        assert_eq!(format!("{}", UnitPrefix::Giga), "G");
        assert_eq!(format!("{}", UnitPrefix::Tera), "T");
        assert_eq!(format!("{}", UnitPrefix::Peta), "P");
        assert_eq!(format!("{}", UnitPrefix::Exa), "E");
        assert_eq!(format!("{}", UnitPrefix::Kibi), "ki");
        assert_eq!(format!("{}", UnitPrefix::Mebi), "Mi");
        assert_eq!(format!("{}", UnitPrefix::Gibi), "Gi");
        assert_eq!(format!("{}", UnitPrefix::Tebi), "Ti");
        assert_eq!(format!("{}", UnitPrefix::Pebi), "Pi");
        assert_eq!(format!("{}", UnitPrefix::Exbi), "Ei");
    }

    #[test]
    fn test_from_unit() {
        assert_eq!(u64::from(Unit::Bit), 1);
        assert_eq!(u64::from(Unit::Byte), 8);
    }

    #[test]
    fn test_from_full_unit() {
        assert_eq!(u64::from(FullUnit(UnitPrefix::Kilo, Unit::Byte)), 8000);
    }

    #[test]
    fn test_ord_full_unit() {
        assert!(FullUnit(UnitPrefix::Kilo, Unit::Byte) < FullUnit(UnitPrefix::Mega, Unit::Byte));
        assert!(FullUnit(UnitPrefix::Kilo, Unit::Byte) > FullUnit(UnitPrefix::Kilo, Unit::Bit));
    }
}
