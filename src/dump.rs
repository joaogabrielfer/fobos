use std::{
    ffi::{OsStr, OsString},
    fs::{self, read_dir, read_to_string, write},
    path::{Path, PathBuf},
};

use crate::{interpreter::Interpreter, lexer::Lexer, parser};

use anyhow::Context;

pub fn dump_expected() -> anyhow::Result<()> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");

    let entries = read_dir(format!("{cargo_dir}/fixtures/"))
        .with_context(|| format!("Failed to open directory '{cargo_dir}/fixtures/'"))?;

    for entry in entries {
        let file = match entry {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to read directory entry: {e}");
                continue;
            }
        };

        let current_file_path = file.path();

        if !current_file_path.is_file() || current_file_path.extension() != Some(OsStr::new("fob"))
        {
            continue;
        }

        println!("Processing: {}", current_file_path.display());

        let content = match read_to_string(&current_file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read '{}': {e}", current_file_path.display());
                continue;
            }
        };

        let tokens = Lexer::new(&current_file_path, &content).tokenize();

        let tokens_str = format!("{tokens:#?}");

        let (ast_str, eval_str) = match tokens {
            Ok(tokens) => {
                let ast = parser::Parser::new(tokens, &current_file_path).parse_program();

                match ast {
                    Ok(program) => {
                        let ast_str = format!("{program:#?}");

                        let mut interpreter = Interpreter::new_buffered(&current_file_path);

                        let eval_result = interpreter.eval_program(program);
                        let output = interpreter.into_output_string();

                        let eval_str = match eval_result {
                            Ok(value) => {
                                if output.is_empty() {
                                    format!("result:\n{value:#?}\n")
                                } else {
                                    format!("output:\n{output}\nresult:\n{value:#?}\n")
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Eval error in '{}':\n{}",
                                    current_file_path.display(),
                                    e
                                );

                                format!("{e:#?}")
                            }
                        };

                        (ast_str, eval_str)
                    }

                    Err(e) => {
                        eprintln!("Parse error in '{}':\n{}", current_file_path.display(), e);

                        let err = format!("{e:#?}");

                        // AST is the parser error. Eval cannot run, so eval also stores the error.
                        (err.clone(), err)
                    }
                }
            }

            Err(e) => {
                eprintln!("Lexer error in '{}':\n{}", current_file_path.display(), e);

                let err = format!("{e:#?}");

                // Tokens are the lexer error. AST/eval cannot run.
                (err.clone(), err)
            }
        };

        let token_file_path = match create_expected_by_ext(&current_file_path, ".tokens") {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Could not create .tokens path for '{}': {e}",
                    current_file_path.display()
                );
                continue;
            }
        };

        let parser_file_path = match create_expected_by_ext(&current_file_path, ".ast") {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Could not create .ast path for '{}': {e}",
                    current_file_path.display()
                );
                continue;
            }
        };

        let eval_file_path = match create_expected_by_ext(&current_file_path, ".eval") {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Could not create .eval path for '{}': {e}",
                    current_file_path.display()
                );
                continue;
            }
        };

        if let Err(e) = write(&token_file_path, tokens_str) {
            eprintln!("Failed to write '{}': {e}", token_file_path.display());
            continue;
        }

        if let Err(e) = write(&parser_file_path, ast_str) {
            eprintln!("Failed to write '{}': {e}", parser_file_path.display());
            continue;
        }

        if let Err(e) = write(&eval_file_path, eval_str) {
            eprintln!("Failed to write '{}': {e}", eval_file_path.display());
            continue;
        }

        println!("  wrote {}", token_file_path.display());
        println!("  wrote {}", parser_file_path.display());
        println!("  wrote {}", eval_file_path.display());
    }

    Ok(())
}

pub fn create_expected_by_ext(file_path: &Path, extension: &str) -> anyhow::Result<PathBuf> {
    // here we would be iterating over the dir entries, so it would always have a parent
    let parent = file_path.parent().unwrap();

    // also i guess it wouldnt end in '..'
    let file_name = file_path.file_name().unwrap();

    let new_dir = parent.join("expected");
    fs::create_dir_all(&new_dir)?;

    let mut new_file_name = OsString::from(file_name);
    new_file_name.push(extension);

    Ok(new_dir.join(new_file_name))
}
