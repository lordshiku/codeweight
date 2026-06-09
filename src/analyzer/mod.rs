mod java;
mod javascript;
mod python;
mod rust;

use anyhow::Result;

pub use java::JavaAnalyzer;
pub use javascript::JavaScriptAnalyzer;
pub use python::PythonAnalyzer;
pub use rust::RustAnalyzer;

use crate::metrics::{FileMetrics, LineCounts};
use crate::scorer::apply_score;

/// Supported source languages for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Python (`.py`).
    Python,
    /// JavaScript and TypeScript (`.js`, `.ts`, etc.).
    JavaScript,
    /// Rust (`.rs`).
    Rust,
    /// Java (`.java`).
    Java,
}

impl Language {
    /// All supported languages.
    pub const ALL: [Language; 4] = [
        Language::Python,
        Language::JavaScript,
        Language::Rust,
        Language::Java,
    ];

    /// Short identifier used in output and CLI filtering.
    pub fn id(self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::Rust => "rust",
            Language::Java => "java",
        }
    }

    /// CLI alias accepted by the `--lang` flag.
    pub fn cli_alias(self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::JavaScript => "js",
            Language::Rust => "rust",
            Language::Java => "java",
        }
    }

    /// Parse a CLI language filter value.
    pub fn from_cli(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "python" | "py" => Some(Language::Python),
            "js" | "javascript" | "typescript" | "ts" => Some(Language::JavaScript),
            "rust" | "rs" => Some(Language::Rust),
            "java" => Some(Language::Java),
            "all" => None,
            _ => None,
        }
    }

    /// Detect language from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "py" => Some(Language::Python),
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "rs" => Some(Language::Rust),
            "java" => Some(Language::Java),
            _ => None,
        }
    }
}

/// Contract for language-specific static analyzers.
pub trait Analyzer {
    /// Language identifier reported in metrics output.
    fn language(&self) -> Language;

    /// Analyze source text and return populated metrics (without path or score).
    fn analyze_source(&self, content: &str) -> Result<AnalyzedSource>;
}

/// Intermediate analysis result before path and scoring are attached.
#[derive(Debug, Clone)]
pub struct AnalyzedSource {
    /// Classified line counts.
    pub lines: LineCounts,
    /// Number of functions or methods detected.
    pub function_count: usize,
    /// Total cyclomatic complexity across functions.
    pub cyclomatic_complexity: usize,
    /// Highest complexity among individual functions.
    pub max_function_complexity: usize,
    /// Maximum block nesting depth.
    pub max_nesting_depth: usize,
}

/// Analyze a file's contents with the appropriate language analyzer.
pub fn analyze_content(language: Language, content: &str) -> Result<FileMetrics> {
    let analyzed = match language {
        Language::Python => PythonAnalyzer.analyze_source(content)?,
        Language::JavaScript => JavaScriptAnalyzer.analyze_source(content)?,
        Language::Rust => RustAnalyzer.analyze_source(content)?,
        Language::Java => JavaAnalyzer.analyze_source(content)?,
    };

    Ok(build_metrics(language, "", analyzed))
}

/// Build a [`FileMetrics`] value from analyzed source data.
pub fn build_metrics(language: Language, path: &str, analyzed: AnalyzedSource) -> FileMetrics {
    let mut metrics = FileMetrics {
        path: path.to_string(),
        language: language.id().to_string(),
        total_lines: 0,
        blank_lines: 0,
        comment_lines: 0,
        code_lines: 0,
        function_count: analyzed.function_count,
        cyclomatic_complexity: analyzed.cyclomatic_complexity,
        max_function_complexity: analyzed.max_function_complexity,
        max_nesting_depth: analyzed.max_nesting_depth,
        maintainability_score: 0.0,
    };

    analyzed.lines.apply_to(&mut metrics);
    apply_score(&mut metrics);
    metrics
}

/// Count cyclomatic complexity contributions on a single line of code.
pub fn count_branch_points(line: &str) -> usize {
    let trimmed = strip_inline_comment(line);
    if trimmed.is_empty() {
        return 0;
    }

    let mut count = 0usize;

    // Longer keywords first; normalize `else if` so inner `if` is not double-counted.
    for keyword in [
        "else if", "elif", "for", "while", "match", "case", "switch", "catch", "&&", "||",
    ] {
        count += count_keyword_occurrences(trimmed, keyword);
    }

    let without_else_if = trimmed.replace("else if", "         ");
    count += count_keyword_occurrences(&without_else_if, "if");
    count += count_standalone_else(trimmed);

    count += count_ternary_operators(trimmed);
    count
}

