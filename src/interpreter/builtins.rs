use crate::{
    ast::{Expr, ExprKind},
    errors::{
        RuntimeError, RuntimeErrorKind, render_parameter_types, render_types_from_value_vector,
    },
    interpreter::{
        Interpreter,
        env::Env,
        eval::{EvalFlow, YieldMode},
        values::{self, RangeValue, Value},
    },
    source::Span,
    typechecker::{TypeChecker, env::TypeEnv, ty::Type},
    value_or_flow,
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
                parameter_overloads: vec![vec![Type::Any]],
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

    pub fn needs_raw_args(&self) -> bool {
        matches!(self, BuiltinFunction::Push)
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
                                RuntimeErrorKind::InvalidFunctionParameter(
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
            _ => unreachable!("raw builtin passed to regular call_builtin"),
        }
    }

    pub fn call_builtin_raw(
        &mut self,
        builtin: BuiltinFunction,
        args: Vec<Expr>,
        call_span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        match builtin {
            BuiltinFunction::Push => {
                let Type::Function {
                    parameter_overloads,
                    ..
                } = builtin.get_type()
                else {
                    panic!("builtin types should always be functions")
                };

                if args.len() != 2 {
                    let eval_flows_for_printing = args
                        .into_iter()
                        .map(|e| self.eval_expr(e, YieldMode::Capture))
                        .collect::<Result<Vec<_>, _>>()?;

                    let mut values_for_printing = vec![];
                    for e in eval_flows_for_printing {
                        values_for_printing.push(value_or_flow!(e));
                    }

                    return Err(self.error_at(
                        call_span,
                        RuntimeErrorKind::SignatureMismatch {
                            expected: render_parameter_types(parameter_overloads),
                            found: render_types_from_value_vector(values_for_printing.clone()),
                        },
                    ));
                }

                let target = &args[0];
                let value_expr = args[1].clone();

                let value = value_or_flow!(self.eval_expr(value_expr, YieldMode::Capture)?);

                match &target.kind {
                    ExprKind::Ident(name) => {
                        self.env
                            .mutate_binding(name, |binding| {
                                if !binding.mutable {
                                    return Err(RuntimeErrorKind::CannotAssignImmutable(
                                        name.clone(),
                                    ));
                                }

                                match &mut binding.value {
                                    Value::Array(items) => {
                                        items.push(value);
                                        Ok(())
                                    }

                                    other => Err(RuntimeErrorKind::ExpectedArray {
                                        found: other.type_name().to_string(),
                                    }),
                                }
                            })
                            .map_err(|kind| self.error_at(target.span, kind))?;

                        Ok(EvalFlow::Continue(Value::Unit))
                    }

                    ExprKind::Index { target, index } => {
                        let index_span = index.span;
                        let (name, mut indices) =
                            self.resolve_nested_index(*target.clone(), *index.clone(), vec![])?;

                        self.env
                            .mutate_binding(&name, |binding| {
                                if !binding.mutable {
                                    return Err(RuntimeErrorKind::CannotAssignImmutable(
                                        name.clone(),
                                    ));
                                }
                                let mut array = match binding.value {
                                    Value::Array(ref mut array) => array,
                                    _ => {
                                        return Err(RuntimeErrorKind::InvalidIndexingTarget(
                                            target.kind.to_string(),
                                        ));
                                    }
                                };
                                indices.reverse();
                                while let Some(idx) = indices.pop() {
                                    if idx as usize >= array.len() || idx < 0 {
                                        return Err(RuntimeErrorKind::OutOfBounds(idx));
                                    }

                                    if indices.is_empty() {
                                        let Value::Array(new_array) =
                                            array
                                                .get_mut(idx as usize)
                                                .ok_or(RuntimeErrorKind::OutOfBounds(idx))?
                                        else {
                                            return Err(RuntimeErrorKind::InvalidIndexingTarget(
                                                target.kind.to_string(),
                                            ));
                                        };

                                        array = new_array;
                                    }
                                }

                                array.push(value);

                                Ok(EvalFlow::Continue(Value::Unit))
                            })
                            .map_err(|kind| self.error_at(index_span, kind))
                    }

                    other => Err(self.error_at(
                        target.span,
                        RuntimeErrorKind::InvalidAssignmentTarget(other.to_string()),
                    )),
                }
            }

            _ => unreachable!("non-raw builtin passed to call_builtin_raw"),
        }
    }
}
