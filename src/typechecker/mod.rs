use std::path::PathBuf;

use crate::{
    ast::Program,
    errors::TypeError,
    module::ModuleInterface,
    typechecker::{env::TypeEnv, ty::Type},
};

pub mod check;
pub mod env;
pub mod ty;

pub struct CheckedProgram {
    pub program: Program,
}

pub struct CheckedModule {
    pub program: CheckedProgram,
    pub interface: ModuleInterface,
    pub constants: std::collections::HashMap<String, crate::interpreter::values::Value>,
}

pub struct TypeChecker {
    file_path: PathBuf,
    env: TypeEnv,
    current_function_return: Option<Type>,
    module_interfaces: std::collections::HashMap<crate::module::ModuleId, ModuleInterface>,
}

impl TypeChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            env: TypeEnv::new(),
            current_function_return: None,
            module_interfaces: std::collections::HashMap::new(),
        }
    }
}

pub type TypeResult<A> = Result<A, Box<TypeError>>;
