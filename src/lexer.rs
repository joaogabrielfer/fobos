use crate::{
    file_utils::read_line_from,
    source::{Span, SrcPos},
};
use std::{iter::Peekable, path::PathBuf, str::Chars};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    file_path: &'a PathBuf,
    raw_src: &'a str,
    source: Peekable<Chars<'a>>,
    pos: SrcPos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub origin: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    LParen, // (
    RParen, // )
    LBrace, // [
    RBrace, // ]
    LAngle, // <
    RAngle, // >
    LCurly, // {
    RCurly, // }

    Colon,    // :
    Comma,    // ,
    Dot,      // .
    RArrow,   // ->
    FatArrow, // =>

    Equals, // =
    Plus,   // +
    Minus,  // -
    Slash,  // /
    Star,   // *
    Bang,   // !

    BangEquals,    // !=
    EqualsEquals,  // ==
    GreaterEquals, // <=
    LessEquals,    // >=

    Ident(&'a str),
    String(&'a str),
    Int(i64),
    Float(f64),
    Bool(bool),

    Fun,
    Return,
    End,
    Var,
    Let,
    For,
    While,
    In,
    Do,
    Match,
    If,
    Else,

    NewLine,
    Eof,
}

impl<'a> TokenKind<'a> {
    pub fn tag(&self) -> TokenTag {
        match self {
            TokenKind::LParen => TokenTag::LParen,
            TokenKind::RParen => TokenTag::RParen,
            TokenKind::LBrace => TokenTag::LBrace,
            TokenKind::RBrace => TokenTag::RBrace,
            TokenKind::LAngle => TokenTag::LAngle,
            TokenKind::RAngle => TokenTag::RAngle,
            TokenKind::LCurly => TokenTag::LCurly,
            TokenKind::RCurly => TokenTag::RCurly,

            TokenKind::Colon => TokenTag::Colon,
            TokenKind::Comma => TokenTag::Comma,
            TokenKind::Dot => TokenTag::Dot,
            TokenKind::RArrow => TokenTag::RArrow,
            TokenKind::FatArrow => TokenTag::FatArrow,

            TokenKind::Equals => TokenTag::Equals,
            TokenKind::Plus => TokenTag::Plus,
            TokenKind::Minus => TokenTag::Minus,
            TokenKind::Slash => TokenTag::Slash,
            TokenKind::Star => TokenTag::Star,
            TokenKind::Bang => TokenTag::Bang,

            TokenKind::BangEquals => TokenTag::BangEquals,
            TokenKind::EqualsEquals => TokenTag::EqualsEquals,
            TokenKind::GreaterEquals => TokenTag::GreaterEquals,
            TokenKind::LessEquals => TokenTag::LessEquals,

            TokenKind::Ident(_) => TokenTag::Ident,
            TokenKind::String(_) => TokenTag::String,
            TokenKind::Int(_) => TokenTag::Int,
            TokenKind::Float(_) => TokenTag::Float,
            TokenKind::Bool(_) => TokenTag::Bool,

            TokenKind::Fun => TokenTag::Fun,
            TokenKind::Return => TokenTag::Return,
            TokenKind::End => TokenTag::End,
            TokenKind::Var => TokenTag::Var,
            TokenKind::Let => TokenTag::Let,
            TokenKind::For => TokenTag::For,
            TokenKind::While => TokenTag::While,
            TokenKind::In => TokenTag::In,
            TokenKind::Do => TokenTag::Do,
            TokenKind::Match => TokenTag::Match,
            TokenKind::If => TokenTag::If,
            TokenKind::Else => TokenTag::Else,

            TokenKind::NewLine => TokenTag::NewLine,
            TokenKind::Eof => TokenTag::Eof,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTag {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    LCurly,
    RCurly,

    Colon,
    Comma,
    Dot,
    RArrow,
    FatArrow,

    Equals,
    Plus,
    Minus,
    Slash,
    Star,
    Bang,

    BangEquals,
    EqualsEquals,
    GreaterEquals,
    LessEquals,

    Ident,
    String,
    Int,
    Float,
    Bool,

    Fun,
    Return,
    End,
    Var,
    Let,
    For,
    While,
    In,
    Do,
    Match,
    If,
    Else,

    NewLine,
    Eof,
}

impl std::fmt::Display for TokenTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TokenTag::LParen => "(",
            TokenTag::RParen => ")",
            TokenTag::LBrace => "[",
            TokenTag::RBrace => "]",
            TokenTag::LAngle => "<",
            TokenTag::RAngle => ">",
            TokenTag::LCurly => "{",
            TokenTag::RCurly => "}",

            TokenTag::Colon => ":",
            TokenTag::Comma => ",",
            TokenTag::Dot => ".",
            TokenTag::RArrow => "->",
            TokenTag::FatArrow => "=>",

            TokenTag::Equals => "=",
            TokenTag::Plus => "+",
            TokenTag::Minus => "-",
            TokenTag::Slash => "/",
            TokenTag::Star => "*",
            TokenTag::Bang => "!",

            TokenTag::BangEquals => "!=",
            TokenTag::EqualsEquals => "==",
            TokenTag::GreaterEquals => ">=",
            TokenTag::LessEquals => "<=",

            TokenTag::Ident => "identifier",
            TokenTag::String => "string",
            TokenTag::Int => "integer",
            TokenTag::Float => "float",
            TokenTag::Bool => "boolean",

            TokenTag::Fun => "fun",
            TokenTag::Return => "return",
            TokenTag::End => "end",
            TokenTag::Var => "var",
            TokenTag::Let => "let",
            TokenTag::For => "for",
            TokenTag::While => "while",
            TokenTag::In => "in",
            TokenTag::Do => "do",
            TokenTag::Match => "match",
            TokenTag::If => "if",
            TokenTag::Else => "else",

            TokenTag::NewLine => "newline",
            TokenTag::Eof => "EOF",
        };
        write!(f, "{str}")
    }
}

#[derive(Error, Debug)]
#[error(
    " --> {file_path}:{}:{}:\n|\n|   {} -> {kind}",
    read_line_from(file_path, *pos).0,
    read_line_from(file_path, *pos).1,
    read_line_from(file_path, *pos).2,
)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub file_path: PathBuf,
    pub pos: Span,
}

