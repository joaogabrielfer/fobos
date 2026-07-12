use crate::{
    ast::{CallParameter, ExprArgument, ExprKind, ValueArgument},
    errors::{RuntimeError, RuntimeErrorKind},
    interpreter::{
        Interpreter,
        env::Env,
        eval::{EvalFlow, YieldMode},
        values::{self, MatchedCall, RangeValue, Value, normalize_arguments},
    },
    source::Span,
    typechecker::{
        TypeChecker,
        env::TypeEnv,
        ty::{ParameterType, Type},
    },
    value_or_flow,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinParameterMode {
    Value,
    Place,
}

#[derive(Debug, Clone)]
pub struct BuiltinParameter {
    pub name: &'static str,
    pub ty: Type,
    pub mode: BuiltinParameterMode,
}

impl CallParameter for BuiltinParameter {
    fn name(&self) -> &str {
        self.name
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinVariant {
    pub parameters: Vec<BuiltinParameter>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Echo,
    Range,
    Push,
}

impl BuiltinFunction {
    pub fn variants(&self) -> Vec<BuiltinVariant> {
        match self {
            BuiltinFunction::Echo => vec![BuiltinVariant {
                parameters: vec![BuiltinParameter {
                    name: "value",
                    ty: Type::Any,
                    mode: BuiltinParameterMode::Value,
                }],
                return_type: Type::Unit,
            }],
            BuiltinFunction::Range => vec![
                BuiltinVariant {
                    parameters: vec![BuiltinParameter {
                        name: "end",
                        ty: Type::Int,
                        mode: BuiltinParameterMode::Value,
                    }],
                    return_type: Type::Range,
                },
                BuiltinVariant {
                    parameters: vec![
                        BuiltinParameter {
                            name: "start",
                            ty: Type::Int,
                            mode: BuiltinParameterMode::Value,
                        },
                        BuiltinParameter {
                            name: "end",
                            ty: Type::Int,
                            mode: BuiltinParameterMode::Value,
                        },
                    ],
                    return_type: Type::Range,
                },
                BuiltinVariant {
                    parameters: vec![
                        BuiltinParameter {
                            name: "start",
                            ty: Type::Int,
                            mode: BuiltinParameterMode::Value,
                        },
                        BuiltinParameter {
                            name: "end",
                            ty: Type::Int,
                            mode: BuiltinParameterMode::Value,
                        },
                        BuiltinParameter {
                            name: "step",
                            ty: Type::Int,
                            mode: BuiltinParameterMode::Value,
                        },
                    ],
                    return_type: Type::Range,
                },
            ],
            BuiltinFunction::Push => vec![BuiltinVariant {
                parameters: vec![
                    BuiltinParameter {
                        name: "target",
                        ty: Type::Array(Box::new(Type::Any)),
                        mode: BuiltinParameterMode::Place,
                    },
                    BuiltinParameter {
                        name: "value",
                        ty: Type::Any,
                        mode: BuiltinParameterMode::Value,
                    },
                ],
                return_type: Type::Unit,
            }],
        }
    }

    pub fn get_type(&self) -> Type {
        let variants = self.variants();
        Type::Function {
            parameter_overloads: variants
                .iter()
                .map(|variant| {
                    variant
                        .parameters
                        .iter()
                        .map(|parameter| ParameterType {
                            name: parameter.name.to_string(),
                            ty: parameter.ty.clone(),
                        })
                        .collect()
                })
                .collect(),
            return_type: Box::new(variants[0].return_type.clone()),
        }
    }

    pub fn needs_raw_args(&self) -> bool {
        matches!(self, BuiltinFunction::Push)
    }
}

impl Env {
    pub fn load_builtins(&mut self) {
        for (name, builtin) in [
            ("echo", BuiltinFunction::Echo),
            ("range", BuiltinFunction::Range),
            ("push", BuiltinFunction::Push),
        ] {
            self.define(
                name.to_string(),
                false,
                values::Value::BuiltinFunction(builtin),
            );
        }
    }
}

impl TypeEnv {
    pub fn load_builtins(&mut self) {
        for (name, builtin) in [
            ("echo", BuiltinFunction::Echo),
            ("range", BuiltinFunction::Range),
            ("push", BuiltinFunction::Push),
        ] {
            self.define(name.to_string(), builtin.get_type());
        }
    }
}

impl<'a, W: std::io::Write> Interpreter<'a, W> {
    fn match_builtin_values(
        &self,
        builtin: &BuiltinFunction,
        arguments: &[ValueArgument],
    ) -> Result<MatchedCall<Value>, Box<RuntimeErrorKind>> {
        let variants = builtin.variants();
        let mut matches = Vec::new();
        let mut last_error = None;

        for (variant_index, variant) in variants.iter().enumerate() {
            let normalized = match normalize_arguments(&variant.parameters, arguments) {
                Ok(arguments) => arguments,
                Err(error) => {
                    last_error = Some(error);
                    continue;
                }
            };

            if variant
                .parameters
                .iter()
                .zip(&normalized)
                .all(|(parameter, value)| {
                    TypeChecker::types_compatible(&parameter.ty, &value.get_type())
                })
            {
                matches.push(MatchedCall {
                    variant_index,
                    arguments: normalized,
                });
            }
        }

        match matches.len() {
            1 => Ok(matches.remove(0)),
            0 => match last_error {
                Some(error) => Err(Box::new(RuntimeErrorKind::ArgumentError { e: error })),
                None => Err(Box::new(RuntimeErrorKind::SignatureMismatch {
                    expected: builtin.get_type().to_string(),
                    found: arguments
                        .iter()
                        .map(|argument| argument.value.get_type().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                })),
            },
            _ => Err(Box::new(RuntimeErrorKind::SignatureMismatch {
                expected: builtin.get_type().to_string(),
                found: "ambiguous arguments".to_string(),
            })),
        }
    }

    pub fn call_builtin(
        &mut self,
        builtin: BuiltinFunction,
        arguments: Vec<ValueArgument>,
        span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let matched = self
            .match_builtin_values(&builtin, &arguments)
            .map_err(|kind| self.error_at(span, *kind))?;

        match (builtin, matched.variant_index, matched.arguments.as_slice()) {
            (BuiltinFunction::Echo, 0, [value]) => {
                self.writeln_output(value.to_string())?;
                Ok(EvalFlow::Continue(Value::Unit))
            }
            (BuiltinFunction::Range, 0, [Value::Int(end)]) => {
                Ok(EvalFlow::Continue(Value::Range(RangeValue {
                    start: 0,
                    end: *end,
                    inclusive: false,
                    step: 1,
                })))
            }
            (BuiltinFunction::Range, 1, [Value::Int(start), Value::Int(end)]) => {
                Ok(EvalFlow::Continue(Value::Range(RangeValue {
                    start: *start,
                    end: *end,
                    inclusive: false,
                    step: 1,
                })))
            }
            (BuiltinFunction::Range, 2, [Value::Int(start), Value::Int(end), Value::Int(step)]) => {
                if *step == 0 {
                    return Err(self.error_at(
                        span,
                        RuntimeErrorKind::BadRangeStep {
                            found: step.to_string(),
                        },
                    ));
                }
                Ok(EvalFlow::Continue(Value::Range(RangeValue {
                    start: *start,
                    end: *end,
                    inclusive: false,
                    step: *step,
                })))
            }
            _ => unreachable!("builtin matching returned an invalid signature"),
        }
    }

    pub fn call_builtin_raw(
        &mut self,
        builtin: BuiltinFunction,
        arguments: Vec<ExprArgument>,
        call_span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let variants = builtin.variants();
        let variant = &variants[0];
        let arguments = normalize_arguments(&variant.parameters, &arguments)
            .map_err(|error| self.argument_error(call_span, *error))?;

        match (builtin, arguments.as_slice()) {
            (BuiltinFunction::Push, [target, value_expression]) => {
                let value =
                    value_or_flow!(self.eval_expr(value_expression.clone(), YieldMode::Capture)?);
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
                    ExprKind::Index {
                        target: indexed_target,
                        index,
                    } => {
                        let index_span = index.span;
                        let (name, mut indices) = self.resolve_nested_index(
                            *indexed_target.clone(),
                            *index.clone(),
                            vec![],
                        )?;
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
                                            indexed_target.kind.to_string(),
                                        ));
                                    }
                                };
                                indices.reverse();
                                while let Some(index) = indices.pop() {
                                    if index < 0 || index as usize >= array.len() {
                                        return Err(RuntimeErrorKind::OutOfBounds(index));
                                    }
                                    if indices.is_empty() {
                                        let Value::Array(new_array) = array
                                            .get_mut(index as usize)
                                            .ok_or(RuntimeErrorKind::OutOfBounds(index))?
                                        else {
                                            return Err(RuntimeErrorKind::InvalidIndexingTarget(
                                                indexed_target.kind.to_string(),
                                            ));
                                        };
                                        array = new_array;
                                    }
                                }
                                array.push(value);
                                Ok(())
                            })
                            .map_err(|kind| self.error_at(index_span, kind))?;
                        Ok(EvalFlow::Continue(Value::Unit))
                    }
                    other => Err(self.error_at(
                        target.span,
                        RuntimeErrorKind::InvalidAssignmentTarget(other.to_string()),
                    )),
                }
            }
            _ => unreachable!("raw builtin matching returned an invalid signature"),
        }
    }
}
