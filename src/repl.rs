//! The interactive Fobos session.

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Result;
use colored::Colorize;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{
    ast::Stmt,
    diagnostic::render_source_span_from_source,
    errors::{LexerError, ParserError, ParserErrorKind, RuntimeError, TypeError},
    interpreter::{Interpreter, values::Value},
    lexer::Lexer,
    parser::Parser,
    source::Span,
    typechecker::TypeChecker,
};

#[derive(Debug, PartialEq)]
pub struct Submission {
    pub output: String,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub enum SubmitError {
    Incomplete,
    Diagnostic(String),
}

/// State that persists for the lifetime of one interactive session.
///
/// Runtime bindings deliberately keep their original shared environment
/// references, so closures retain the same capture semantics as file modules.
pub struct Session {
    path: PathBuf,
    interpreter: Interpreter<Vec<u8>>,
    type_checker: TypeChecker,
    check_types: bool,
}

impl Session {
    pub fn new(check_types: bool) -> Self {
        let path = PathBuf::from("<repl>");
        Self {
            interpreter: Interpreter::new_buffered(&path),
            type_checker: TypeChecker::new(path.clone()),
            path,
            check_types,
        }
    }

    pub fn submit(&mut self, source: &str) -> std::result::Result<Submission, SubmitError> {
        let source = ensure_trailing_newline(source);
        let tokens = Lexer::new(&self.path, &source)
            .tokenize()
            .map_err(|error| SubmitError::Diagnostic(lexer_diagnostic(&error, &source)))?;
        let program = match Parser::new(tokens, &self.path).parse_program() {
            Ok(program) => program,
            Err(error) if needs_more_input(&error) => return Err(SubmitError::Incomplete),
            Err(error) => return Err(SubmitError::Diagnostic(parser_diagnostic(&error, &source))),
        };

        if let Some(statement) = program
            .statements
            .iter()
            .find(|statement| matches!(statement, Stmt::ImportDecl { .. }))
        {
            return Err(SubmitError::Diagnostic(diagnostic(
                &"imports are not supported in the REPL; run a .fob file to use modules",
                statement.span(),
                &source,
            )));
        }

        // Type checking is staged so a rejected entry does not introduce type
        // bindings that the interpreter never received.
        let mut next_type_checker = self.type_checker.clone();
        if self.check_types {
            next_type_checker
                .check_program(program.clone())
                .map_err(|error| SubmitError::Diagnostic(type_diagnostic(&error, &source)))?;
        }

        let value = self
            .interpreter
            .eval_program(program)
            .map_err(|error| SubmitError::Diagnostic(runtime_diagnostic(&error, &source)))?;
        let output = self.interpreter.take_output_string();
        self.type_checker = next_type_checker;

        Ok(Submission { output, value })
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.check_types);
    }
}

