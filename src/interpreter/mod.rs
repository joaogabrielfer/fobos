use std::path::PathBuf;

use crate::interpreter::env::Env;

pub mod builtins;
pub mod env;
pub mod errors;
pub mod eval;
pub mod values;

pub struct Interpreter<'a> {
    env: Env<'a>,
    file_path: &'a PathBuf,
}

impl<'a> Interpreter<'a> {
    pub fn new(file_path: &'a PathBuf) -> Self {
        Self {
            env: env::Env::default(),
            file_path,
        }
    }
}
