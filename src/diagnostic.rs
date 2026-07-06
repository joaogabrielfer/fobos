use std::path::Path;

use crate::source::Span;

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
