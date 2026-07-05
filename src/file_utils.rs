use std::{
    ffi::OsString,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::source::{Span, SrcPos};

pub fn create_expected_by_ext(file_path: &Path, extension: &str) -> anyhow::Result<PathBuf> {
    // here we would be iterating over the dir entries, so it would always have a parent
    let parent = file_path.parent().unwrap();

    // also i guess it wouldnt end in '..'
    let file_name = file_path.file_name().unwrap();

    let new_dir = parent.join("expected");
    fs::create_dir_all(&new_dir)?;

    let mut new_file_name = OsString::from(file_name);
    new_file_name.push(extension);

    Ok(new_dir.join(new_file_name))
}

pub fn read_line_from(file_path: &Path, span: Span) -> (usize, usize, String) {
    let mut result = String::new();
    let content = read_to_string(file_path).unwrap();

    let is_start = span.start.col == 0;
    let mut span = if is_start {
        Span {
            start: SrcPos {
                line: span.start.line - 1,
                col: 0,
                idx: 0,
            },
            end: span.end,
        }
    } else {
        span
    };

    let line = content
        .lines()
        .nth(span.start.line - 1)
        .with_context(|| {
            format!(
                "invalid source span: line {} does not exist in source",
                span.start.line - 1
            )
        })
        .unwrap();

    if is_start {
        span.end.line = span.start.line;
        span.start.col = line.len() + 2;
        span.end.col = line.len() + 2;
    }

    result.push_str(line);
    result.push('\n');

    result.push('|');
    for _ in 0..=span.start.col + 2 {
        result.push(' ');
    }

    let count = if span.start.line == span.end.line {
        span.end.col - span.start.col
    } else {
        line.len() - span.start.col
    };

    for _ in 0..=count {
        result.push('^');
    }

    (span.start.line, span.start.col, result)
}
