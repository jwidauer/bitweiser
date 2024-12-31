use clap::Parser;
use colored::Colorize;
use miette::{IntoDiagnostic, Result};

mod format;
mod interpreter;

use format::as_bin;
use interpreter::Interpreter;
use rustyline::error::ReadlineError;

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

struct Repl {
    interpreter: Interpreter,
}

impl Repl {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    fn run(&self) -> Result<()> {
        let mut rl = rustyline::DefaultEditor::new().into_diagnostic()?;
        println!("Welcome to the REPL! Type :h or :help for help.");
        loop {
            let readline = rl.readline(">> ");

            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str()).into_diagnostic()?;
                    self.eval_line(&line);
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn eval_line(&self, line: &str) {
        match line {
            ":q" | ":quit" => std::process::exit(0),
            ":h" | ":help" => {
                println!("Commands:");
                println!("  :q | :quit - Quit the REPL");
                println!("  :h | :help - Display this help message");
            }
            _ => self.eval_expr(line),
        }
    }

    fn eval_expr(&self, expr: &str) {
        match self
            .interpreter
            .interpret(expr)
            .map_err(miette::Report::new)
            .map_err(|e| e.with_source_code(expr.to_string()))
        {
            Ok(value) => println!("{expr} = {value}"),
            Err(e) => eprintln!("{e:?}"),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    expr: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let repl = Repl::new();
    match args.expr {
        Some(expr) => repl.eval_expr(&expr),
        None => repl.run()?,
    }

    Ok(())
}
