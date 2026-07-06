use std::{
    ffi::{OsStr, OsString},
    fs::{self, read_dir, read_to_string, write},
    path::{Path, PathBuf},
};

use crate::{lexer::Lexer, parser};

use anyhow::Context;

pub fn dump_expected() -> anyhow::Result<()> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");

    let entries = read_dir(format!("{cargo_dir}/fixtures/"))
        .with_context(|| format!("Failed to open directory '{cargo_dir}/fixtures/'"))?;

    for entry in entries {
        let file = entry.with_context(|| "Failed to read directory entry")?;
        let current_file_path = file.path();

        if current_file_path.is_file() && current_file_path.extension() == Some(OsStr::new("blorp"))
        {
            let content = read_to_string(&current_file_path).with_context(|| {
                format!("Failed to read file '{}'", current_file_path.display())
            })?;

            let tokens = Lexer::new(&current_file_path, &content).tokenize();
            let ast_str = match &tokens {
                Ok(t) => {
                    let ast = parser::Parser::new(t.clone(), &current_file_path).parse_program();
                    if let Err(e) = &ast {
                        eprintln!(
                            "Error while trying to dump into files: {e}\nWriting the error to the file:"
                        );
                    }
                    format!("{ast:#?}")
                }
                Err(e) => {
                    eprintln!(
                        "Error while trying to dump into files: {e}\nWriting the error to the file."
                    );
                    format!("{e:#?}")
                }
            };
            let tokens_str = format!("{tokens:#?}");

            let token_file_path = create_expected_by_ext(&current_file_path, ".tokens")?;
            write(&token_file_path, tokens_str)?;

            let parser_file_path = create_expected_by_ext(&current_file_path, ".ast")?;
            write(&parser_file_path, ast_str)?;

            println!("Writing to: {}", token_file_path.display());
            println!("Writing to: {}", parser_file_path.display());
        }
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
