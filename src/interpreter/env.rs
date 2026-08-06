use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{
    errors::RuntimeErrorKind,
    interpreter::values::{ModuleValue, Value},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvFrame {
    pub frame: HashMap<String, Binding>,
    pub parent: Option<EnvRef>,
}

pub type EnvRef = Rc<RefCell<EnvFrame>>;

#[derive(Debug, Clone)]
pub struct Env {
    current: EnvRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub mutable: bool,
    pub kind: BindingKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindingKind {
    Value(Value),
    ImportedMember {
        module: Rc<ModuleValue>,
        export_name: String,
    },
    Module(Rc<ModuleValue>),
}

impl Default for Env {
    fn default() -> Self {
        let frame = EnvFrame::default();
        Self {
            current: Rc::new(RefCell::new(frame)),
        }
    }
}

impl Env {
    pub fn current_ref(&self) -> EnvRef {
        Rc::clone(&self.current)
    }

    pub fn from_ref(current: EnvRef) -> Self {
        Self { current }
    }

    pub fn push_scope(&mut self) {
        let new_frame = Rc::new(RefCell::new(EnvFrame {
            frame: HashMap::new(),
            parent: Some(Rc::clone(&self.current)),
        }));

        self.current = new_frame;
    }

    pub fn pop_scope(&mut self) {
        let parent = self
            .current
            .borrow()
            .parent
            .clone()
            .expect("cannot pop global scope");

        self.current = parent;
    }
    pub fn define(&mut self, name: String, mutable: bool, value: Value) {
        self.current.borrow_mut().frame.insert(
            name,
            Binding {
                mutable,
                kind: BindingKind::Value(value),
            },
        );
    }

    pub fn define_module(&mut self, name: String, module: Rc<ModuleValue>) {
        self.current.borrow_mut().frame.insert(
            name,
            Binding {
                mutable: false,
                kind: BindingKind::Module(module),
            },
        );
    }

    pub fn define_imported_member(
        &mut self,
        name: String,
        module: Rc<ModuleValue>,
        export_name: String,
    ) {
        self.current.borrow_mut().frame.insert(
            name,
            Binding {
                mutable: false,
                kind: BindingKind::ImportedMember {
                    module,
                    export_name,
                },
            },
        );
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeErrorKind> {
        let mut current = Some(Rc::clone(&self.current));

        while let Some(env) = current {
            let borrowed = env.borrow();

            if let Some(binding) = borrowed.frame.get(name) {
                return match &binding.kind {
                    BindingKind::Value(value) => Ok(value.clone()),
                    BindingKind::ImportedMember {
                        module,
                        export_name,
                    } => {
                        if !module.exports.contains(export_name) {
                            return Err(RuntimeErrorKind::UnknownModuleExport {
                                module: module.id.to_string(),
                                member: export_name.clone(),
                            });
                        }
                        Env::from_ref(module.env.clone()).get(export_name)
                    }
                    BindingKind::Module(module) => Ok(Value::Module(module.clone())),
                };
            }

            current = borrowed.parent.clone();
        }

        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeErrorKind> {
        let mut current = Some(Rc::clone(&self.current));

        while let Some(env) = current {
            let mut borrowed = env.borrow_mut();

            if let Some(binding) = borrowed.frame.get_mut(name) {
                if !binding.mutable {
                    return Err(RuntimeErrorKind::CannotAssignImmutable(name.to_string()));
                }

                let BindingKind::Value(existing) = &mut binding.kind else {
                    return Err(RuntimeErrorKind::CannotAssignImportedMember(
                        name.to_string(),
                    ));
                };
                *existing = value;
                return Ok(());
            }

            current = borrowed.parent.clone();
        }

        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }

    pub fn mutate_binding<R>(
        &mut self,
        name: &str,
        f: impl FnOnce(&mut Binding) -> Result<R, RuntimeErrorKind>,
    ) -> Result<R, RuntimeErrorKind> {
        let mut current = Some(self.current_ref());

        while let Some(env) = current {
            let mut borrowed = env.borrow_mut();

            if let Some(binding) = borrowed.frame.get_mut(name) {
                if !matches!(binding.kind, BindingKind::Value(_)) {
                    return Err(RuntimeErrorKind::CannotAssignImportedMember(
                        name.to_string(),
                    ));
                }
                return f(binding);
            }

            current = borrowed.parent.clone();
        }

        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }
}