#[derive(Error, Debug)]
pub enum LexerErrorKind {
    #[error("Unrecognized char '{0}'")]
    UnknownChar(char),
    #[error("Unrecognized Token '{0}'")]
    UnknownToken(String),
    #[error("Invalid number '{0}'")]
    InvalidNumber(String),
    #[error("Unterminated String")]
    UnterminatedString,
}

impl<'a> Lexer<'a> {
    pub fn new(file_path: &'a PathBuf, source: &'a str) -> Self {
        Self {
            file_path,
            raw_src: source,
            source: source.chars().peekable(),
            pos: SrcPos {
                line: 1,
                col: 1,
                idx: 0,
            },
        }
    }

    fn advance(&mut self) -> Option<char> {
        let result = self.source.next();
        match result {
            Some('\n') => {
                self.pos.line += 1;
                self.pos.col = 0;
                self.pos.idx += '\n'.len_utf8();
                result
            }
            Some(ch) => {
                self.pos.col += 1;
                self.pos.idx += ch.len_utf8();
                result
            }
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.source.peek() {
            if c.is_whitespace() && *c != '\n' {
                self.pos.col += c.len_utf8();
                self.pos.idx += c.len_utf8();
                self.source.next();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>, LexerError> {
        self.skip_whitespace();
        let start_idx = self.pos.idx;
        // println!("current = {:?}", self.source.peek());
        match self.advance() {
            Some('(') => Ok(self.new_token(TokenKind::LParen, "(")),
            Some(')') => Ok(self.new_token(TokenKind::RParen, ")")),
            Some('{') => Ok(self.new_token(TokenKind::LCurly, "{")),
            Some('}') => Ok(self.new_token(TokenKind::RCurly, "}")),
            Some('[') => Ok(self.new_token(TokenKind::LBrace, "[")),
            Some(']') => Ok(self.new_token(TokenKind::RBrace, "]")),
            Some(':') => Ok(self.new_token(TokenKind::Colon, ":")),
            Some(',') => Ok(self.new_token(TokenKind::Comma, ",")),
            Some('.') => Ok(self.new_token(TokenKind::Dot, ".")),
            Some('+') => Ok(self.new_token(TokenKind::Plus, "+")),
            Some('*') => Ok(self.new_token(TokenKind::Star, "*")),
            Some('\n') => Ok(self.new_token(TokenKind::NewLine, "\n")),
            Some('>') => match self.source.peek() {
                Some('=') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::GreaterEquals, ">="))
                }
                _ => Ok(self.new_token(TokenKind::RAngle, ">")),
            },
            Some('<') => match self.source.peek() {
                Some('=') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::LessEquals, "<="))
                }
                _ => Ok(self.new_token(TokenKind::LAngle, "<")),
            },
            Some('-') => match self.source.peek() {
                Some('>') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::RArrow, "->"))
                }
                _ => Ok(self.new_token(TokenKind::Minus, "-")),
            },
            Some('/') => match self.source.peek() {
                Some('/') => {
                    while let Some(c) = self.advance()
                        && c != '\n'
                    {}
                    Ok(self.new_token(TokenKind::NewLine, "\\n"))
                }
                _ => Ok(self.new_token(TokenKind::Slash, "/")),
            },
            Some('=') => match self.source.peek() {
                Some('=') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::EqualsEquals, "=="))
                }
                Some('>') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::FatArrow, "=>"))
                }
                _ => Ok(self.new_token(TokenKind::Equals, "=")),
            },
            Some('!') => match self.source.peek() {
                Some('=') => {
                    self.advance();
                    Ok(self.new_token(TokenKind::BangEquals, "!="))
                }
                _ => Ok(self.new_token(TokenKind::Bang, "!")),
            },
            Some('a'..='z') | Some('A'..='Z') | Some('_') => {
                while let Some('a'..='z') | Some('A'..='Z') | Some('_') | Some('0'..='9') =
                    self.source.peek()
                {
                    self.advance();
                }

                let end_idx = self.pos.idx;

                let s = &self.raw_src[start_idx..end_idx];
                match s {
                    "true" => Ok(self.new_token(TokenKind::Bool(true), "true")),
                    "false" => Ok(self.new_token(TokenKind::Bool(false), "false")),
                    "fun" => Ok(self.new_token(TokenKind::Fun, "fun")),
                    "return" => Ok(self.new_token(TokenKind::Return, "return")),
                    "end" => Ok(self.new_token(TokenKind::End, "end")),
                    "var" => Ok(self.new_token(TokenKind::Var, "var")),
                    "let" => Ok(self.new_token(TokenKind::Let, "let")),
                    "for" => Ok(self.new_token(TokenKind::For, "for")),
                    "while" => Ok(self.new_token(TokenKind::While, "while")),
                    "in" => Ok(self.new_token(TokenKind::In, "in")),
                    "do" => Ok(self.new_token(TokenKind::Do, "do")),
                    "match" => Ok(self.new_token(TokenKind::Match, "match")),
                    "if" => Ok(self.new_token(TokenKind::If, "if")),
                    "else" => Ok(self.new_token(TokenKind::Else, "else")),
                    _ => Ok(self.new_token(TokenKind::Ident(s), s)),
                }
            }
            Some('0'..='9') => {
                while let Some('0'..='9') | Some('.') = self.source.peek() {
                    self.advance();
                }

                if let Some(c) = self.source.peek()
                    && matches!(c, 'a'..='z' |'A'..='Z' | '_' | '.')
                {
                    let end_idx = self.pos.idx + 1;
                    let num_str = &self.raw_src[start_idx..end_idx];
                    return Err(self.error(LexerErrorKind::InvalidNumber(num_str.to_string())));
                }

                let end_idx = self.pos.idx;
                let num_str = &self.raw_src[start_idx..end_idx];
                if num_str.contains(".") {
                    match num_str.parse::<f64>() {
                        Ok(num) => Ok(self.new_token(TokenKind::Float(num), num_str)),
                        Err(_) => {
                            Err(self.error(LexerErrorKind::InvalidNumber(num_str.to_string())))
                        }
                    }
                } else {
                    match num_str.parse::<i64>() {
                        Ok(num) => Ok(self.new_token(TokenKind::Int(num), num_str)),
                        Err(_) => {
                            Err(self.error(LexerErrorKind::InvalidNumber(num_str.to_string())))
                        }
                    }
                }
            }
            Some('"') => {
                while let Some(c) = self.source.peek() {
                    if *c == '"' {
                        break;
                    }

                    self.advance();
                }

                match self.source.peek() {
                    Some('"') => {
                        let end_idx = self.pos.idx;

                        self.advance(); // consume closing quote

                        let s = &self.raw_src[start_idx + 1..end_idx];
                        let origin = &self.raw_src[start_idx..end_idx + 1];
                        Ok(self.new_token(TokenKind::String(s), origin))
                    }
                    _ => Err(self.error(LexerErrorKind::UnterminatedString)),
                }
            }
            Some(other) => Err(self.error(LexerErrorKind::UnknownChar(other))),
            None => Ok(self.new_token(TokenKind::Eof, "EOF")),
        }
    }

    fn error(&self, kind: LexerErrorKind) -> LexerError {
        LexerError {
            kind,
            file_path: self.file_path.clone(),
            pos: Span {
                start: self.pos,
                end: self.pos,
            },
        }
    }

    fn new_token(&self, kind: TokenKind<'a>, origin: &'a str) -> Token<'a> {
        let offset = match origin {
            "EOF" => 0,
            "\\n" => 0,
            _ if self.pos.col < 1 => 0,
            _ => origin.len(),
        };

        let subtract = if self.pos.col > 1 { 1 } else { 0 };
        // eprintln!("offset = {offset}\nkind = {kind:?}");
        Token {
            origin,
            kind,
            span: Span::new(
                SrcPos {
                    line: self.pos.line,
                    col: self.pos.col - offset,
                    idx: self.pos.idx - offset,
                },
                SrcPos {
                    line: self.pos.line,
                    col: self.pos.col - subtract,
                    idx: self.pos.idx - subtract,
                },
            ),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, LexerError> {
        let mut tokens = vec![];
        loop {
            let tk = self.next_token()?;
            if tk.kind == TokenKind::Eof {
                tokens.push(tk);
                break;
            }
            tokens.push(tk);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string},
    };

    use crate::file_utils::create_expected_by_ext;

    use super::*;

    #[test]
    fn validate_expected_tokens() {
        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let entries = read_dir(format!("{cargo_dir}/fixtures")).unwrap();

        for entry in entries {
            let current_file_path = entry.unwrap().path();

            if current_file_path.is_file()
                && current_file_path.extension() == Some(OsStr::new("blorp"))
            {
                let content = read_to_string(&current_file_path).unwrap();
                let tokens = Lexer::new(&current_file_path, &content).tokenize();
                let tokens_str = format!("{tokens:#?}");

                let token_expected_path =
                    create_expected_by_ext(&current_file_path, ".tokens").unwrap();
                let expected_tokens = read_to_string(token_expected_path.clone()).unwrap();

                for (i, (my_line, expected_line)) in
                    tokens_str.lines().zip(expected_tokens.lines()).enumerate()
                {
                    assert_eq!(
                        my_line, expected_line,
                        "failed to match at line '{i}' in file {token_expected_path:?}: "
                    )
                }
            }
        }
    }

    #[test]
    fn test_successful_tokenization() {
        let test_cases = vec![
            (
                "{}[]<>>()",
                vec![
                    TokenKind::LCurly,
                    TokenKind::RCurly,
                    TokenKind::LBrace,
                    TokenKind::RBrace,
                    TokenKind::LAngle,
                    TokenKind::RAngle,
                    TokenKind::RAngle,
                    TokenKind::LParen,
                    TokenKind::RParen,
                    TokenKind::Eof,
                ],
            ),
            (
                "+-*:.,",
                vec![
                    TokenKind::Plus,
                    TokenKind::Minus,
                    TokenKind::Star,
                    TokenKind::Colon,
                    TokenKind::Dot,
                    TokenKind::Comma,
                    TokenKind::Eof,
                ],
            ),
            (
                "!= == <= >= -> > >= = =>",
                vec![
                    TokenKind::BangEquals,
                    TokenKind::EqualsEquals,
                    TokenKind::LessEquals,
                    TokenKind::GreaterEquals,
                    TokenKind::RArrow,
                    TokenKind::RAngle,
                    TokenKind::GreaterEquals,
                    TokenKind::Equals,
                    TokenKind::FatArrow,
                    TokenKind::Eof,
                ],
            ),
            (
                r#"let bar
                let baz
                var bar :=
                while true do
                    self.foo()
                end"#,
                vec![
                    TokenKind::Let,
                    TokenKind::Ident("bar"),
                    TokenKind::NewLine,
                    TokenKind::Let,
                    TokenKind::Ident("baz"),
                    TokenKind::NewLine,
                    TokenKind::Var,
                    TokenKind::Ident("bar"),
                    TokenKind::Colon,
                    TokenKind::Equals,
                    TokenKind::NewLine,
                    TokenKind::While,
                    TokenKind::Bool(true),
                    TokenKind::Do,
                    TokenKind::NewLine,
                    TokenKind::Ident("self"),
                    TokenKind::Dot,
                    TokenKind::Ident("foo"),
                    TokenKind::LParen,
                    TokenKind::RParen,
                    TokenKind::NewLine,
                    TokenKind::End,
                    TokenKind::Eof,
                ],
            ),
            (
                r#"let foo := 10
                var bar: String = "bar""#,
                vec![
                    TokenKind::Let,
                    TokenKind::Ident("foo"),
                    TokenKind::Colon,
                    TokenKind::Equals,
                    TokenKind::Int(10),
                    TokenKind::NewLine,
                    TokenKind::Var,
                    TokenKind::Ident("bar"),
                    TokenKind::Colon,
                    TokenKind::Ident("String"),
                    TokenKind::Equals,
                    TokenKind::String("bar"),
                    TokenKind::Eof,
                ],
            ),
            (
                r#"10 15.4 let var "foo bar" "18""#,
                vec![
                    TokenKind::Int(10),
                    TokenKind::Float(15.4),
                    TokenKind::Let,
                    TokenKind::Var,
                    TokenKind::String("foo bar"),
                    TokenKind::String("18"),
                    TokenKind::Eof,
                ],
            ),
        ];

        for (input, expected_kinds) in test_cases {
            let file_path = PathBuf::new();
            let mut lexer = Lexer::new(&file_path, input);

            // TODO: test error cases too instead of crashing out here
            let tokens = lexer.tokenize().expect("Lexing failed unexpectedly");
            // println!("{tokens:#?}");
            // println!();
            // println!();

            assert_eq!(
                tokens.len(),
                expected_kinds.len(),
                "Token count mismatch for input: '{}'",
                input
            );

            for (token, expected_kind) in tokens.iter().zip(expected_kinds.iter()) {
                assert_eq!(
                    &token.kind, expected_kind,
                    "Expected token kind {:?}, got {:?} in input: '{}'",
                    expected_kind, token.kind, input
                );
            }
        }
    }
}
