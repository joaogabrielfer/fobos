use std::fmt::Display;
use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use crate::file_utils::read_line_from;
use crate::lexer::{Token, TokenKind, TokenTag};
use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Bind {
        mutable: bool,
        name: String,
        type_annotation: Option<Type>,
        value: Expr,
    },
    Expr(Expr),
    Return(Expr),
    Assignment {
        target: Expr,
        value: Expr,
    },
    While {
        condition: Expr,
        block: Block,
    },
}

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Unit,

    Block(Block),

    Tuple(Vec<Expr>),

    Unary {
        op: UnaryOp,
        operan: Box<Expr>,
    },

    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "integer '{i}'"),
            Expr::Float(d) => write!(f, "float '{d}'"),
            Expr::String(s) => write!(f, "string '{s}'"),
            Expr::Bool(b) => write!(f, "bool '{b}'"),
            Expr::Ident(i) => write!(f, "identifier '{i}'"),
            Expr::Unit => write!(f, "unit"),
            Expr::Block(_) => write!(f, "block"),
            Expr::Tuple(_) => write!(f, "tuple"),
            Expr::Unary { .. } => write!(f, "unary operation"),
            Expr::Binary { .. } => write!(f, "binary operation"),
            Expr::Call { .. } => write!(f, "function calling"),
            Expr::If { .. } => write!(f, "if condition"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    file_path: &'a PathBuf,
}

