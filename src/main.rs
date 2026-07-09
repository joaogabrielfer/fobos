use clap::{Parser, Subcommand};
use colored::Colorize;
use rustyline::DefaultEditor;
use std::{fs::read_to_string, path::PathBuf, process::exit};
use thiserror::Error;

use blorp::{
    dump::dump_expected,
    interpreter::{self, Interpreter},
    lexer::{self, Lexer},
    parser::{self},
    typechecker::TypeChecker,
};

#[derive(Parser)]
#[command(name = "blorp")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        path: PathBuf,
        #[arg(short = 't', long = "checker", default_value_t = false)]
        type_checker: bool,
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
        None => {
            let mut editor = DefaultEditor::new()?;
            let path = &PathBuf::from("repl");
            let stdout = std::io::stdout();
            let mut interpreter = Interpreter::new(path, stdout.lock());
            // let ast_arena = Arena::new();
            let mut c_c_pressed = false;
            loop {
                let prompt = ">> ".green().to_string();
                let mut line = match editor.readline(&prompt) {
                    Ok(line) => {
                        c_c_pressed = false;
                        line
                    }
                    Err(rustyline::error::ReadlineError::Interrupted) => {
                        if c_c_pressed {
                            return Ok(());
                        } else {
                            println!("(press 'C-c' again to exit the repl)");
                            c_c_pressed = true;
                            continue;
                        }
                    }
                    Err(rustyline::error::ReadlineError::Eof) => {
                        return Ok(());
                    }
                    Err(err) => return Err(err.into()),
                };

                let mut source = std::mem::take(&mut line);
                source.push('\n');
                let mut tokens = match Lexer::new(path, &source).tokenize() {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                tokens.insert(0, lexer::Token::new(lexer::TokenKind::Return));
                let ast = match parser::Parser::new(tokens, path).parse_program() {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                // let ast: Program = ast_arena.alloc(ast);
                let value = match interpreter.eval_program(ast) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                if interpreter::values::Value::Unit != value {
                    println!("{value}");
                }
            }
        }
        Some(Commands::Run { path, type_checker }) => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, &content).tokenize()?;
            let ast = parser::Parser::new(tokens, path).parse_program()?;
            let stdout = std::io::stdout();
            if *type_checker {
                let checked_program = TypeChecker::new(path.clone()).check_program(ast)?;
                Interpreter::new(path, stdout.lock()).eval_program(checked_program.program)?;
            } else {
                Interpreter::new(path, stdout.lock()).eval_program(ast)?;
            }
            Ok(())
        }
        Some(Commands::Tokens { path, only_kinds }) => {
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
        Some(Commands::Ast { path }) => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, &content).tokenize()?;
            let ast = parser::Parser::new(tokens, path).parse_program()?;
            println!("{ast:#?}");
            Ok(())
        }
        Some(Commands::GenerateExpected) => dump_expected(),
    }
}
