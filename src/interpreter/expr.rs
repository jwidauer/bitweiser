use std::fmt::Display;

use super::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Operator(OperatorExpr),
    Grouping(Box<Expr>),
    Literal { kind: Token, unit: Option<Token> },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Operator(expr) => write!(f, "{}", expr),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal { kind, unit } => {
                write!(f, "{}", kind)?;
                if let Some(unit) = unit {
                    write!(f, "{}", unit)?;
                }
                write!(f, "")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorExpr {
    ArithmeticOrLogical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    TypeCast {
        left: Box<Expr>,
        unit: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Display for OperatorExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorExpr::ArithmeticOrLogical {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator, left, right)
            }
            OperatorExpr::TypeCast { left, unit } => {
                write!(f, "(as {} {})", left, unit)
            }
            OperatorExpr::Unary { operator, right } => {
                write!(f, "({} {})", operator, right)
            }
        }
    }
}

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
