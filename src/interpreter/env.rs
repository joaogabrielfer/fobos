use std::collections::HashMap;

use anyhow::Result;

use crate::interpreter::{errors::RuntimeErrorKind, values::Value};

pub struct Env<'a> {
    scopes: Vec<HashMap<&'a str, Binding>>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub mutable: bool,
    pub value: Value,
}

impl<'a> Default for Env<'a> {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
}

impl<'a> Env<'a> {
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    pub fn get(&self, name: &str) -> Result<Value, RuntimeErrorKind> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name) {
                return Ok(scope.get(&name).unwrap().clone().value);
            }
        }
        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }
    pub fn define(&mut self, name: &'a str, mutable: bool, value: Value) {
        self.scopes
            .last_mut()
            .expect("scopes should always have at least one entry")
            .insert(name, Binding { mutable, value });
    }

    pub fn assign(&mut self, name: &'a str, value: Value) -> Result<(), RuntimeErrorKind> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(key) = scope.get_mut(name) {
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
}