#[derive(Error, Debug)]
#[error(
    " --> {file_path}:{}:{}:\n|\n|   {} -> {kind}",
    read_line_from(file_path, *pos).0,
    read_line_from(file_path, *pos).1,
    read_line_from(file_path, *pos).2,
)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("expected token '{expected}', found '{found}'")]
    ExpectedToken { expected: String, found: String },
    #[error("expected tokens '{expected:?}', found '{found}'")]
    ExpectedTokens {
        expected: Vec<String>,
        found: String,
    },
    #[error("expected identifier, found '{found}'")]
    ExpectedIdentifier { found: String },
    #[error("expected expression, found '{found}'")]
    ExpectedExpression { found: String },
    #[error("unexpected token '{found}'")]
    UnexpectedToken { found: String },
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("{expr} is not a valid assignment target")]
    InvalidAssignmentTarget { expr: String },
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, file_path: &'a PathBuf) -> Self {
        Self {
            tokens,
            file_path,
            index: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Box<ParserError>> {
        let mut statements = vec![];
        self.consume_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            // eprintln!("{:#?}", stmt.clone());
            statements.push(stmt);
            self.expect_many(vec![TokenTag::NewLine])?;
            self.consume_newlines();
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, Box<ParserError>> {
        match self.current().kind {
            TokenKind::Let => self.parse_binding(false),
            TokenKind::Var => self.parse_binding(true),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => {
                self.advance();
                Ok(Stmt::Return(self.parse_expr()?))
            }
            _ => {
                let expr = self.parse_expr()?;
                if self.check(TokenTag::Equals) {
                    if !matches!(expr, Expr::Ident(_) | Expr::Tuple(_)) {
                        return Err(self.error(ParserErrorKind::InvalidAssignmentTarget {
                            expr: expr.to_string(),
                        }));
                    }
                    self.advance();
                    let value = self.parse_expr()?;
                    Ok(Stmt::Assignment {
                        target: expr,
                        value,
                    })
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_binding(&mut self, mutable: bool) -> Result<Stmt, Box<ParserError>> {
        if mutable {
            self.expect(TokenTag::Var)?;
        } else {
            self.expect(TokenTag::Let)?;
        }
        let name = self.expect_ident()?;
        let type_annotation = self.parse_type()?;
        self.expect(TokenTag::Equals)?;
        let value = if self.check(TokenTag::NewLine) {
            Expr::Block(self.parse_block()?)
        } else {
            self.parse_expr()?
        };

        Ok(Stmt::Bind {
            mutable,
            name,
            type_annotation,
            value,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, Box<ParserError>> {
        self.expect(TokenTag::While)?;
        let condition = self.parse_expr()?;
        self.expect(TokenTag::Do)?;
        let block = self.parse_block()?;
        Ok(Stmt::While { condition, block })
    }

    fn parse_expr(&mut self) -> Result<Expr, Box<ParserError>> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_level: u8) -> Result<Expr, Box<ParserError>> {
        let mut lhs = self.parse_unary_expr()?;

        while let Some((op, level)) = self.current_tag().precedence_level()
            && level >= min_level
        {
            self.advance();
            let rhs = self.parse_binary_expr(level + 1)?;
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_block(&mut self) -> Result<Block, Box<ParserError>> {
        let mut statements = vec![];

        if !self.check(TokenTag::NewLine) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            return Ok(Block { statements });
        }

        self.consume_newlines();
        while !self.check(TokenTag::End) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.expect_many(vec![TokenTag::NewLine])?;
            self.consume_newlines();
        }

        self.advance();

        Ok(Block { statements })
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, Box<ParserError>> {
        if self.check(TokenTag::Bang) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                operan: Box::new(expr),
            });
        }

        if self.check(TokenTag::Minus) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                operan: Box::new(expr),
            });
        }

        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, Box<ParserError>> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(TokenTag::LParen) {
                let args = self.parse_call_args()?;

                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };

                continue;
            }

            if self.check(TokenTag::Dot) {
                self.advance();

                let method_name = self.expect_ident()?;

                if !self.check(TokenTag::LParen) {
                    return Err(self.error(ParserErrorKind::ExpectedToken {
                        expected: "(".to_string(),
                        found: self.current().kind.tag().to_string(),
                    }));
                }

                let mut args = self.parse_call_args()?;

                if let Expr::Tuple(mut t) = expr {
                    while let Some(e) = t.pop() {
                        args.insert(0, e);
                    }
                } else {
                    args.insert(0, expr);
                }

                expr = Expr::Call {
                    callee: Box::new(Expr::Ident(method_name)),
                    args,
                };

                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, Box<ParserError>> {
        self.expect(TokenTag::LParen)?;
        let mut args = vec![];

        if self.check(TokenTag::RParen) {
            self.advance();
            return Ok(args);
        }

        loop {
            let expr = self.parse_expr()?;
            args.push(expr);

            if self.check(TokenTag::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenTag::RParen)?;
        Ok(args)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, Box<ParserError>> {
        match self.current().kind.clone() {
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Expr::Float(n))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr::Bool(b))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expr::String(s.to_string()))
            }
            TokenKind::Ident(i) => {
                self.advance();
                Ok(Expr::Ident(i.to_string()))
            }
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Do => {
                self.expect(TokenTag::Do)?;
                Ok(Expr::Block(self.parse_block()?))
            }
            TokenKind::LParen => {
                self.advance();
                let mut exprs = vec![];
                if self.check(TokenTag::RParen) {
                    self.advance();
                    return Ok(Expr::Unit);
                }
                let mut tuple = self.check(TokenTag::LParen);
                exprs.push(self.parse_expr()?);
                while self.check(TokenTag::Comma) {
                    tuple = true;
                    self.advance();
                    exprs.push(self.parse_expr()?);
                }
                self.expect(TokenTag::RParen)?;
                if tuple {
                    Ok(Expr::Tuple(exprs))
                } else {
                    Ok(exprs[0].clone())
                }
            }
            other => Err(self.error(ParserErrorKind::ExpectedExpression {
                found: other.tag().to_string(),
            })),
        }
    }

    fn parse_type(&mut self) -> Result<Option<Type>, Box<ParserError>> {
        self.expect(TokenTag::Colon)?;
        match self.current().kind.clone() {
            TokenKind::Equals => Ok(None),
            TokenKind::Ident(t) => {
                self.advance();
                Ok(Some(Type::Named(t.to_string())))
            }
            other => Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: vec!["=".to_string(), "type".to_string()],
                found: format!("{other:?}"),
            })),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, Box<ParserError>> {
        self.expect(TokenTag::If)?;
        let condition = Box::new(self.parse_expr()?);
        let then_branch = Box::new(self.parse_expr()?);
        let else_branch = if self.check(TokenTag::Else) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        Ok(Expr::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn current(&self) -> &Token<'a> {
        &self.tokens[self.index]
    }

    fn current_tag(&self) -> TokenTag {
        self.current().kind.tag()
    }

    fn check(&self, tag: TokenTag) -> bool {
        self.current_tag() == tag
    }

    fn is_at_end(&self) -> bool {
        self.check(TokenTag::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.index += 1
        }
    }

    fn _previous(&self) -> Option<&Token<'a>> {
        if self.index > 0 {
            Some(&self.tokens[self.index - 1])
        } else {
            None
        }
    }

    fn consume_newlines(&mut self) {
        while self.check(TokenTag::NewLine) {
            self.advance();
        }
    }

    fn error(&self, kind: ParserErrorKind) -> Box<ParserError> {
        Box::new(ParserError {
            kind,
            file_path: self.file_path.clone(),
            pos: self.current().span,
        })
    }

    fn expect(&mut self, tag: TokenTag) -> Result<(), Box<ParserError>> {
        if self.current_tag() == tag {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParserErrorKind::ExpectedToken {
                expected: tag.to_string(),
                found: self.current().origin.to_string(),
            }))
        }
    }
    fn expect_many(&mut self, tags: Vec<TokenTag>) -> Result<(), Box<ParserError>> {
        let mut any = false;
        for tag in &tags {
            if *tag == self.current_tag() {
                any = true
            }
        }
        if any {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: tags.iter().map(|t| t.to_string()).collect(),
                found: self.current().origin.to_string(),
            }))
        }
    }
    fn expect_ident(&mut self) -> Result<String, Box<ParserError>> {
        match self.current().kind.clone() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s.to_string())
            }
            other => Err(self.error(ParserErrorKind::ExpectedIdentifier {
                found: format!("{other:?}"),
            })),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::lexer::Lexer;
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string},
    };

    use crate::{file_utils::create_expected_by_ext, parser};

    #[test]
    fn validate_expected_ast() {
        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let entries = read_dir(format!("{cargo_dir}/fixtures")).unwrap();

        for entry in entries {
            let current_file_path = entry.unwrap().path();

            if current_file_path.is_file()
                && current_file_path.extension() == Some(OsStr::new("blorp"))
            {
                let content = read_to_string(&current_file_path).unwrap();
                let tokens = Lexer::new(&current_file_path, &content).tokenize();

                let ast_str = match tokens {
                    Ok(t) => {
                        let ast = parser::Parser::new(t, &current_file_path).parse_program();
                        format!("{ast:#?}")
                    }
                    Err(e) => format!("{e:#?}"),
                };

                let ast_expected_path = create_expected_by_ext(&current_file_path, ".ast").unwrap();
                let expected_ast = match read_to_string(ast_expected_path.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Expected ast file {ast_expected_path:?} not found. Skipping it");
                        continue;
                    }
                };

                for (i, (my_line, expected_line)) in
                    ast_str.lines().zip(expected_ast.lines()).enumerate()
                {
                    assert_eq!(
                        my_line, expected_line,
                        "failed to match at line '{i}' in file {ast_expected_path:?}: "
                    )
                }
            }
        }
    }
}
