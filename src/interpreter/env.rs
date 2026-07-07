use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{errors::RuntimeErrorKind, interpreter::values::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    scopes: Vec<HashMap<String, Binding>>,
}

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub mutable: bool,
    pub value: Value,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
}

impl Env {
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    pub fn get(&self, name: String) -> Result<Value, RuntimeErrorKind> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name) {
                return Ok(scope.get(&name).unwrap().clone().value);
            }
        }
        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }
    pub fn define(&mut self, name: String, mutable: bool, value: Value) {
        self.scopes
            .last_mut()
            .expect("scopes should always have at least one entry")
            .insert(name, Binding { mutable, value });
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeErrorKind> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(key) = scope.get_mut(&name) {
                if !key.mutable {
                    return Err(RuntimeErrorKind::CannotAssignImmutable(name.to_string()));
                } else {
                    key.value = value;
                    return Ok(());
                }
            }
        }
        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }
    pub fn debug_scopes(&self) {
        eprintln!("scopes:");
        for (i, scope) in self.scopes.iter().enumerate() {
            eprintln!("  scope {i}: {:?}", scope.keys().collect::<Vec<_>>());
        }
    }
}
