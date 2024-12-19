use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

use crate::unit_prefix::UnitPrefix;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Bit,
    Byte,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Bit => write!(f, "bit"),
            Unit::Byte => write!(f, "byte"),
        }
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
    Unit(Option<UnitPrefix>, Unit),
    Number(u64),

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
            TokenKind::Unit(prefix, unit) => {
                if let Some(prefix) = prefix {
                    write!(f, "{}", prefix)?;
                }
                write!(f, "{}", unit)
            }
            TokenKind::Number(num) => write!(f, "{}", num),
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
