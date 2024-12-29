use miette::Diagnostic;
use thiserror::Error;

use super::{
    expr::{Expr, OperatorExpr as OE},
    lexer::{LexError, Lexer},
    token::{Token, TokenKind},
    SyntaxErrorKind,
};

// Grammar:
// expression   -> term EOF ;
// term         -> factor ( ( "-" | "+" ) factor )* ;
// factor       -> unitcast ( ( "/" | "*" ) unitcast )* ;
// unitcast     -> unary ( "as" UNIT )? ;
// unary        -> "-" unary | primary ;
// primary      -> NUMBER ( UNIT )? | "(" expression ")" ;
//
// NUMBER   -> BINARY | OCTAL | DECIMAL | HEX ;
// BINARY   -> "0b" [01]+ ;
// OCTAL    -> "0o" [0-7]+ ;
// DECIMAL  -> [0-9]+ ;
// HEX      -> "0x" [0-9a-fA-F]+ ;
//
// UNIT     -> UNITPREFIX? "b" | "B" ;
// UNITPREFIX -> DECUNITPREFIX | BINUNITPREFIX ;
// BINUNITPREFIX -> DECUNITPREFIX "i" ;
// DECUNITPREFIX -> "k" | "m" | "g" | "t" | "p" | "e" | "K" | "M" | "G" | "T" | "P" | "E" ;

#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum ParseErrorKind {
    #[error("Expected {0}")]
    UnexpectedToken(&'static str),
    #[error("Expected expression")]
    ExpectedExpression,
    #[error("Expected end of expression")]
    ExpectedEof,
    #[error("Expected unit")]
    ExpectedUnit,
}

#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
#[error("{}, found '{}'", kind, token)]
pub struct ParseError {
    kind: ParseErrorKind,
    #[label = "here"]
    token: Token,
}

impl ParseError {
    fn new(kind: ParseErrorKind, token: Token) -> Self {
        Self { kind, token }
    }

    #[inline]
    pub fn token(&self) -> &Token {
        &self.token
    }
}

macro_rules! error {
    ($kind:ident, $token:expr) => {
        ParseError::new(ParseErrorKind::$kind, $token)
    };
    ($kind:ident($($arg:expr),+), $token:expr) => {
        ParseError::new(ParseErrorKind::$kind($($arg),+), $token)
    };
}

pub struct Parser<'a> {
    iter: std::iter::Peekable<Lexer<'a>>,
}

