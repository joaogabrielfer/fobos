use std::{cell::RefCell, path::PathBuf, rc::Rc};

use anyhow::Result;

use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    interpreter::{
        env::{Env, EnvRef},
        values::BuiltinFunction,
    },
    source::Span,
};

pub mod builtins;
pub mod env;
pub mod eval;
pub mod values;

pub struct Interpreter<'a, W: std::io::Write> {
    env: EnvRef,
    file_path: &'a PathBuf,
    output: W,
}

impl<'a, W: std::io::Write> Interpreter<'a, W> {
    pub fn new(file_path: &'a PathBuf, output: W) -> Self {
        let env = Rc::new(RefCell::new(Env::default()));
        env.borrow_mut().define(
            "echo".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Echo),
        );
        Self {
            env,
            file_path,
            output,
        }
    }
    pub fn new_with_env(file_path: &'a PathBuf, env: EnvRef, output: W) -> Self {
        Self {
            env,
            file_path,
            output,
        }
    }
    fn _write_output(&mut self, value: impl std::fmt::Display) -> Result<(), Box<RuntimeError>> {
        write!(self.output, "{value}").map_err(|err| self.runtime_io_error(err))
    }
    fn writeln_output(&mut self, value: impl std::fmt::Display) -> Result<(), Box<RuntimeError>> {
        writeln!(self.output, "{value}").map_err(|err| self.runtime_io_error(err))
    }

    fn runtime_io_error(&self, err: std::io::Error) -> Box<RuntimeError> {
        self.error_at(Span::dummy(), RuntimeErrorKind::IoError(err.to_string()))
    }
}

impl<'a> Interpreter<'a, Vec<u8>> {
    pub fn new_buffered(file_path: &'a PathBuf) -> Self {
        Self::new(file_path, Vec::new())
    }

    pub fn into_output_string(self) -> String {
        String::from_utf8(self.output).expect("interpreter output should be valid UTF-8")
    }
}
