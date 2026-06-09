use serde::Serialize;

/// Metrics collected for a single source file.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FileMetrics {
    /// Path to the analyzed file (relative to the scan root when possible).
    pub path: String,
    /// Detected language identifier (e.g. `python`, `javascript`).
    pub language: String,
    /// Total number of lines in the file.
    pub total_lines: usize,
    /// Number of blank lines.
    pub blank_lines: usize,
    /// Number of comment-only lines.
    pub comment_lines: usize,
    /// Number of lines containing executable code.
    pub code_lines: usize,
    /// Number of function or method definitions detected.
    pub function_count: usize,
    /// Sum of cyclomatic complexity across all functions in the file.
    pub cyclomatic_complexity: usize,
    /// Highest cyclomatic complexity among individual functions.
    pub max_function_complexity: usize,
    /// Deepest nesting level of blocks/braces in the file.
    pub max_nesting_depth: usize,
    /// Computed maintainability score from 0 (worst) to 100 (best).
    pub maintainability_score: f64,
}

/// Aggregate metrics across multiple analyzed files.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AggregateMetrics {
    /// Number of files included in the aggregate.
    pub file_count: usize,
    /// Sum of total lines across all files.
    pub total_lines: usize,
    /// Sum of blank lines across all files.
    pub blank_lines: usize,
    /// Sum of comment lines across all files.
    pub comment_lines: usize,
    /// Sum of code lines across all files.
    pub code_lines: usize,
    /// Sum of function counts across all files.
    pub function_count: usize,
    /// Sum of cyclomatic complexity across all files.
    pub cyclomatic_complexity: usize,
    /// Highest per-function complexity observed in any file.
    pub max_function_complexity: usize,
    /// Highest nesting depth observed in any file.
    pub max_nesting_depth: usize,
    /// Average maintainability score across all files.
    pub avg_maintainability_score: f64,
}

impl AggregateMetrics {
    /// Build aggregate metrics from a slice of per-file results.
    pub fn from_files(files: &[FileMetrics]) -> Self {
        let file_count = files.len();
        let total_lines: usize = files.iter().map(|f| f.total_lines).sum();
        let blank_lines: usize = files.iter().map(|f| f.blank_lines).sum();
        let comment_lines: usize = files.iter().map(|f| f.comment_lines).sum();
        let code_lines: usize = files.iter().map(|f| f.code_lines).sum();
        let function_count: usize = files.iter().map(|f| f.function_count).sum();
        let cyclomatic_complexity: usize = files.iter().map(|f| f.cyclomatic_complexity).sum();
        let max_function_complexity = files
            .iter()
            .map(|f| f.max_function_complexity)
            .max()
            .unwrap_or(0);
        let max_nesting_depth = files
            .iter()
            .map(|f| f.max_nesting_depth)
            .max()
            .unwrap_or(0);
        let avg_maintainability_score = if file_count == 0 {
            0.0
        } else {
            files
                .iter()
                .map(|f| f.maintainability_score)
                .sum::<f64>()
                / file_count as f64
        };

        Self {
            file_count,
            total_lines,
            blank_lines,
            comment_lines,
            code_lines,
            function_count,
            cyclomatic_complexity,
            max_function_complexity,
            max_nesting_depth,
            avg_maintainability_score,
        }
    }
}

/// Raw line classification counts produced during analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineCounts {
    /// Total lines processed.
    pub total: usize,
    /// Blank lines.
    pub blank: usize,
    /// Comment-only lines.
    pub comment: usize,
    /// Lines containing code.
    pub code: usize,
}

impl LineCounts {
    /// Convert raw line counts into the line fields of [`FileMetrics`].
    pub fn apply_to(&self, metrics: &mut FileMetrics) {
        metrics.total_lines = self.total;
        metrics.blank_lines = self.blank;
        metrics.comment_lines = self.comment;
        metrics.code_lines = self.code;
    }
}
