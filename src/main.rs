use clap::{Parser, Subcommand};
use std::{fs::read_to_string, path::PathBuf, process::exit};

use blorp::lexer::Lexer;

#[derive(Parser)]
#[command(name = "blorp")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { path: PathBuf },
    Tokens { path: PathBuf },
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
        Commands::Run { path: _path } => {
            // let content = read_to_string(path)?;
            // let lexer = Lexer::new(content.as_str());
        }
        Commands::Tokens { path } => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(path, content.as_str()).tokenize()?;
            // println!("{tokens:#?}");
            for tk in tokens {
                println!("{:?}", tk.kind)
            }
        }
    }
    Ok(())
}
