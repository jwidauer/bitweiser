use anyhow::Result;
use thiserror::Error;

use super::{
    expr::Expr,
    token::{Token, TokenKind},
};

// Grammar:
// expression   -> term EOF ;
// term         -> factor ( ( "-" | "+" ) factor )* ;
// factor       -> unitcast ( ( "/" | "*" ) unitcast )* ;
// unitcast     -> unary ( "as" UNIT )? ;
// unary        -> "-" unary | primary ;
// primary      -> NUMBER ( UNIT )? | "(" expression ")" | EOF ;
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
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    iter: std::iter::Peekable<std::slice::Iter<'a, Token>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            iter: tokens.iter().peekable(),
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr> {
        let expr = self.term()?;

        if self.is_at_end() {
            return Ok(expr);
        }

        // Err(ParserError::ExpectedEof)
        anyhow::bail!("Expected end of expression.")
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while matches!(self.peek(), Some(TokenKind::Minus) | Some(TokenKind::Plus)) {
            let operator = self.iter.next().unwrap().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        // while self.match_token(&[TokenKind::Minus, TokenKind::Plus]) {
        //     let operator = self.previous();
        //     let right = self.factor()?;
        //     expr = Expr::Binary {
        //         left: Box::new(expr),
        //         operator,
        //         right: Box::new(right),
        //     };
        // }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, anyhow::Error> {
        let mut expr = self.unitcast()?;

        while matches!(self.peek(), Some(TokenKind::Slash) | Some(TokenKind::Star)) {
            let operator = self.iter.next().unwrap().clone();
            let right = self.unitcast()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        // while self.match_token(&[TokenKind::Slash, TokenKind::Star]) {
        //     let operator = self.previous();
        //     let right = self.unitcast()?;
        //     expr = Expr::Binary {
        //         left: Box::new(expr),
        //         operator,
        //         right: Box::new(right),
        //     };
        // }

        Ok(expr)
    }

    fn unitcast(&mut self) -> Result<Expr, anyhow::Error> {
        let mut expr = self.unary()?;

        if matches!(self.peek(), Some(TokenKind::As)) {
            let as_op = self.iter.next().unwrap().clone();
            let unit = self.consume_unit()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: as_op,
                right: Box::new(Expr::Literal { value: unit }),
            };
        }
        // if self.match_token(&[TokenKind::As]) {
        //     let as_op = self.previous();
        //     let unit = self.consume_unit()?;

        //     expr = Expr::Binary {
        //         left: Box::new(expr),
        //         operator: as_op,
        //         right: Box::new(Expr::Literal { value: unit }),
        //     };
        // }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, anyhow::Error> {
        if matches!(self.peek(), Some(TokenKind::Minus)) {
            let operator = self.iter.next().unwrap().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        // if self.match_token(&[TokenKind::Minus]) {
        //     let operator = self.previous();
        //     let right = self.unary()?;
        //     return Ok(Expr::Unary {
        //         operator,
        //         right: Box::new(right),
        //     });
        // }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(TokenKind::Number(_)) => {
                let token = self.iter.next().unwrap().clone();
                return Ok(Expr::Literal { value: token });
            }
            Some(TokenKind::LeftParen) => {
                self.iter.next();
                let expr = self.expression()?;
                if !matches!(self.next(), Some(TokenKind::RightParen)) {
                    anyhow::bail!("Expected ')' after expression.")
                }
                return Ok(Expr::Grouping {
                    expression: Box::new(expr),
                });
            }
            _ => {}
        }

        anyhow::bail!("Expected expression")
    }

    fn next(&mut self) -> Option<TokenKind> {
        self.iter.next().map(|t| t.kind())
    }

    // fn primary(&mut self) -> Result<Expr, anyhow::Error> {
    //     if self.match_token(&[TokenKind::Number(1234)]) {
    //         return Ok(Expr::Literal {
    //             value: self.previous(),
    //         });
    //     }

    //     if self.match_token(&[TokenKind::LeftParen]) {
    //         let expr = self.expression()?;
    //         self.consume(&TokenKind::RightParen, "Expect ')' after expression.")?;
    //         return Ok(Expr::Grouping {
    //             expression: Box::new(expr),
    //         });
    //     }

    //     Err(self.error(self.peek(), "Expect expression"))
    // }

    fn consume_unit(&mut self) -> Result<Token, anyhow::Error> {
        if matches!(self.peek(), Some(TokenKind::Unit(_, _))) {
            return Ok(self.iter.next().unwrap().clone());
        }

        anyhow::bail!("Expected unit.")
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.iter.peek().map(|t| t.kind())
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().map(|k| k == kind).unwrap_or(false)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.current].kind() == TokenKind::Eof
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
