use crate::{
    errors::{
        RuntimeError, RuntimeErrorKind, render_parameter_types, render_types_from_value_vector,
    },
    interpreter::{
        Interpreter,
        env::Env,
        eval::EvalFlow,
        values::{self, RangeValue, Value},
    },
    typechecker::{TypeChecker, env::TypeEnv, ty::Type},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Echo,
    Range,
    Push,
}

impl BuiltinFunction {
    pub fn get_type(&self) -> Type {
        match self {
            BuiltinFunction::Echo => Type::Function {
                parameter_overloads: vec![vec![Type::String]],
                return_type: Box::new(Type::Range),
            },
            BuiltinFunction::Range => Type::Function {
                parameter_overloads: vec![
                    vec![Type::Int],
                    vec![Type::Int, Type::Int],
                    vec![Type::Int, Type::Int, Type::Int],
                ],
                return_type: Box::new(Type::Range),
            },
            BuiltinFunction::Push => Type::Function {
                parameter_overloads: vec![vec![Type::Array(Box::new(Type::Any)), Type::Any]],
                return_type: Box::new(Type::Array(Box::new(Type::Any))),
            },
        }
    }
}

impl Env {
    pub fn load_builtins(&mut self) {
        self.define(
            "echo".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Echo),
        );
        self.define(
            "range".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Range),
        );
        self.define(
            "push".to_string(),
            false,
            values::Value::BuiltinFunction(BuiltinFunction::Push),
        );
    }
}

impl TypeEnv {
    pub fn load_builtins(&mut self) {
        self.define("echo".to_string(), BuiltinFunction::Echo.get_type());
        self.define("range".to_string(), BuiltinFunction::Range.get_type());
        self.define("push".to_string(), BuiltinFunction::Push.get_type());
    }
}

impl<'a, W: std::io::Write> Interpreter<'a, W> {
    pub fn call_builtin(
        &mut self,
        builtin: BuiltinFunction,
        args_values: &mut Vec<Value>,
        span: crate::source::Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        match builtin {
            BuiltinFunction::Echo => {
                let Type::Function {
                    parameter_overloads,
                    ..
                } = builtin.get_type()
                else {
                    panic!("builtin types should always be functions")
                };

                if let Some(0) = TypeChecker::check_function_call(&parameter_overloads, args_values)
                {
                    self.writeln_output(format!("{}", args_values[0]))?;
                    Ok(EvalFlow::Continue(Value::Unit))
                } else {
                    Err(self.error_at(
                        span,
                        RuntimeErrorKind::SignatureMismatch {
                            expected: render_parameter_types(parameter_overloads),
                            found: render_types_from_value_vector(args_values.clone()),
                        },
                    ))
                }
            }
            BuiltinFunction::Range => {
                let Type::Function {
                    parameter_overloads,
                    ..
                } = builtin.get_type()
                else {
                    panic!("builtin types should always be functions")
                };

                // check_function_call here returns the index of the parameter list that matched
                match TypeChecker::check_function_call(&parameter_overloads, args_values) {
                    Some(0) => {
                        let Value::Int(end) = args_values[0] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        };

                        Ok(EvalFlow::Continue(Value::Range(RangeValue {
                            start: 0,
                            end,
                            inclusive: false,
                            step: 1,
                        })))
                    }
                    Some(1) => {
                        let Value::Int(start) = args_values[0] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        };

                        let Value::Int(end) = args_values[1] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::InvalidBuiltinParameter(
                                    args_values[0].to_string(),
                                ),
                            ));
                        };

                        Ok(EvalFlow::Continue(Value::Range(RangeValue {
                            start,
                            end,
                            inclusive: false,
                            step: 1,
                        })))
                    }
                    Some(2) => {
                        let Value::Int(start) = args_values[0] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        };

                        let Value::Int(end) = args_values[1] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        };

                        let Value::Int(step) = args_values[2] else {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        };

                        if step == 0 {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::BadRangeStep {
                                    found: step.to_string(),
                                },
                            ));
                        }

                        Ok(EvalFlow::Continue(Value::Range(RangeValue {
                            start,
                            end,
                            inclusive: false,
                            step,
                        })))
                    }
                    _ => Err(self.error_at(
                        span,
                        RuntimeErrorKind::SignatureMismatch {
                            expected: render_parameter_types(parameter_overloads),
                            found: render_types_from_value_vector(args_values.clone()),
                        },
                    )),
                }
            }
            BuiltinFunction::Push => {
                let Type::Function {
                    parameter_overloads,
                    ..
                } = builtin.get_type()
                else {
                    panic!("builtin types should always be functions")
                };

                if let Some(0) = TypeChecker::check_function_call(&parameter_overloads, args_values)
                {
                    let item = args_values
                        .pop()
                        .expect("array should never have less than 2 elements");
                    let array = args_values
                        .pop()
                        .expect("array should never have less than 2 elements");
                    let Value::Array(mut array) = array else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::SignatureMismatch {
                                expected: render_parameter_types(parameter_overloads),
                                found: render_types_from_value_vector(args_values.clone()),
                            },
                        ));
                    };
                    eprintln!("item = {}", item.type_name());
                    eprintln!("item = {}", item.get_type());
                    match array.first() {
                        None => array.push(item),
                        Some(t) if t.get_type() == item.get_type() => {
                            array.push(item);
                            eprintln!("array = {:?}", array)
                        }
                        Some(_) => {
                            return Err(self.error_at(
                                span,
                                RuntimeErrorKind::SignatureMismatch {
                                    expected: render_parameter_types(parameter_overloads),
                                    found: render_types_from_value_vector(args_values.clone()),
                                },
                            ));
                        }
                    }
                    Ok(EvalFlow::Continue(Value::Unit))
                } else {
                    Err(self.error_at(
                        span,
                        RuntimeErrorKind::SignatureMismatch {
                            expected: render_parameter_types(parameter_overloads),
                            found: render_types_from_value_vector(args_values.clone()),
                        },
                    ))
                }
            }
        }
    }
}
