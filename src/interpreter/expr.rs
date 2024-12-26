use super::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Operator(OperatorExprKind),
    Grouping { expression: Box<Expr> },
    Literal { kind: Token, unit: Option<Token> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorExprKind {
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
