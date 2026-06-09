use anyhow::Result;

use super::{
    classify_lines, complexity_for_body, nesting_depth_from_delimiters, split_into_functions,
    AnalyzedSource, Analyzer, Language,
};

/// Analyzer for Java source files.
pub struct JavaAnalyzer;

impl Analyzer for JavaAnalyzer {
    fn language(&self) -> Language {
        Language::Java
    }

    fn analyze_source(&self, content: &str) -> Result<AnalyzedSource> {
        let lines = classify_lines(content, is_java_comment_line);
        let bodies = split_into_functions(content, is_java_function_start);
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

fn is_java_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

fn is_java_function_start(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.starts_with('@') || trimmed.ends_with(';') || trimmed.ends_with('{') && !trimmed.contains('(')
    {
        return false;
    }

    if !(trimmed.contains('(') && trimmed.contains(')')) {
        return false;
    }

    let keywords = [
        "public", "private", "protected", "static", "final", "synchronized", "native", "abstract",
        "void", "int", "long", "double", "float", "boolean", "char", "byte", "short", "String",
    ];

    keywords.iter().any(|kw| trimmed.contains(kw)) && trimmed.contains('(')
}
