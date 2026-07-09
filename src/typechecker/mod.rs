use std::path::PathBuf;

use crate::{
    ast::Program,
    errors::TypeError,
    typechecker::{env::TypeEnv, ty::Type},
};

pub mod check;
pub mod env;
pub mod ty;

pub struct CheckedProgram {
    pub program: Program,
}

pub struct TypeChecker {
    file_path: PathBuf,
    env: TypeEnv,
    current_function_return: Option<Type>,
}

impl TypeChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            env: TypeEnv::new(),
            current_function_return: None,
        }
    }
}

pub type TypeResult<A> = Result<A, Box<TypeError>>;
