use std::collections::HashMap;

use crate::{errors::TypeErrorKind, typechecker::ty::Type};

#[derive(Debug, Default)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("shoulnt pop global scope");
    }

    pub fn define(&mut self, name: String, ty: Type) {
        self.scopes
            .last_mut()
            .expect("should always have at least one scope")
            .insert(name, ty);
    }
    pub fn get(&self, name: &str) -> Result<Type, TypeErrorKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(TypeErrorKind::UndefinedVariable(name.to_string()))
    }
}