/// Count `else` branches that are not part of an `else if` chain.
fn count_standalone_else(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut count = 0;
    let mut i = 0;

    while i + 4 <= bytes.len() {
        if line[i..].starts_with("else") {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1] as char);
            let after_index = i + 4;
            let after_ok =
                after_index >= bytes.len() || !is_ident_char(bytes[after_index] as char);

            let is_else_if = line[i..].starts_with("else if");
            if before_ok && after_ok && !is_else_if {
                count += 1;
            }
        }
        i += 1;
    }

    count
}

/// Strip trailing inline comments while preserving string literals (approximate).
pub fn strip_inline_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if ch == '\'' && !in_double {
            in_single = !in_single;
        } else if ch == '"' && !in_single {
            in_double = !in_double;
        } else if !in_single && !in_double {
            if ch == '#' {
                return &line[..i];
            }
            if ch == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                return &line[..i];
            }
        }

        i += 1;
    }

    line
}

/// Count standalone keyword or operator occurrences outside of identifiers.
pub fn count_keyword_occurrences(line: &str, keyword: &str) -> usize {
    let bytes = line.as_bytes();
    let key_bytes = keyword.as_bytes();
    let mut count = 0;
    let mut i = 0;

    while i + key_bytes.len() <= bytes.len() {
        if line[i..].starts_with(keyword) {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1] as char);
            let after_index = i + key_bytes.len();
            let after_ok =
                after_index >= bytes.len() || !is_ident_char(bytes[after_index] as char);

            if before_ok && after_ok {
                count += 1;
            }
        }
        i += 1;
    }

    count
}

fn count_ternary_operators(line: &str) -> usize {
    line.chars().filter(|&c| c == '?').count()
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Track nesting depth using opening and closing delimiters.
pub fn nesting_depth_from_delimiters(content: &str, open: char, close: char) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;

    for ch in content.chars() {
        if ch == open {
            depth += 1;
            max_depth = max_depth.max(depth);
        } else if ch == close && depth > 0 {
            depth -= 1;
        }
    }

    max_depth
}

/// Track Python-style indentation nesting depth.
pub fn python_nesting_depth(content: &str) -> usize {
    let mut max_depth = 0usize;

    for line in content.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }

        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let depth = indent / 4 + 1;
        max_depth = max_depth.max(depth);
    }

    max_depth
}

/// Split source into function bodies using line-based detection.
pub fn split_into_functions(content: &str, is_function_start: fn(&str) -> bool) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut bodies = Vec::new();
    let mut current = Vec::new();
    let mut in_function = false;

    for line in lines {
        if is_function_start(line) {
            if in_function && !current.is_empty() {
                bodies.push(current.join("\n"));
                current.clear();
            }
            in_function = true;
        }

        if in_function {
            current.push(line);
        }
    }

    if in_function && !current.is_empty() {
        bodies.push(current.join("\n"));
    }

    bodies
}

/// Compute cyclomatic complexity for a function body (base complexity of 1).
pub fn complexity_for_body(body: &str) -> usize {
    let branch_points: usize = body
        .lines()
        .map(count_branch_points)
        .sum();
    1 + branch_points
}

/// Classify lines as blank, comment, or code using language-specific rules.
pub fn classify_lines(content: &str, is_comment_line: fn(&str) -> bool) -> LineCounts {
    let mut counts = LineCounts::default();

    for line in content.lines() {
        counts.total += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            counts.blank += 1;
        } else if is_comment_line(line) {
            counts.comment += 1;
        } else {
            counts.code += 1;
        }
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_branch_keywords() {
        let line = "if x > 0 && y < 1 || z == 2 {";
        assert!(count_branch_points(line) >= 3);
    }

    #[test]
    fn ignores_comments_in_strip() {
        let line = "let x = 1; // not an if";
        assert_eq!(strip_inline_comment(line), "let x = 1; ");
    }
}