pub fn run(check_types: bool) -> Result<()> {
    let mut editor = DefaultEditor::new()?;
    let history = history_path();
    if let Some(path) = &history {
        let _ = editor.load_history(path);
    }

    println!("Fobos REPL — type :help for commands.");
    let mut session = Session::new(check_types);
    let mut buffer = String::new();
    let mut interrupt_pending = false;

    loop {
        let prompt = if buffer.is_empty() {
            "fobos> ".green().to_string()
        } else {
            "...    ".green().to_string()
        };

        match editor.readline(&prompt) {
            Ok(line) => {
                interrupt_pending = false;

                if buffer.is_empty() {
                    match handle_command(&line, &mut session)? {
                        Command::Handled => continue,
                        Command::Exit => break,
                        Command::Evaluate => {}
                    }
                }

                buffer.push_str(&line);
                buffer.push('\n');

                match session.submit(&buffer) {
                    Ok(result) => {
                        if !result.output.is_empty() {
                            print!("{}", result.output);
                        }
                        if result.value != Value::Unit {
                            println!("{}", result.value);
                        }
                        editor.add_history_entry(&buffer)?;
                        buffer.clear();
                    }
                    Err(SubmitError::Incomplete) => {}
                    Err(SubmitError::Diagnostic(error)) => {
                        eprintln!("{error}");
                        buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) if !buffer.is_empty() => {
                buffer.clear();
                interrupt_pending = false;
                println!("^C");
            }
            Err(ReadlineError::Interrupted) if interrupt_pending => break,
            Err(ReadlineError::Interrupted) => {
                println!("(press Ctrl-C again to exit the REPL)");
                interrupt_pending = true;
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(error) => return Err(error.into()),
        }
    }

    if let Some(path) = history {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        editor.save_history(&path)?;
    }
    Ok(())
}

enum Command {
    Evaluate,
    Handled,
    Exit,
}

fn handle_command(line: &str, session: &mut Session) -> Result<Command> {
    match line.trim() {
        "" => Ok(Command::Evaluate),
        ":help" => {
            println!("Commands: :help, :clear, :reset, :quit (or :q)");
            Ok(Command::Handled)
        }
        ":clear" => {
            print!("\x1B[2J\x1B[H");
            io::stdout().flush()?;
            Ok(Command::Handled)
        }
        ":reset" => {
            session.reset();
            println!("REPL state reset.");
            Ok(Command::Handled)
        }
        ":quit" | ":q" => Ok(Command::Exit),
        command if command.starts_with(':') => {
            eprintln!("unknown REPL command '{command}'; type :help");
            Ok(Command::Handled)
        }
        _ => Ok(Command::Evaluate),
    }
}

fn ensure_trailing_newline(source: &str) -> String {
    if source.ends_with('\n') {
        source.to_string()
    } else {
        format!("{source}\n")
    }
}

fn needs_more_input(error: &ParserError) -> bool {
    match &error.kind {
        ParserErrorKind::UnexpectedEof => true,
        ParserErrorKind::ExpectedToken { found, .. }
        | ParserErrorKind::ExpectedTokens { found, .. } => found == "EOF",
        ParserErrorKind::ExpectedExpression { found } => found == "EOF" || found == "Eof",
        _ => false,
    }
}

fn lexer_diagnostic(error: &LexerError, source: &str) -> String {
    diagnostic(&error.kind, error.pos, source)
}

fn parser_diagnostic(error: &ParserError, source: &str) -> String {
    diagnostic(&error.kind, error.pos, source)
}

fn type_diagnostic(error: &TypeError, source: &str) -> String {
    diagnostic(&error.kind, error.span, source)
}

fn runtime_diagnostic(error: &RuntimeError, source: &str) -> String {
    diagnostic(&error.kind, error.span, source)
}

fn diagnostic(kind: &impl std::fmt::Display, span: Span, source: &str) -> String {
    match render_source_span_from_source(source, span) {
        Ok((line, column, snippet)) => {
            format!("error: {kind}\n --> <repl>:{line}:{column}\n{snippet}")
        }
        Err(_) => format!("error: {kind}"),
    }
}

fn history_path() -> Option<PathBuf> {
    let state_dir = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))?;
    Some(state_dir.join("fobos/history"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions_are_evaluated_and_printable() {
        let mut session = Session::new(true);
        let result = session.submit("1 + 2").unwrap();

        assert_eq!(result.value, Value::Int(3));
        assert!(result.output.is_empty());
    }

    #[test]
    fn declarations_persist_between_submissions() {
        let mut session = Session::new(true);
        session.submit("let answer := 41").unwrap();

        assert_eq!(session.submit("answer + 1").unwrap().value, Value::Int(42));
    }

    #[test]
    fn multiline_function_waits_for_end_and_then_persists() {
        let mut session = Session::new(true);
        assert_eq!(
            session
                .submit("fun add(x: Int, y: Int): Int =\n")
                .unwrap_err(),
            SubmitError::Incomplete
        );

        session
            .submit("fun add(x: Int, y: Int): Int =\nreturn x + y\nend")
            .unwrap();
        assert_eq!(session.submit("add(20, 22)").unwrap().value, Value::Int(42));
    }

    #[test]
    fn failed_type_checks_do_not_leak_bindings() {
        let mut session = Session::new(true);
        assert!(matches!(
            session.submit("let invalid: Int = \"no\""),
            Err(SubmitError::Diagnostic(_))
        ));
        assert!(matches!(
            session.submit("invalid"),
            Err(SubmitError::Diagnostic(_))
        ));
    }

    #[test]
    fn runtime_errors_do_not_end_the_session() {
        let mut session = Session::new(true);
        assert!(matches!(
            session.submit("1 / 0"),
            Err(SubmitError::Diagnostic(_))
        ));
        assert_eq!(session.submit("20 + 22").unwrap().value, Value::Int(42));
    }

    #[test]
    fn imports_are_rejected_instead_of_being_silently_ignored() {
        let mut session = Session::new(true);
        let error = session.submit("import std::foo").unwrap_err();

        assert!(
            matches!(error, SubmitError::Diagnostic(message) if message.contains("not supported"))
        );
    }
}