macro_rules! bump_if {
    ($self:ident, $($kind:ident),+) => {
        matches!($self.peek()?, $(Some(TokenKind::$kind))|+).then(|| $self.bump())
    };
    ($self:ident, $($kind:ident(_)),+) => {
        matches!($self.peek()?, $(Some(TokenKind::$kind(_)))|+).then(|| $self.bump())
    };
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            iter: lexer.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, SyntaxErrorKind> {
        let expr = self.expression()?;

        if bump_if!(self, Eof).is_some() {
            return Ok(expr);
        }

        Err(error!(ExpectedEof, self.bump()).into())
    }

    fn expression(&mut self) -> Result<Expr, SyntaxErrorKind> {
        self.term()
    }

    fn term(&mut self) -> Result<Expr, SyntaxErrorKind> {
        let mut expr = self.factor()?;

        while let Some(operator) = bump_if!(self, Minus, Plus) {
            let right = Box::new(self.factor()?);
            expr = Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, SyntaxErrorKind> {
        let mut expr = self.type_cast()?;

        while let Some(operator) = bump_if!(self, Slash, Star) {
            let right = Box::new(self.type_cast()?);
            expr = Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn type_cast(&mut self) -> Result<Expr, SyntaxErrorKind> {
        let mut expr = self.unary()?;

        if bump_if!(self, As).is_some() {
            let unit = self.consume_unit()?;

            expr = Expr::Operator(OE::TypeCast {
                left: Box::new(expr),
                unit,
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, SyntaxErrorKind> {
        if let Some(operator) = bump_if!(self, Minus) {
            let right = Box::new(self.unary()?);
            return Ok(Expr::Operator(OE::Unary { operator, right }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, SyntaxErrorKind> {
        match self.peek()? {
            Some(TokenKind::Integer(_)) => {
                let kind = self.bump();
                let unit = bump_if!(self, Unit(_));
                return Ok(Expr::Literal { kind, unit });
            }
            Some(TokenKind::LeftParen) => {
                self.bump();
                let expression = Box::new(self.expression()?);
                self.consume_r_paren()?;
                return Ok(Expr::Grouping(expression));
            }
            _ => {}
        }

        Err(error!(ExpectedExpression, self.bump()).into())
    }

    fn bump(&mut self) -> Token {
        self.iter.next().unwrap().unwrap().clone()
    }

    fn peek(&mut self) -> Result<Option<TokenKind>, LexError> {
        self.iter
            .peek()
            .map(ToOwned::to_owned)
            .transpose()
            .map(|o| o.map(|t| t.kind()))
    }

    fn consume_unit(&mut self) -> Result<Token, SyntaxErrorKind> {
        bump_if!(self, Unit(_)).ok_or(error!(ExpectedUnit, self.bump()).into())
    }

    fn consume_r_paren(&mut self) -> Result<Token, SyntaxErrorKind> {
        bump_if!(self, RightParen).ok_or(error!(UnexpectedToken(")"), self.bump()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{
        lexer::Lexer,
        token::{token, FullUnit, Unit},
        unit_prefix::UnitPrefix,
    };

    macro_rules! parse {
        ($input:expr) => {
            Parser::new(Lexer::new($input)).parse()
        };
    }

    #[test]
    fn test_parser_single_token() {
        let expr = parse!("1234").unwrap();
        assert_eq!(
            expr,
            Expr::Literal {
                kind: token!(Integer(1234), 0..4),
                unit: None
            }
        );
    }

    #[test]
    fn test_parser_binary_expr() {
        let expr = parse!("1234 + 5678").unwrap();
        assert_eq!(
            expr,
            Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(Expr::Literal {
                    kind: token!(Integer(1234), 0..4),
                    unit: None
                }),
                operator: token!(Plus, 5..6),
                right: Box::new(Expr::Literal {
                    kind: token!(Integer(5678), 7..11),
                    unit: None
                })
            })
        );
    }

    #[test]
    fn test_parser_binary_expr_with_precedence() {
        let expr = parse!("1234 * 5678 + 91011").unwrap();
        assert_eq!(
            expr,
            Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(Expr::Operator(OE::ArithmeticOrLogical {
                    left: Box::new(Expr::Literal {
                        kind: token!(Integer(1234), 0..4),
                        unit: None
                    }),
                    operator: token!(Star, 5..6),
                    right: Box::new(Expr::Literal {
                        kind: token!(Integer(5678), 7..11),
                        unit: None
                    }),
                })),
                operator: token!(Plus, 12..13),
                right: Box::new(Expr::Literal {
                    kind: token!(Integer(91011), 14..19),
                    unit: None
                })
            })
        );
    }

    #[test]
    fn test_parser_nested_binary_expr() {
        let expr = parse!("1234 + 5678 * 91011 / 121314").unwrap();
        assert_eq!(
            expr,
            Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(Expr::Literal {
                    kind: token!(Integer(1234), 0..4),
                    unit: None
                }),
                operator: token!(Plus, 5..6),
                right: Box::new(Expr::Operator(OE::ArithmeticOrLogical {
                    left: Box::new(Expr::Operator(OE::ArithmeticOrLogical {
                        left: Box::new(Expr::Literal {
                            kind: token!(Integer(5678), 7..11),
                            unit: None
                        }),
                        operator: token!(Star, 12..13),
                        right: Box::new(Expr::Literal {
                            kind: token!(Integer(91011), 14..19),
                            unit: None
                        }),
                    })),
                    operator: token!(Slash, 20..21),
                    right: Box::new(Expr::Literal {
                        kind: token!(Integer(121314), 22..28),
                        unit: None
                    })
                }))
            })
        );
    }

    #[test]
    fn test_parser_unary_expr() {
        let expr = parse!("-1234").unwrap();
        assert_eq!(
            expr,
            Expr::Operator(OE::Unary {
                operator: token!(Minus, 0..1),
                right: Box::new(Expr::Literal {
                    kind: token!(Integer(1234), 1..5),
                    unit: None
                })
            })
        );
    }

    #[test]
    fn test_parser_grouped_expr() {
        let expr = parse!("(1234 + 5678)").unwrap();
        assert_eq!(
            expr,
            Expr::Grouping(Box::new(Expr::Operator(OE::ArithmeticOrLogical {
                left: Box::new(Expr::Literal {
                    kind: token!(Integer(1234), 1..5),
                    unit: None
                }),
                operator: token!(Plus, 6..7),
                right: Box::new(Expr::Literal {
                    kind: token!(Integer(5678), 8..12),
                    unit: None
                })
            })))
        );
    }

    #[test]
    fn test_parser_type_cast_expr() {
        let expr = parse!("1234 as KiB").unwrap();
        assert_eq!(
            expr,
            Expr::Operator(OE::TypeCast {
                left: Box::new(Expr::Literal {
                    kind: token!(Integer(1234), 0..4),
                    unit: None
                }),
                unit: token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Byte)), 8..11),
            })
        );
    }

    #[test]
    fn test_parser_int_literal_with_unit() {
        let expr = parse!("1234 KiB").unwrap();
        assert_eq!(
            expr,
            Expr::Literal {
                kind: token!(Integer(1234), 0..4),
                unit: Some(token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Byte)), 5..8)),
            }
        );
    }
}
