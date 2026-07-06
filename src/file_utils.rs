use std::{
    ffi::OsString,
    fs::{self},
    path::{Path, PathBuf},
};

use crate::source::Span;

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

pub fn render_source_span(file_path: &Path, span: Span) -> anyhow::Result<(usize, usize, String)> {
    let content = std::fs::read_to_string(file_path)?;

    let line_idx = span.start.line.saturating_sub(1);

    // `split('\n')` keeps a final empty line better than `lines()`.
    let lines: Vec<&str> = content.split('\n').collect();

    let line = lines.get(line_idx).copied().ok_or_else(|| {
        anyhow::anyhow!(
            "invalid source span: line {} does not exist in source",
            span.start.line
        )
    })?;

    let line_len = line.chars().count();

    let start_col = span.start.col.max(1).min(line_len + 1);

    let width = if span.start.line == span.end.line {
        span.end.col.saturating_sub(span.start.col).max(1)
    } else {
        line_len.saturating_sub(start_col - 1).max(1)
    };

    let mut result = String::new();

    result.push_str(line);
    result.push('\n');

    result.push(' ');
    result.push(' ');
    result.push('|');
    result.push_str("   ");

    for _ in 1..start_col {
        result.push(' ');
    }

    for _ in 0..width {
        result.push('^');
    }

    Ok((span.start.line, span.start.col, result))
}
