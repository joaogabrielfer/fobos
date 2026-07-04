use anyhow::Context;
use clap::{Parser, Subcommand};
use std::{
    ffi::OsStr,
    fs::{read_dir, read_to_string, write},
    path::PathBuf,
    process::exit,
};
use thiserror::Error;

use blorp::lexer::Lexer;

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
    GenerateExpected,
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
    let cargo_dir = env!("CARGO_MANIFEST_DIR");

    match &cli.command {
        Commands::Run { path: _path } => {
            // let content = read_to_string(path)?;
            // let lexer = Lexer::new(content.as_str());
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
        Commands::GenerateExpected => {
            let entries = read_dir(format!("{cargo_dir}/tests"))
                .with_context(|| format!("Failed to open directory '{cargo_dir}/tests/'"))?;

            for entry in entries {
                let file = entry.with_context(|| "Failed to read directory entry")?;
                let current_file_path = file.path();

                if current_file_path.is_file()
                    && current_file_path.extension() != Some(OsStr::new("expected"))
                {
                    let content = read_to_string(&current_file_path).with_context(|| {
                        format!("Failed to read file '{}'", current_file_path.display())
                    })?;

                    let tokens = Lexer::new(&current_file_path, &content).tokenize()?;

                    let mut expected_file_path = current_file_path.clone();
                    expected_file_path.as_mut_os_string().push(".expected");

                    println!(
                        "Writing expected tokens to: {}",
                        expected_file_path.display()
                    );

                    write(expected_file_path, format!("{tokens:#?}"))?;
                }
            }
            Ok(())
        }
    }
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(
        "Path {0:?} is not a directory. Please provide a directory for generating the expected results"
    )]
    GenerateTestPathNotDir(PathBuf),
}
