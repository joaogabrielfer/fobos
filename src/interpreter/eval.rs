use anyhow::Result;

use crate::{
    ast::{
        BinaryOp, Block, Expr, ExprKind, Program,
        Stmt::{self},
    },
    interpreter::{
        Interpreter,
        errors::{RuntimeError, RuntimeErrorKind},
        values::Value,
    },
    source::Span,
};

pub enum EvalFlow {
    Continue(Value),
    Return(Value),
    Yield(Value),
}

macro_rules! value_or_flow {
    ($flow:expr) => {
        match $flow {
            EvalFlow::Continue(value) => value,
            EvalFlow::Return(value) => return Ok(EvalFlow::Return(value)),
            EvalFlow::Yield(value) => return Ok(EvalFlow::Yield(value)),
        }
    };
}

impl<'a> Interpreter<'a> {
    pub fn eval_program(&mut self, program: &'a Program) -> Result<Value, Box<RuntimeError>> {
        for stmt in &program.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(_value) => {}
                EvalFlow::Yield(_value) => {}
                EvalFlow::Return(value) => {
                    self.env.pop_scope();
                    return Ok(value);
                }
            }
        }
        Ok(Value::Unit)
    }

    fn eval_statement(&mut self, statement: &'a Stmt) -> Result<EvalFlow, Box<RuntimeError>> {
        match statement {
            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(EvalFlow::Continue(value_or_flow!(value)))
            }
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(EvalFlow::Return(value_or_flow!(value)))
            }
            Stmt::Yield(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(EvalFlow::Yield(value_or_flow!(value)))
            }
            Stmt::Bind {
                mutable,
                name,
                value,
                ..
            } => {
                let value = self.eval_expr(value)?;
                self.env.define(name, *mutable, value_or_flow!(value));
                Ok(EvalFlow::Continue(Value::Unit))
            }
            Stmt::Assignment { target, value } => {
                let value_span = value.span;
                let value = self.eval_expr(value)?;
                if let ExprKind::Ident(name) = &target.kind {
                    self.env
                        .assign(name, value_or_flow!(value))
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
            Stmt::While { condition, block } => {
                loop {
                    let condition_bool = match value_or_flow!(self.eval_expr(condition)?) {
                        Value::Bool(b) => b,
                        other => {
                            return Err(self.error_at(
                                condition.span,
                                RuntimeErrorKind::ExpectedBool {
                                    found: other.to_string(),
                                },
                            ));
                        }
                    };

                    if !condition_bool {
                        break;
                    }

                    match self.eval_block(block)? {
                        EvalFlow::Continue(_) => {}
                        EvalFlow::Return(value) => {
                            return Ok(EvalFlow::Return(value));
                        }
                        EvalFlow::Yield(value) => {
                            return Ok(EvalFlow::Yield(value));
                        }
                    }
                }

                Ok(EvalFlow::Continue(Value::Unit))
            }
            Stmt::FunDecl {
                name,
                generics,
                parameters,
                return_type,
                body,
            } => todo!(),
        }
    }

    fn eval_block(&mut self, block: &'a Block) -> Result<EvalFlow, Box<RuntimeError>> {
        self.env.push_scope();

        let mut last_value = Value::Unit;

        for stmt in &block.statements {
            match self.eval_statement(stmt)? {
                EvalFlow::Continue(value) => {
                    last_value = value;
                }
                EvalFlow::Return(value) => {
                    self.env.pop_scope();
                    return Ok(EvalFlow::Return(value));
                }
                EvalFlow::Yield(value) => {
                    self.env.pop_scope();
                    return Ok(EvalFlow::Yield(value));
                }
            }
        }
        self.env.pop_scope();
        Ok(EvalFlow::Continue(last_value))
    }

    fn eval_block_expr(&mut self, block: &'a Block) -> Result<EvalFlow, Box<RuntimeError>> {
        match self.eval_block(block)? {
            EvalFlow::Continue(value) => Ok(EvalFlow::Continue(value)),
            EvalFlow::Return(value) => Ok(EvalFlow::Return(value)),
            EvalFlow::Yield(value) => Ok(EvalFlow::Continue(value)),
        }
    }

    fn eval_expr(&mut self, expr: &'a Expr) -> Result<EvalFlow, Box<RuntimeError>> {
        match &expr.kind {
            ExprKind::Int(val) => Ok(EvalFlow::Continue(Value::Int(*val))),
            ExprKind::Float(val) => Ok(EvalFlow::Continue(Value::Float(*val))),
            ExprKind::String(val) => Ok(EvalFlow::Continue(Value::String(val.clone()))),
            ExprKind::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(*val))),
            ExprKind::Ident(i) => {
                let value = self.env.get(i).map_err(|kind| {
                    Box::new(RuntimeError {
                        kind,
                        span: expr.span,
                        file_path: self.file_path.clone(),
                    })
                })?;
                Ok(EvalFlow::Continue(value))
            }
            ExprKind::Unit => Ok(EvalFlow::Continue(Value::Unit)),
            ExprKind::Block(block) => self.eval_block_expr(block),
            ExprKind::Tuple(exprs) => {
                let mut values = vec![];

                for expr in exprs {
                    let value = value_or_flow!(self.eval_expr(expr)?);
                    values.push(value);
                }

                Ok(EvalFlow::Continue(Value::Tuple(values)))
            }
            ExprKind::Unary { op, operand } => {
                let value = value_or_flow!(self.eval_expr(operand)?);
                match op {
                    crate::ast::UnaryOp::Negate => match value {
                        Value::Int(val) => Ok(EvalFlow::Continue(Value::Int(-val))),
                        Value::Float(val) => Ok(EvalFlow::Continue(Value::Float(-val))),
                        other => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidUnaryOp {
                                op: *op,
                                operand: other.to_string(),
                            },
                        )),
                    },
                    crate::ast::UnaryOp::Not => match value {
                        Value::Bool(val) => Ok(EvalFlow::Continue(Value::Bool(!val))),
                        other => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidUnaryOp {
                                op: *op,
                                operand: other.to_string(),
                            },
                        )),
                    },
                }
            }
            ExprKind::Binary { lhs, op, rhs } => {
                let lhs = value_or_flow!(self.eval_expr(lhs)?);
                let rhs = value_or_flow!(self.eval_expr(rhs)?);
                match op {
                    BinaryOp::Add => match (lhs, rhs) {
                        (Value::String(s1), Value::String(s2)) => Ok(EvalFlow::Continue(
                            Value::String(format!("{s1}{s2}").to_string()),
                        )),
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 + i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 + f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Sub => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 - i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 - f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Mul => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 * i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 * f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Div => match (lhs, rhs) {
                        (Value::Int(i1), Value::Int(i2)) => {
                            Ok(EvalFlow::Continue(Value::Int(i1 / i2)))
                        }
                        (Value::Float(f1), Value::Float(f2)) => {
                            Ok(EvalFlow::Continue(Value::Float(f1 / f2)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Eq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Bool(a), Value::Bool(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Tuple(a), Value::Tuple(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a == b)))
                        }
                        (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(true))),
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::NotEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Bool(a), Value::Bool(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Tuple(a), Value::Tuple(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a != b)))
                        }
                        (Value::Unit, Value::Unit) => Ok(EvalFlow::Continue(Value::Bool(false))),
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Greater => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a > b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a > b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::GreaterEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a >= b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a >= b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::Less => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a < b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a < b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                    BinaryOp::LessEq => match (lhs, rhs) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a <= b)))
                        }
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(EvalFlow::Continue(Value::Bool(a <= b)))
                        }
                        (other_lhs, other_rhs) => Err(self.error_at(
                            expr.span,
                            RuntimeErrorKind::InvalidBinaryOp {
                                op: *op,
                                lhs: other_lhs.to_string(),
                                rhs: other_rhs.to_string(),
                            },
                        )),
                    },
                }
            }
            ExprKind::Call { callee, args } => todo!(),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = match self.eval_expr(condition)? {
                    EvalFlow::Continue(value) => value,
                    EvalFlow::Return(value) => return Ok(EvalFlow::Return(value)),
                    EvalFlow::Yield(value) => return Ok(EvalFlow::Yield(value)),
                };

                let condition_bool = match condition_value {
                    Value::Bool(b) => b,
                    other => {
                        return Err(self.error_at(
                            condition.span,
                            RuntimeErrorKind::ExpectedBool {
                                found: other.to_string(),
                            },
                        ));
                    }
                };

                if condition_bool {
                    self.eval_expr(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.eval_expr(else_branch)
                } else {
                    Ok(EvalFlow::Continue(Value::Unit))
                }
            }
            ExprKind::Lambda { params, body } => todo!(),
        }
    }

    fn error_at(&self, span: Span, kind: RuntimeErrorKind) -> Box<RuntimeError> {
        Box::new(RuntimeError {
            kind,
            span,
            file_path: self.file_path.clone(),
        })
    }
}
