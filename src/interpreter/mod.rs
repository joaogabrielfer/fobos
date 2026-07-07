use std::path::PathBuf;

use crate::interpreter::{env::Env, values::BuiltinFunction};

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
        let mut env = Env::default();
        env.define(
            "echo",
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Echo),
        );
        Self { env, file_path }
    }
    pub fn new_with_env(file_path: &'a PathBuf, env: Env<'a>) -> Self {
        Self { env, file_path }
    }
}
