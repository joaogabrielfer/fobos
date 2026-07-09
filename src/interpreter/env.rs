use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{errors::RuntimeErrorKind, interpreter::values::Value};

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
    pub value: Value,
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
        self.current
            .borrow_mut()
            .frame
            .insert(name, Binding { mutable, value });
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeErrorKind> {
        let mut current = Some(Rc::clone(&self.current));

        while let Some(env) = current {
            let borrowed = env.borrow();

            if let Some(binding) = borrowed.frame.get(name) {
                return Ok(binding.value.clone());
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

                binding.value = value;
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
                return f(binding);
            }

            current = borrowed.parent.clone();
        }

        Err(RuntimeErrorKind::UndefinedVariable(name.to_string()))
    }
}
