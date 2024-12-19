use anyhow::Result;
use colored::Colorize;

mod format;
mod interpreter;
mod unit_prefix;

use format::as_bin;
use interpreter::expr::Expr;

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
        Expr::Binary {
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
        Expr::Unary { operator, right } => {
            print!("({} ", operator);
            pretty_print(right);
            print!(")");
        }
        Expr::Grouping { expression } => {
            print!("(group ");
            pretty_print(expression);
            print!(")");
        }
        Expr::Literal { value } => {
            print!("{}", value);
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();

    let tokens = interpreter::lexer::tokenize(&args[0])?;
    println!("{:?}", tokens);
    let mut parser = interpreter::parser::Parser::new(&tokens);
    let expr = parser.parse()?;

    pretty_print(&expr);
    println!();

    Ok(())
}
