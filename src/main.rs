use std::io::Write;

use clap::Parser;
use colored::Colorize;
use miette::{IntoDiagnostic, Result};

mod format;
mod interpreter;

use format::as_bin;
use interpreter::Interpreter;

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

fn repl() -> Result<()> {
    let interpreter = Interpreter::new();

    loop {
        print!("> ");
        std::io::stdout().flush().into_diagnostic()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).into_diagnostic()?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            ":q" | ":quit" => break,
            ":h" | ":help" => {
                println!("Commands:");
                println!("  :q | :quit - Quit the REPL");
                println!("  :h | :help - Display this help message");
                continue;
            }
            _ => match interpreter
                .interpret(input)
                .map_err(miette::Report::new)
                .map_err(|e| e.with_source_code(input.to_string()))
            {
                Ok(value) => println!("{input} = {value}"),
                Err(e) => eprintln!("{e:?}"),
            },
        }
    }

    Ok(())
}

fn eval_expr(expr: &str) -> Result<()> {
    let interpreter = Interpreter::new();
    let value = interpreter
        .interpret(expr)
        .map_err(miette::Report::new)
        .map_err(|e| e.with_source_code(expr.to_string()))?;
    println!("{expr} = {value}");

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    expr: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.expr {
        Some(expr) => eval_expr(&expr)?,
        None => repl()?,
    }

    Ok(())
}
