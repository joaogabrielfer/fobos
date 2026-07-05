use anyhow::Context;
use std::fs::write;

use crate::lexer::Lexer;
use crate::parser;
use crate::parser::Program;
use crate::path_utils::create_expected_by_ext;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::fs::read_to_string;

pub fn dump_expected() -> anyhow::Result<()> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");

    let entries = read_dir(format!("{cargo_dir}/tests"))
        .with_context(|| format!("Failed to open directory '{cargo_dir}/tests/'"))?;

    for entry in entries {
        let file = entry.with_context(|| "Failed to read directory entry")?;
        let current_file_path = file.path();

        if current_file_path.is_file() && current_file_path.extension() == Some(OsStr::new("blorp"))
        {
            let content = read_to_string(&current_file_path).with_context(|| {
                format!("Failed to read file '{}'", current_file_path.display())
            })?;

            let tokens = match Lexer::new(&current_file_path, &content).tokenize() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{e}");
                    vec![]
                }
            };
            let ast = match parser::Parser::new(tokens.clone(), &current_file_path).parse_program()
            {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("{e}");
                    Program { statements: vec![] }
                }
            };

            let token_file_path = create_expected_by_ext(&current_file_path, ".tokens")?;
            write(&token_file_path, format!("{tokens:#?}"))?;

            let parser_file_path = create_expected_by_ext(&current_file_path, ".ast")?;
            write(&parser_file_path, format!("{ast:#?}"))?;

            println!("Writing expected tokens to: {}", token_file_path.display());
            println!("Writing expected tokens to: {}", parser_file_path.display());
        }
    }
    Ok(())
}
