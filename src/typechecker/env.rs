use std::collections::HashMap;

use crate::{errors::TypeErrorKind, module::ModuleId, typechecker::ty::Type};

#[derive(Debug, Clone)]
pub enum TypeBinding {
    Local(Type),
    ImportedMember {
        module: ModuleId,
        export_name: String,
        ty: Type,
    },
    Module(ModuleId),
}

impl TypeBinding {
    pub fn value_type(&self) -> Option<Type> {
        match self {
            Self::Local(ty) | Self::ImportedMember { ty, .. } => Some(ty.clone()),
            Self::Module(_) => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, TypeBinding>>,
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
        self.define_binding(name, TypeBinding::Local(ty));
    }

    pub fn define_binding(&mut self, name: String, binding: TypeBinding) {
        self.scopes
            .last_mut()
            .expect("should always have at least one scope")
            .insert(name, binding);
    }

    pub fn get_binding(&self, name: &str) -> Result<TypeBinding, TypeErrorKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(TypeErrorKind::UndefinedVariable(name.to_string()))
    }

    pub fn get(&self, name: &str) -> Result<Type, TypeErrorKind> {
        self.get_binding(name)?
            .value_type()
            .ok_or_else(|| TypeErrorKind::NotAValue(name.to_string()))
    }

    pub fn get_current(&self, name: &str) -> Option<TypeBinding> {
        self.scopes
            .last()
            .and_then(|scope| scope.get(name))
            .cloned()
    }

    pub fn at_module_scope(&self) -> bool {
        self.scopes.len() == 1
    }
}
