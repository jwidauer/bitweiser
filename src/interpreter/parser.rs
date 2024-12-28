use anyhow::Result;
use thiserror::Error;

use super::{
    expr::{Expr, OperatorExpr as OE},
    lexer::{Lexer, LexerError},
    token::{Token, TokenKind},
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

#[derive(Debug, Clone, PartialEq, Error)]
enum ParserError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(Token),
    #[error("Expected end of expression.")]
    ExpectedEof,
    #[error("Expected unit.")]
    ExpectedUnit,
    #[error(transparent)]
    LexerError(#[from] LexerError),
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

    pub fn parse(&mut self) -> Result<Expr> {
        let expr = self.expression()?;

        if bump_if!(self, Eof).is_some() {
            return Ok(expr);
        }

        anyhow::bail!("Expected end of expression, found {}.", self.bump().kind());
    }

    fn expression(&mut self) -> Result<Expr> {
        self.term()
    }

    fn term(&mut self) -> Result<Expr> {
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

    fn factor(&mut self) -> Result<Expr> {
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

    fn type_cast(&mut self) -> Result<Expr> {
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

    fn unary(&mut self) -> Result<Expr> {
        if let Some(operator) = bump_if!(self, Minus) {
            let right = Box::new(self.unary()?);
            return Ok(Expr::Operator(OE::Unary { operator, right }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.peek()? {
            Some(TokenKind::Integer(_)) => {
                let kind = self.bump();
                let unit = bump_if!(self, Unit(_));
                return Ok(Expr::Literal { kind, unit });
            }
            Some(TokenKind::LeftParen) => {
                self.bump();
                let expression = Box::new(self.expression()?);
                if bump_if!(self, RightParen).is_none() {
                    anyhow::bail!("Expected ')' after expression.")
                }
                return Ok(Expr::Grouping(expression));
            }
            _ => {}
        }

        anyhow::bail!("Expected expression")
    }

    fn bump(&mut self) -> Token {
        self.iter.next().unwrap().unwrap().clone()
    }

    fn peek(&mut self) -> Result<Option<TokenKind>, LexerError> {
        self.iter
            .peek()
            .map(ToOwned::to_owned)
            .transpose()
            .map(|o| o.map(|t| t.kind()))
    }

    fn consume_unit(&mut self) -> Result<Token, ParserError> {
        bump_if!(self, Unit(_)).ok_or(ParserError::ExpectedUnit)
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

    #[test]
    fn test_parser_single_token() {
        let input = "1234";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "1234 + 5678";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "1234 * 5678 + 91011";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "1234 + 5678 * 91011 / 121314";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "-1234";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "(1234 + 5678)";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "1234 as KiB";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
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
        let input = "1234 KiB";
        let mut parser = Parser::new(Lexer::new(input));
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Literal {
                kind: token!(Integer(1234), 0..4),
                unit: Some(token!(Unit(FullUnit(UnitPrefix::Kibi, Unit::Byte)), 5..8)),
            }
        );
    }
}
