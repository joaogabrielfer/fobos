use clap::{Parser, Subcommand};
use std::{fs::read_to_string, path::PathBuf, process::exit};
use thiserror::Error;

use blorp::{
    dump::dump_expected,
    interpreter::Interpreter,
    lexer::Lexer,
    parser::{self},
};

#[derive(Parser)]
#[command(name = "blorp")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        path: PathBuf,
    },
    Tokens {
        path: PathBuf,
        #[arg(short = 'k', long = "kinds", default_value_t = false)]
        only_kinds: bool,
    },
    Ast {
        path: PathBuf,
    },
    GenerateExpected,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(
        "Path {0:?} is not a directory. Please provide a directory for generating the expected results"
    )]
    GenerateTestPathNotDir(PathBuf),
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run { path } => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, &content).tokenize()?;
            let ast = parser::Parser::new(tokens, path).parse_program()?;
            let value = Interpreter::new(path).eval_program(&ast)?;
            println!("{value}");
            Ok(())
        }
        Commands::Tokens { path, only_kinds } => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, &content).tokenize()?;
            if *only_kinds {
                for tk in tokens {
                    println!("{:?}", tk.kind)
                }
            } else {
                println!("{tokens:#?}");
            }
            Ok(())
        }
        Commands::Ast { path } => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, &content).tokenize()?;
            let ast = parser::Parser::new(tokens, path).parse_program()?;
            println!("{ast:#?}");
            Ok(())
        }
        Commands::GenerateExpected => dump_expected(),
    }
}
