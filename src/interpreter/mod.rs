pub mod expr;
pub mod lexer;
mod num;
pub mod parser;
pub mod unit_prefix;
pub mod value;

#[macro_use]
mod token;

use anyhow::Result;
use expr::Expr;
use value::Value;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, input: &str) -> Result<Value> {
        let lexer = lexer::Lexer::new(input);
        let mut parser = parser::Parser::new(lexer);
        let expr = parser.parse()?;
        evaluate(&expr)
    }
}

fn evaluate(expr: &Expr) -> Result<Value> {
    use expr::OperatorExpr as OE;
    use token::TokenKind as TK;

    match expr {
        Expr::Operator(expr) => match expr {
            OE::ArithmeticOrLogical {
                left,
                operator,
                right,
            } => {
                let left = evaluate(left)?;
                let right = evaluate(right)?;
                match operator.kind() {
                    TK::Plus => Ok(left + right),
                    TK::Minus => Ok(left - right),
                    TK::Star => left.try_mul(right),
                    TK::Slash => left.try_div(right),
                    k => unreachable!("Invalid binary operator: {:?}", k),
                }
            }
            OE::TypeCast { left, unit } => {
                let left = evaluate(left)?;
                let unit = match unit.kind() {
                    TK::Unit(unit) => unit,
                    u => unreachable!("Invalid unit: {:?}", u),
                };
                Ok(left.convert_to(unit))
            }
            OE::Unary { operator, right } => {
                let right = evaluate(right)?;
                match operator.kind() {
                    TK::Minus => Ok(-right),
                    TK::Plus => Ok(right),
                    k => unreachable!("Invalid unary operator: {:?}", k),
                }
            }
        },
        Expr::Grouping(expr) => evaluate(expr),
        Expr::Literal { kind, unit } => match kind.kind() {
            TK::Integer(num) => {
                let value = num as f64;
                let unit = unit.as_ref().map(|u| match u.kind() {
                    TK::Unit(unit) => unit,
                    k => unreachable!("Invalid unit: {:?}", k),
                });
                Ok(Value::new(value, unit))
            }
            k => unreachable!("Invalid literal: {:?}", k),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::token::{FullUnit, Unit};
    use crate::interpreter::unit_prefix::UnitPrefix;

    #[test]
    fn test_interpreter() {
        let interpreter = Interpreter::new();
        let value = interpreter.interpret("1 + 2").unwrap();
        assert_eq!(value.value(), 3.0);
        assert_eq!(value.unit(), None);

        let value = interpreter.interpret("1 + 2 B").unwrap();
        assert_eq!(value.value(), 3.0);
        assert_eq!(value.unit(), Some(FullUnit::byte()));

        let value = interpreter.interpret("1 + 2 KiB").unwrap();
        assert_eq!(value.value(), 3.0);
        assert_eq!(
            value.unit(),
            Some(FullUnit::new(UnitPrefix::Kibi, Unit::Byte))
        );

        let value = interpreter.interpret("1 + 2 KiB + 3 MiB").unwrap();
        assert_eq!(value.value(), 3.0 + 3.0 * 1024.0);
        assert_eq!(
            value.unit(),
            Some(FullUnit::new(UnitPrefix::Kibi, Unit::Byte))
        );

        let value = interpreter.interpret("1 + 2 KiB + 3 MiB + 4 GiB").unwrap();
        assert_eq!(value.value(), 3.0 + 3.0 * 1024.0 + 4.0 * 1024.0 * 1024.0);
        assert_eq!(
            value.unit(),
            Some(FullUnit::new(UnitPrefix::Kibi, Unit::Byte))
        );

        let value = interpreter
            .interpret("1 + 2 KiB + 3 MiB + 4 GiB + 5 TiB")
            .unwrap();
        assert_eq!(
            value.value(),
            3.0 + 3.0 * 1024.0 + 4.0 * 1024.0 * 1024.0 + 5.0 * 1024.0 * 1024.0 * 1024.0
        );
        assert_eq!(
            value.unit(),
            Some(FullUnit::new(UnitPrefix::Kibi, Unit::Byte))
        );
    }
}
