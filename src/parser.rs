use std::fmt::Display;
use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use crate::ast::{self, Expr, ExprKind};
use crate::diagnostic::render_source_span;
use crate::lexer::TokenTag::RParen;
use crate::lexer::{Token, TokenKind, TokenTag};
use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    file_path: &'a PathBuf,
}

#[derive(Error, Debug)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match render_source_span(&self.file_path, self.pos) {
            Ok((line, col, snippet)) => {
                write!(
                    f,
                    "error: {}\n --> {}:{}:{}:\n  |\n{} |   {}",
                    self.kind,
                    self.file_path.display(),
                    line,
                    col,
                    line,
                    snippet,
                )
            }
            Err(_) => {
                write!(
                    f,
                    "error: {}\n --> {}:{}:{}",
                    self.kind,
                    self.file_path.display(),
                    self.pos.start.line,
                    self.pos.start.col,
                )
            }
        }
    }
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
    #[error("{expr} is not a valid lamda parameter")]
    InvalidParameter { expr: String },
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, file_path: &'a PathBuf) -> Self {
        Self {
            tokens,
            file_path,
            index: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<ast::Program, Box<ParserError>> {
        let mut statements = vec![];
        self.consume_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            // eprintln!("{:#?}", stmt.clone());
            statements.push(stmt);
            self.expect_many(vec![TokenTag::NewLine])?;
            self.consume_newlines();
        }
        Ok(ast::Program { statements })
    }

    fn parse_statement(&mut self) -> Result<ast::Stmt, Box<ParserError>> {
        match self.current().kind {
            TokenKind::Let => self.parse_binding(false),
            TokenKind::Var => self.parse_binding(true),
            TokenKind::While => self.parse_while(),
            TokenKind::Fun => self.parse_fun_decl(),
            TokenKind::Return => {
                self.advance();
                Ok(ast::Stmt::Return(self.parse_expr()?))
            }
            _ => {
                let starting_span = self.current().span;
                let expr = self.parse_expr()?;
                if self.check(TokenTag::Equals) {
                    if !matches!(expr.kind, ast::ExprKind::Ident(_) | ast::ExprKind::Tuple(_)) {
                        return Err(Box::new(ParserError {
                            kind: ParserErrorKind::InvalidAssignmentTarget {
                                expr: expr.kind.to_string(),
                            },
                            file_path: self.file_path.clone(),
                            pos: starting_span,
                        }));
                    }
                    self.advance();
                    let value = self.parse_expr()?;
                    Ok(ast::Stmt::Assignment {
                        target: expr,
                        value,
                    })
                } else {
                    Ok(ast::Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_binding(&mut self, mutable: bool) -> Result<ast::Stmt, Box<ParserError>> {
        let start_span = self.current().span;
        if mutable {
            self.expect(TokenTag::Var)?;
        } else {
            self.expect(TokenTag::Let)?;
        }
        let name = self.expect_ident()?;
        let type_annotation = self.parse_option_type()?;
        self.expect(TokenTag::Equals)?;
        let value = if self.check(TokenTag::NewLine) {
            let kind = ExprKind::Block(self.parse_block()?);
            self.new_expr(start_span, kind)
        } else {
            self.parse_expr()?
        };

        Ok(ast::Stmt::Bind {
            mutable,
            name,
            type_annotation,
            value,
        })
    }

    fn parse_while(&mut self) -> Result<ast::Stmt, Box<ParserError>> {
        self.expect(TokenTag::While)?;
        let condition = self.parse_expr()?;
        self.expect(TokenTag::Do)?;
        let block = self.parse_block()?;
        Ok(ast::Stmt::While { condition, block })
    }

    fn parse_fun_decl(&mut self) -> Result<ast::Stmt, Box<ParserError>> {
        self.expect(TokenTag::Fun)?;
        let mut generics = vec![];
        if let TokenTag::LBrace = self.current_tag() {
            self.advance();
            while !self.check(TokenTag::RBrace) {
                let name = self.expect_ident()?;
                generics.push(name);
                if let TokenTag::Comma = self.current_tag() {
                    self.expect(TokenTag::Comma)?;
                }
            }
            self.expect(TokenTag::RBrace)?;
        }
        let name = self.expect_ident()?;
        self.expect(TokenTag::LParen)?;
        let mut parameters = vec![];
        while !self.check(TokenTag::NewLine) {
            if let TokenTag::RParen = self.current_tag() {
                self.advance();
                break;
            }
            let name = self.expect_ident()?;
            self.expect(TokenTag::Colon)?;
            let t = ast::Type::Named(self.expect_ident()?);
            parameters.push(ast::Parameter { name, t });
            match self.current_tag() {
                TokenTag::Comma => {
                    self.expect(TokenTag::Comma)?;
                }
                TokenTag::RParen => {
                    self.expect(RParen)?;
                    break;
                }
                other => {
                    return Err(self.error(ParserErrorKind::ExpectedTokens {
                        expected: vec![")".to_string(), ",".to_string()],
                        found: other.to_string(),
                    }));
                }
            }
        }
        let return_type = self.parse_option_type()?;
        self.expect(TokenTag::Equals)?;
        let body = self.parse_block()?;
        Ok(ast::Stmt::FunDecl {
            name,
            generics,
            parameters,
            body,
            return_type,
        })
    }

    fn parse_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        self.parse_lambda()
    }

    fn parse_lambda(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let expr = self.parse_binary_expr(0)?;
        if let TokenTag::RArrow = self.current_tag() {
            let mut params = vec![];
            match expr.kind {
                ExprKind::Ident(i) => params.push(i),
                ExprKind::Tuple(t) => {
                    for e in t {
                        match e.kind {
                            ExprKind::Ident(i) => params.push(i),
                            other => {
                                return Err(self.error(ParserErrorKind::InvalidParameter {
                                    expr: other.to_string(),
                                }));
                            }
                        }
                    }
                }
                other => {
                    return Err(self.error(ParserErrorKind::InvalidParameter {
                        expr: other.to_string(),
                    }));
                }
            }
            self.expect(TokenTag::RArrow)?;
            let body = Box::new(self.parse_expr()?);
            Ok(self.new_expr(start_span, ExprKind::Lambda { params, body }))
        } else {
            Ok(expr)
        }
    }

    fn parse_binary_expr(&mut self, min_level: u8) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let mut lhs = self.parse_unary_expr()?;

        while let Some((op, level)) = self.current_tag().precedence_level()
            && level >= min_level
        {
            self.advance();
            let rhs = self.parse_binary_expr(level + 1)?;
            lhs = self.new_expr(
                start_span,
                ExprKind::Binary {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            );
        }

        Ok(lhs)
    }

    fn parse_block(&mut self) -> Result<ast::Block, Box<ParserError>> {
        let mut statements = vec![];

        if !self.check(TokenTag::NewLine) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            return Ok(ast::Block { statements });
        }

        self.consume_newlines();
        while !self.check(TokenTag::End) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.expect_many(vec![TokenTag::NewLine])?;
            self.consume_newlines();
        }

        self.advance();

        Ok(ast::Block { statements })
    }

    fn parse_unary_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        if self.check(TokenTag::Bang) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(self.new_expr(
                start_span,
                ExprKind::Unary {
                    op: ast::UnaryOp::Not,
                    operand: Box::new(expr),
                },
            ));
        }

        if self.check(TokenTag::Minus) {
            self.advance();

            let expr = self.parse_unary_expr()?;
            return Ok(self.new_expr(
                start_span,
                ExprKind::Unary {
                    op: ast::UnaryOp::Negate,
                    operand: Box::new(expr),
                },
            ));
        }

        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(TokenTag::LParen) {
                let args = self.parse_call_args()?;

                expr = self.new_expr(
                    start_span,
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                );

                continue;
            }

            if self.check(TokenTag::Dot) {
                self.advance();

                let method_name = self.expect_ident()?;
                let end_span = self.current().span;

                if !self.check(TokenTag::LParen) {
                    return Err(self.error(ParserErrorKind::ExpectedToken {
                        expected: "(".to_string(),
                        found: self.current().kind.tag().to_string(),
                    }));
                }

                let mut args = self.parse_call_args()?;

                if let ExprKind::Tuple(mut t) = expr.kind {
                    while let Some(e) = t.pop() {
                        args.insert(0, e);
                    }
                } else {
                    args.insert(0, expr);
                }

                expr = self.new_expr(
                    start_span,
                    ExprKind::Call {
                        callee: Box::new(Expr {
                            kind: ExprKind::Ident(method_name),
                            span: Span {
                                start: start_span.start,
                                end: end_span.end,
                            },
                        }),
                        args,
                    },
                );

                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<ast::Expr>, Box<ParserError>> {
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

    fn parse_primary_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        match self.current().kind.clone() {
            TokenKind::Int(n) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Int(n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Float(n)))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Bool(b)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::String(s.to_string())))
            }
            TokenKind::Ident(i) => {
                self.advance();
                Ok(self.new_expr(start_span, ExprKind::Ident(i.to_string())))
            }
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Do => {
                self.expect(TokenTag::Do)?;
                let block = self.parse_block()?;
                Ok(self.new_expr(start_span, ExprKind::Block(block)))
            }
            TokenKind::LParen => {
                self.advance();
                let mut exprs = vec![];
                if self.check(TokenTag::RParen) {
                    self.advance();
                    return Ok(self.new_expr(start_span, ExprKind::Unit));
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
                    Ok(self.new_expr(start_span, ExprKind::Tuple(exprs)))
                } else {
                    Ok(exprs[0].clone())
                }
            }
            other => Err(self.error(ParserErrorKind::ExpectedExpression {
                found: other.tag().to_string(),
            })),
        }
    }

    fn parse_option_type(&mut self) -> Result<ast::TypeAnnotation, Box<ParserError>> {
        self.expect(TokenTag::Colon)?;
        match self.current_tag() {
            TokenTag::Equals => Ok(ast::TypeAnnotation::Inferred),
            TokenTag::Ident => {
                let t = self.expect_ident()?;
                Ok(ast::TypeAnnotation::Explicit(ast::Type::Named(
                    t.to_string(),
                )))
            }
            TokenTag::LParen => {
                self.expect(TokenTag::LParen)?;
                self.expect(TokenTag::RParen)?;
                Ok(ast::TypeAnnotation::Explicit(ast::Type::Unit))
            }
            other => Err(self.error(ParserErrorKind::ExpectedTokens {
                expected: vec!["=".to_string(), "type".to_string()],
                found: format!("{other:?}"),
            })),
        }
    }

    fn parse_if_expr(&mut self) -> Result<ast::Expr, Box<ParserError>> {
        let start_span = self.current().span;
        self.expect(TokenTag::If)?;
        let condition = Box::new(self.parse_expr()?);
        let then_branch = Box::new(self.parse_expr()?);
        let else_branch = if self.check(TokenTag::Else) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        Ok(self.new_expr(
            start_span,
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
        ))
    }

    fn new_expr(&self, start_span: Span, kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span {
                start: start_span.start,
                end: self.current().span.end,
            },
        }
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

    use crate::{dump::create_expected_by_ext, parser};

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
