use anyhow::Result;
use colored::Colorize;

mod format;
mod interpreter;

use format::as_bin;
use interpreter::{
    expr::{Expr, OperatorExprKind},
    Interpreter,
};

fn print_stats(num: u64) {
    let dec = "Decimal".green();
    println!("{dec}:\t{num}");

    let hex = "Hex".green();
    println!("{hex}:\t\t0x{num:X}");

    let oct = "Octal".green();
    println!("{oct}:\t\t0o{num:o}");

    let bin_str = "Binary".green();
    let bin = as_bin(num);
    println!("{bin_str}:\t\t{bin}");

    let dec_size_str = "Decimal Size".green();
    let dec_size = format::as_dec_size(num);
    println!("{dec_size_str}:\t{dec_size}");

    let bin_size_str = "Binary Size".green();
    let bin_size = format::as_bin_size(num);
    println!("{bin_size_str}:\t{bin_size}");
}

fn pretty_print(expr: &Expr) {
    match expr {
        Expr::Operator(expr) => match expr {
            OperatorExprKind::ArithmeticOrLogical {
                left,
                operator,
                right,
            } => {
                print!("({} ", operator);
                pretty_print(left);
                print!(" ");
                pretty_print(right);
                print!(")");
            }
            OperatorExprKind::TypeCast { left, unit } => {
                print!("(as ");
                pretty_print(left);
                print!(" {})", unit);
            }
            OperatorExprKind::Unary { operator, right } => {
                print!("({} ", operator);
                pretty_print(right);
                print!(")");
            }
        },
        Expr::Grouping { expression } => {
            print!("(group ");
            pretty_print(expression);
            print!(")");
        }
        Expr::Literal { kind, unit } => {
            print!("{}", kind);
            if let Some(unit) = unit {
                print!("{}", unit);
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();

    let input = &args[0];

    let interpreter = Interpreter::new();
    let value = interpreter.interpret(input)?;
    println!("{} = {}", input, value);

    Ok(())
}
