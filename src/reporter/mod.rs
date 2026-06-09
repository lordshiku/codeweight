mod json;
mod table;

pub use json::render_json;
pub use table::render_table;

use crate::metrics::{AggregateMetrics, FileMetrics};

/// Sort key for ordering analyzed files in output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by total lines (descending).
    Lines,
    /// Sort by cyclomatic complexity (descending).
    Complexity,
    /// Sort by function count (descending).
    Functions,
    /// Sort by maintainability score (ascending — worst files first).
    Score,
}

impl SortBy {
    /// Parse a CLI sort value.
    pub fn from_cli(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "lines" => Some(SortBy::Lines),
            "complexity" => Some(SortBy::Complexity),
            "functions" => Some(SortBy::Functions),
            "score" => Some(SortBy::Score),
            _ => None,
        }
    }
}

/// Filter and sort file metrics according to CLI options.
pub fn prepare_files(
    mut files: Vec<FileMetrics>,
    min_complexity: Option<usize>,
    sort_by: SortBy,
) -> Vec<FileMetrics> {
    if let Some(min) = min_complexity {
        files.retain(|f| f.cyclomatic_complexity >= min);
    }

    match sort_by {
        SortBy::Lines => files.sort_by_key(|file| std::cmp::Reverse(file.total_lines)),
        SortBy::Complexity => {
            files.sort_by_key(|file| std::cmp::Reverse(file.cyclomatic_complexity))
        }
        SortBy::Functions => files.sort_by_key(|file| std::cmp::Reverse(file.function_count)),
        SortBy::Score => files.sort_by(|a, b| {
            a.maintainability_score
                .partial_cmp(&b.maintainability_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    files
}

/// Render analysis results in the requested output format.
pub fn render_report(
    files: &[FileMetrics],
    aggregate: &AggregateMetrics,
    json: bool,
    include_files: bool,
) -> String {
    if json {
        render_json(files, aggregate, include_files)
    } else {
        render_table(files, aggregate, include_files)
    }
}
