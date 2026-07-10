use crate::source::Span;

use anyhow::Context;
use colored::Colorize;
use std::path::Path;

const TAB_WIDTH: usize = 4;

pub fn render_source_span(file_path: &Path, span: Span) -> anyhow::Result<(usize, usize, String)> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("failed to read source file '{}'", file_path.display()))?;

    // Unlike `lines()`, this preserves a final empty line.
    let lines: Vec<&str> = content.split('\n').collect();

    let current_idx = span.start.line.saturating_sub(1);

    let current_line = lines.get(current_idx).copied().ok_or_else(|| {
        anyhow::anyhow!(
            "invalid source span: line {} does not exist in source",
            span.start.line
        )
    })?;

    let first_idx = current_idx.saturating_sub(1);
    let last_idx = (current_idx + 1).min(lines.len().saturating_sub(1));

    let line_number_width = (last_idx + 1).to_string().len();

    let current_line_char_count = current_line.chars().count();

    // Source columns are assumed to be 1-based character columns.
    let start_col = span.start.col.max(1).min(current_line_char_count + 1);

    let end_col = if span.start.line == span.end.line {
        span.end
            .col
            .max(start_col + 1)
            .min(current_line_char_count + 1)
    } else {
        current_line_char_count + 1
    };

    let visual_start = visual_width_until(current_line, start_col, TAB_WIDTH);

    let visual_end = visual_width_until(current_line, end_col, TAB_WIDTH);

    let underline_width = visual_end.saturating_sub(visual_start).max(1);

    let mut result = format!("{:>line_number_width$} {}\n", "", "|".cyan());

    for (idx, line) in lines.iter().enumerate().take(last_idx + 1).skip(first_idx) {
        let line_number = idx + 1;
        let rendered_line = expand_tabs(line, TAB_WIDTH);

        result.push_str(&format!(
            "{:>line_number_width$} {}   {rendered_line}\n",
            line_number.to_string().cyan(),
            "|".cyan(),
        ));

        if idx == current_idx {
            result.push_str(&format!("{:>line_number_width$} {}   ", "", "|".cyan(),));

            result.push_str(&" ".repeat(visual_start));
            result.push_str(&"^".repeat(underline_width).red().to_string());
            result.push('\n');
        }
    }

    result.pop();

    Ok((span.start.line, span.start.col, result))
}

/// Expands tabs using fixed tab stops.
///
/// A tab advances to the next multiple of `tab_width`, rather than always
/// inserting exactly `tab_width` spaces.
fn expand_tabs(line: &str, tab_width: usize) -> String {
    let mut result = String::new();
    let mut visual_col = 0;

    for c in line.chars() {
        if c == '\t' {
            let spaces = spaces_until_next_tab_stop(visual_col, tab_width);
            result.push_str(&" ".repeat(spaces));
            visual_col += spaces;
        } else {
            result.push(c);
            visual_col += 1;
        }
    }

    result
}

/// Returns the visual width of all characters before a 1-based source column.
///
/// For example, `source_col == 1` means no characters come before the span,
/// so this returns 0.
fn visual_width_until(line: &str, source_col: usize, tab_width: usize) -> usize {
    let chars_to_take = source_col.saturating_sub(1);

    let mut visual_col = 0;

    for c in line.chars().take(chars_to_take) {
        if c == '\t' {
            visual_col += spaces_until_next_tab_stop(visual_col, tab_width);
        } else {
            visual_col += 1;
        }
    }

    visual_col
}

fn spaces_until_next_tab_stop(visual_col: usize, tab_width: usize) -> usize {
    let remainder = visual_col % tab_width;

    if remainder == 0 {
        tab_width
    } else {
        tab_width - remainder
    }
}
