use anyhow::Result;

use super::{
    classify_lines, complexity_for_body, nesting_depth_from_delimiters, split_into_functions,
    AnalyzedSource, Analyzer, Language,
};

/// Analyzer for Rust source files.
pub struct RustAnalyzer;

impl Analyzer for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn analyze_source(&self, content: &str) -> Result<AnalyzedSource> {
        let lines = classify_lines(content, is_rust_comment_line);
        let bodies = split_into_functions(content, is_rust_function_start);
        let function_count = bodies.len();
        let complexities: Vec<usize> = bodies.iter().map(|b| complexity_for_body(b)).collect();
        let cyclomatic_complexity: usize = complexities.iter().sum();
        let max_function_complexity = complexities.into_iter().max().unwrap_or(0);

        Ok(AnalyzedSource {
            lines,
            function_count,
            cyclomatic_complexity,
            max_function_complexity,
            max_nesting_depth: nesting_depth_from_delimiters(content, '{', '}'),
        })
    }
}

fn is_rust_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

fn is_rust_function_start(line: &str) -> bool {
    let trimmed = line.trim_start();

    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("pub unsafe fn ")
        || trimmed.starts_with("const fn ")
        || trimmed.starts_with("pub const fn ")
}
