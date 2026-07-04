use clap::{Parser, Subcommand};
use std::{fs::read_to_string, path::PathBuf};

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path } => {
            // let content = read_to_string(path)?;
            // let lexer = Lexer::new(content.as_str());
        }
        Commands::Tokens { path } => {
            let content = read_to_string(path)?;
            let tokens = Lexer::new(content.as_str()).tokenize()?;
            for tk in tokens {
                println!("TOKEN: {tk:?}");
            }
        }
    }
    Ok(())
}
