use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{
    ast::{
        BinaryOp, Block, Expr, ExprKind, Program,
        Stmt::{self},
        TypeAnnotation, UnaryOp,
    },
    errors::{RuntimeError, RuntimeErrorKind},
    interpreter::{
        Interpreter,
        env::{Env, EnvFrame},
        values::{FunctionBody, FunctionValue, RangeValue, RuntimeIterator, Value},
    },
    source::{Span, SrcPos},
};

pub enum EvalFlow {
    Continue(Value),
    Return { value: Value, span: Span },
    Yield { value: Value, span: Span },
}

#[derive(Debug, Clone, Copy)]
pub enum YieldMode {
    Bubble,
    Capture,
}

#[macro_export]
macro_rules! value_or_flow {
    ($flow:expr) => {
        match $flow {
            EvalFlow::Continue(value) => value,
            EvalFlow::Return { value, span } => return Ok(EvalFlow::Return { value, span }),
            EvalFlow::Yield { value, span } => return Ok(EvalFlow::Yield { value, span }),
        }
    };
}

impl<'a, W: std::io::Write> Interpreter<'a, W> {
    pub fn eval_program(&mut self, program: Program) -> Result<Value, Box<RuntimeError>> {
        for stmt in program.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(_value) => {}

                EvalFlow::Yield { span, .. } => {
                    return Err(self.error_at(span, RuntimeErrorKind::YieldOutsideHandler));
                }

                EvalFlow::Return { value, span: _ } => {
                    return Ok(value);
                }
            }
        }

        Ok(Value::Unit)
    }

    fn eval_statement(&mut self, statement: Stmt) -> Result<EvalFlow, Box<RuntimeError>> {
        match statement {
            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr, YieldMode::Bubble)?;
                Ok(EvalFlow::Continue(value_or_flow!(value)))
            }
            Stmt::Return(expr) => {
                let span = expr.span;
                let value = self.eval_expr(expr, YieldMode::Capture)?;
                Ok(EvalFlow::Return {
                    value: value_or_flow!(value),
                    span,
                })
            }
            Stmt::Yield(expr) => {
                let span = expr.span;
                let value = self.eval_expr(expr, YieldMode::Capture)?;
                Ok(EvalFlow::Yield {
                    value: value_or_flow!(value),
                    span,
                })
            }
            Stmt::Bind {
                mutable,
                name,
                value,
                ..
            } => {
                let value = value_or_flow!(self.eval_expr(value, YieldMode::Capture)?);
                self.env.define(name, mutable, value);
                Ok(EvalFlow::Continue(Value::Unit))
            }
            Stmt::Assignment { target, value } => {
                let value_span = value.span;
                let value = self.eval_expr(value, YieldMode::Capture)?;
                if let ExprKind::Ident(name) = target.kind {
                    self.env
                        .assign(&name, value_or_flow!(value))
                        .map_err(|kind| {
                            self.error_at(
                                Span {
                                    start: target.span.start,
                                    end: value_span.end,
                                },
                                kind,
                            )
                        })?;
                    Ok(EvalFlow::Continue(Value::Unit))
                } else {
                    Err(self.error_at(
                        target.span,
                        RuntimeErrorKind::InvalidAssignmentTarget(target.kind.to_string()),
                    ))
                }
            }
            Stmt::FunDecl {
                name,
                generics: _,
                parameters,
                return_type,
                body,
                ..
            } => {
                // let return_type = match return_type {
                //     TypeAnnotation::Explicit(t) => t,
                //     TypeAnnotation::Inferred => todo!("type inference is not implemented yet"),
                // };
                let fun = Value::Function(FunctionValue {
                    name: Some(name.clone()),
                    parameters,
                    body: FunctionBody::Block(body),
                    return_type,
                    captured_env: self.env.current_ref(),
                });

                self.env.define(name, false, fun);
                Ok(EvalFlow::Continue(Value::Unit))
            }
        }
    }

    pub fn eval_expr(
        &mut self,
        expr: Expr,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        match expr.kind {
            ExprKind::Int(val) => Ok(EvalFlow::Continue(Value::Int(val))),
            ExprKind::Float(val) => Ok(EvalFlow::Continue(Value::Float(val))),
            ExprKind::String(val) => Ok(EvalFlow::Continue(Value::String(val))),
            ExprKind::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(val))),
            ExprKind::Ident(i) => self.eval_ident(i, expr.span),
            ExprKind::Unit => Ok(EvalFlow::Continue(Value::Unit)),
            ExprKind::Block(block) => self.eval_block(block, yield_mode),
            ExprKind::Tuple(exprs) => self.eval_tuple(exprs),
            ExprKind::Array(exprs) => self.eval_array(exprs),
            ExprKind::Unary { op, operand } => self.eval_unary(op, *operand, expr.span),
            ExprKind::Binary { lhs, op, rhs } => self.eval_binary(op, *lhs, *rhs, expr.span),
            ExprKind::Call { callee, args } => self.eval_call(*callee, args, expr.span),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if(*condition, *then_branch, else_branch, yield_mode),
            ExprKind::While { condition, block } => self.eval_while(*condition, block, yield_mode),
            ExprKind::For {
                binding,
                iterable,
                block,
            } => self.eval_for(*binding, *iterable, block, yield_mode),
            ExprKind::Lambda { parameters, body } => {
                Ok(EvalFlow::Continue(Value::Function(FunctionValue {
                    name: None,
                    parameters,
                    body: FunctionBody::Expr(*body),
                    captured_env: self.env.current_ref(),
                    return_type: TypeAnnotation::Inferred,
                })))
            }
            ExprKind::Index { target, index } => {
                let target_span = target.span;
                let index_span = Span {
                    start: index.span.start,
                    end: SrcPos {
                        line: index.span.end.line,
                        col: index.span.end.col - 1,
                        idx: index.span.end.idx - 1,
                    },
                };
                let array = match value_or_flow!(self.eval_expr(*target, YieldMode::Capture)?) {
                    Value::Array(values) => values,
                    other => {
                        return Err(self.error_at(
                            target_span,
                            RuntimeErrorKind::InvalidIndexingTarget(other.to_string()),
                        ));
                    }
                };
                let index = match value_or_flow!(self.eval_expr(*index, YieldMode::Capture)?) {
                    Value::Int(i) => i,
                    other => {
                        return Err(self.error_at(
                            target_span,
                            RuntimeErrorKind::InvalidIndex(other.to_string()),
                        ));
                    }
                };
                if index < 0 || index as usize >= array.len() {
                    return Err(self.error_at(index_span, RuntimeErrorKind::OutOfBounds(index)));
                }
                Ok(EvalFlow::Continue(array[index as usize].clone()))
            }
        }
    }

    fn eval_call(
        &mut self,
        callee: Expr,
        args: Vec<Expr>,
        span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let callee_span = callee.span;
        let callee = value_or_flow!(self.eval_expr(callee, YieldMode::Capture)?);

        match callee {
            Value::BuiltinFunction(builtin) => {
                if builtin.needs_raw_args() {
                    self.call_builtin_raw(builtin, args, span)
                } else {
                    let mut args_values = vec![];
                    for arg in args {
                        let value = value_or_flow!(self.eval_expr(arg, YieldMode::Capture)?);
                        args_values.push(value);
                    }
                    self.call_builtin(builtin, &mut args_values, span)
                }
            }

            Value::Function(fun) => {
                let mut args_values = vec![];
                for arg in args {
                    let value = value_or_flow!(self.eval_expr(arg, YieldMode::Capture)?);
                    args_values.push(value);
                }
                self.call_function(fun, args_values, span)
            }

            other => Err(self.error_at(
                callee_span,
                RuntimeErrorKind::NotCallable(other.type_name().to_string()),
            )),
        }
    }

    fn eval_block(
        &mut self,
        block: Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let previous_env = self.env.clone();

        self.env.push_scope();

        let result = self.eval_block_inner(block, yield_mode);

        self.env = previous_env;

        result
    }

    fn eval_block_inner(
        &mut self,
        block: Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let mut last_value = Value::Unit;

        for stmt in block.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(value) => {
                    last_value = value;
                }

                EvalFlow::Return { value, span } => {
                    return Ok(EvalFlow::Return { value, span });
                }

                EvalFlow::Yield { value, span } => {
                    return match yield_mode {
                        YieldMode::Capture => Ok(EvalFlow::Continue(value)),
                        YieldMode::Bubble => Ok(EvalFlow::Yield { value, span }),
                    };
                }
            }
        }

        Ok(EvalFlow::Continue(last_value))
    }

    fn eval_if(
        &mut self,
        condition: Expr,
        then_branch: Expr,
        else_branch: Option<Box<Expr>>,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let condition_span = condition.span;
        let condition = value_or_flow!(self.eval_expr(condition, YieldMode::Capture)?);

        let condition_bool = match condition {
            Value::Bool(b) => b,
            other => {
                return Err(self.error_at(
                    condition_span,
                    RuntimeErrorKind::ExpectedBool {
                        found: other.type_name(),
                    },
                ));
            }
        };

        let flow = if condition_bool {
            self.eval_expr(then_branch, yield_mode)?
        } else if let Some(else_branch) = else_branch {
            self.eval_expr(*else_branch, yield_mode)?
        } else if let YieldMode::Capture = yield_mode {
            return Err(self.error_at(
                Span {
                    start: condition_span.start,
                    end: then_branch.span.end,
                },
                RuntimeErrorKind::ElseBranchMissing,
            ));
        } else {
            EvalFlow::Continue(Value::Unit)
        };

        match (yield_mode, flow) {
            (YieldMode::Capture, EvalFlow::Yield { value, .. }) => Ok(EvalFlow::Continue(value)),
            (_, other) => Ok(other),
        }
    }

    fn eval_while(
        &mut self,
        condition: Expr,
        block: Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        loop {
            let condition_span = condition.span;
            let condition_value =
                value_or_flow!(self.eval_expr(condition.clone(), YieldMode::Capture)?);

            let condition_bool = match condition_value {
                Value::Bool(b) => b,
                other => {
                    return Err(self.error_at(
                        condition_span,
                        RuntimeErrorKind::ExpectedBool {
                            found: other.type_name(),
                        },
                    ));
                }
            };

            if !condition_bool {
                break;
            }

            let flow = self.eval_block(block.clone(), YieldMode::Bubble)?;

            match flow {
                EvalFlow::Continue(_) => {}

                EvalFlow::Return { value, span } => {
                    return Ok(EvalFlow::Return { value, span });
                }

                EvalFlow::Yield { value, span } => {
                    return match yield_mode {
                        YieldMode::Capture => Ok(EvalFlow::Continue(value)),
                        YieldMode::Bubble => Ok(EvalFlow::Yield { value, span }),
                    };
                }
            }
        }

        Ok(EvalFlow::Continue(Value::Unit))
    }

    fn eval_for(
        &mut self,
        binding: Expr,
        iterable: Expr,
        block: Block,
        yield_mode: YieldMode,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let iterable_span = iterable.span;
        let binding_span = binding.span;
        let mut iterable = RuntimeIterator::from_value(value_or_flow!(
            self.eval_expr(iterable, YieldMode::Capture)?
        ))
        .map_err(|e| self.error_at(iterable_span, e))?;
        let ExprKind::Ident(binding) = binding.kind else {
            return Err(self.error_at(
                binding_span,
                RuntimeErrorKind::InvalidAssignmentTarget(binding.kind.to_string()),
            ));
        };
        while let Some(value) = iterable.next_value() {
            self.env.define(binding.clone(), false, value);
            let flow = self.eval_block(block.clone(), YieldMode::Bubble)?;

            match flow {
                EvalFlow::Continue(_) => {}

                EvalFlow::Return { value, span } => {
                    return Ok(EvalFlow::Return { value, span });
                }

                EvalFlow::Yield { value, span } => {
                    return match yield_mode {
                        YieldMode::Capture => Ok(EvalFlow::Continue(value)),
                        YieldMode::Bubble => Ok(EvalFlow::Yield { value, span }),
                    };
                }
            }
        }

        Ok(EvalFlow::Continue(Value::Unit))
    }
    fn eval_ident(&mut self, i: String, span: Span) -> Result<EvalFlow, Box<RuntimeError>> {
        // eprintln!("evaluating ident {i}");
        let value = self.env.get(&i).map_err(|kind| {
            Box::new(RuntimeError {
                kind,
                span,
                file_path: self.file_path.clone(),
            })
        })?;
        // eprintln!("has value of {value}");
        Ok(EvalFlow::Continue(value))
    }

    fn eval_tuple(&mut self, exprs: Vec<Expr>) -> Result<EvalFlow, Box<RuntimeError>> {
        {
            let mut values = vec![];

            for expr in exprs {
                let value = value_or_flow!(self.eval_expr(expr, YieldMode::Capture)?);
                values.push(value);
            }

            Ok(EvalFlow::Continue(Value::Tuple(values)))
        }
    }

    fn eval_array(&mut self, exprs: Vec<Expr>) -> Result<EvalFlow, Box<RuntimeError>> {
        {
            let mut values = vec![];

            for expr in exprs {
                let value = value_or_flow!(self.eval_expr(expr, YieldMode::Capture)?);
                values.push(value);
            }

            Ok(EvalFlow::Continue(Value::Array(values)))
        }
    }

    fn eval_unary(
        &mut self,
        op: UnaryOp,
        operand: Expr,
        span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let value = value_or_flow!(self.eval_expr(operand, YieldMode::Capture)?);
        match op {
            crate::ast::UnaryOp::Negate => match value {
                Value::Int(val) => Ok(EvalFlow::Continue(Value::Int(-val))),
                Value::Float(val) => Ok(EvalFlow::Continue(Value::Float(-val))),
                other => Err(self.error_at(
                    span,
                    RuntimeErrorKind::InvalidUnaryOp {
                        op,
                        operand: other.type_name(),
                    },
                )),
            },
            crate::ast::UnaryOp::Not => match value {
                Value::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(!val))),
                other => Err(self.error_at(
                    span,
                    RuntimeErrorKind::InvalidUnaryOp {
                        op,
                        operand: other.type_name(),
                    },
                )),
            },
        }
    }

    fn eval_binary(
        &mut self,
        op: BinaryOp,
        lhs: Expr,
        rhs: Expr,
        span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        {
            let lhs = value_or_flow!(self.eval_expr(lhs, YieldMode::Capture)?);
            let rhs = value_or_flow!(self.eval_expr(rhs, YieldMode::Capture)?);
            match op {
                BinaryOp::Add => match (lhs, rhs) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(EvalFlow::Continue(Value::Int(i1 + i2))),
                    (Value::Float(f1), Value::Float(f2)) => {
                        Ok(EvalFlow::Continue(Value::Float(f1 + f2)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Sub => match (lhs, rhs) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(EvalFlow::Continue(Value::Int(i1 - i2))),
                    (Value::Float(f1), Value::Float(f2)) => {
                        Ok(EvalFlow::Continue(Value::Float(f1 - f2)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Mul => match (lhs, rhs) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(EvalFlow::Continue(Value::Int(i1 * i2))),
                    (Value::Float(f1), Value::Float(f2)) => {
                        Ok(EvalFlow::Continue(Value::Float(f1 * f2)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Div => match (lhs, rhs) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(EvalFlow::Continue(Value::Int(i1 / i2))),
                    (Value::Float(f1), Value::Float(f2)) => {
                        Ok(EvalFlow::Continue(Value::Float(f1 / f2)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Eq => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a == b))),
                    (Value::Bool(a), Value::Bool(b)) => Ok(EvalFlow::Continue(Value::Bool(a == b))),
                    (Value::Tuple(a), Value::Tuple(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::String(a), Value::String(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(true))),
                    (Value::Array(a), Value::Array(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::BuiltinFunction(a), Value::BuiltinFunction(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::Range(a), Value::Range(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (Value::Function(a), Value::Function(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a == b)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::NotEq => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a != b))),
                    (Value::Bool(a), Value::Bool(b)) => Ok(EvalFlow::Continue(Value::Bool(a != b))),
                    (Value::Tuple(a), Value::Tuple(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::String(a), Value::String(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(false))),
                    (Value::Array(a), Value::Array(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::BuiltinFunction(a), Value::BuiltinFunction(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::Range(a), Value::Range(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (Value::Function(a), Value::Function(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a != b)))
                    }
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Greater => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a > b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a > b))),
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::GreaterEq => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a >= b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a >= b))),
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Less => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a < b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a < b))),
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::LessEq => match (lhs, rhs) {
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(EvalFlow::Continue(Value::Bool(a <= b)))
                    }
                    (Value::Int(a), Value::Int(b)) => Ok(EvalFlow::Continue(Value::Bool(a <= b))),
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::Combine => match (lhs, rhs) {
                    (Value::String(s1), s2) => Ok(EvalFlow::Continue(Value::String(
                        format!("{s1}{s2}").to_string(),
                    ))),
                    (s1, Value::String(s2)) => Ok(EvalFlow::Continue(Value::String(
                        format!("{s1}{s2}").to_string(),
                    ))),
                    (other_lhs, other_rhs) => Err(self.error_at(
                        span,
                        RuntimeErrorKind::InvalidBinaryOp {
                            op,
                            lhs: other_lhs.type_name(),
                            rhs: other_rhs.type_name(),
                        },
                    )),
                },
                BinaryOp::InclusiveRange => {
                    let Value::Int(start) = lhs else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidBuiltinParameter(lhs.to_string()),
                        ));
                    };
                    let Value::Int(end) = rhs else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidBuiltinParameter(rhs.to_string()),
                        ));
                    };
                    Ok(EvalFlow::Continue(Value::Range(RangeValue {
                        start,
                        end,
                        inclusive: true,
                        step: 1,
                    })))
                }
                BinaryOp::ExclusiveRange => {
                    let Value::Int(start) = lhs else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidBuiltinParameter(lhs.to_string()),
                        ));
                    };
                    let Value::Int(end) = rhs else {
                        return Err(self.error_at(
                            span,
                            RuntimeErrorKind::InvalidBuiltinParameter(rhs.to_string()),
                        ));
                    };
                    Ok(EvalFlow::Continue(Value::Range(RangeValue {
                        start,
                        end,
                        inclusive: false,
                        step: 1,
                    })))
                }
            }
        }
    }

    fn call_function(
        &mut self,
        fun: FunctionValue,
        args: Vec<Value>,
        span: Span,
    ) -> Result<EvalFlow, Box<RuntimeError>> {
        let previous_env = self.env.clone();
        self.env = Env::from_ref(Rc::new(RefCell::new(EnvFrame {
            frame: HashMap::new(),
            parent: Some(fun.captured_env),
        })));

        for (param, arg) in fun.parameters.iter().zip(args) {
            self.env.define(param.name.clone(), false, arg);
        }

        let flow = match fun.body {
            FunctionBody::Block(block) => self.eval_block(block, YieldMode::Bubble)?,
            FunctionBody::Expr(expr) => self.eval_expr(expr, YieldMode::Capture)?,
        };

        self.env = previous_env;
        match flow {
            EvalFlow::Continue(value) => Ok(EvalFlow::Continue(value)),
            EvalFlow::Return { value, .. } => Ok(EvalFlow::Continue(value)),
            EvalFlow::Yield { .. } => {
                Err(self.error_at(span, RuntimeErrorKind::YieldOutsideHandler))
            }
        }
    }

    pub fn error_at(&self, span: Span, kind: RuntimeErrorKind) -> Box<RuntimeError> {
        Box::new(RuntimeError {
            kind,
            span,
            file_path: self.file_path.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string},
    };

    use crate::{dump::create_expected_by_ext, interpreter::Interpreter, parser};

    #[test]
    fn validate_expected_eval() {
        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let entries = read_dir(format!("{cargo_dir}/fixtures")).unwrap();

        for entry in entries {
            let current_file_path = entry.unwrap().path();

            if current_file_path.is_file()
                && current_file_path.extension() == Some(OsStr::new("blorp"))
            {
                let eval_expected_path =
                    create_expected_by_ext(&current_file_path, ".eval").unwrap();

                let expected_eval = match read_to_string(eval_expected_path.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!(
                            "Expected eval file {eval_expected_path:?} not found. Skipping it"
                        );
                        continue;
                    }
                };

                let content = read_to_string(&current_file_path).unwrap();
                let tokens = Lexer::new(&current_file_path, &content).tokenize();

                let eval_str = match tokens {
                    Ok(tokens) => {
                        let ast = parser::Parser::new(tokens, &current_file_path).parse_program();

                        match ast {
                            Ok(program) => {
                                let mut interpreter = Interpreter::new_buffered(&current_file_path);

                                let eval_result = interpreter.eval_program(program);
                                let output = interpreter.into_output_string();

                                match eval_result {
                                    Ok(value) => {
                                        if output.is_empty() {
                                            format!("result:\n{value:#?}\n")
                                        } else {
                                            format!("output:\n{output}\nresult:\n{value:#?}\n")
                                        }
                                    }
                                    Err(e) => format!("{e:#?}"),
                                }
                            }
                            Err(e) => format!("{e:#?}"),
                        }
                    }
                    Err(e) => format!("{e:#?}"),
                };

                assert_eq!(
                    eval_str, expected_eval,
                    "failed to match eval output in file {eval_expected_path:?}"
                );
            }
        }
    }
}
