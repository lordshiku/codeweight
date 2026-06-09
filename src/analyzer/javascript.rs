use anyhow::Result;

use super::{
    classify_lines, complexity_for_body, count_keyword_occurrences, nesting_depth_from_delimiters,
    split_into_functions, AnalyzedSource, Analyzer, Language,
};

/// Analyzer for JavaScript and TypeScript source files.
pub struct JavaScriptAnalyzer;

impl Analyzer for JavaScriptAnalyzer {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn analyze_source(&self, content: &str) -> Result<AnalyzedSource> {
        let lines = classify_lines(content, is_javascript_comment_line);
        let bodies = split_into_functions(content, is_javascript_function_start);
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

fn is_javascript_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

fn is_javascript_function_start(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.starts_with('}')
        || trimmed.starts_with("else")
        || trimmed.starts_with("if")
        || trimmed.starts_with("for")
        || trimmed.starts_with("while")
        || trimmed.starts_with("switch")
        || trimmed.starts_with("catch")
    {
        return false;
    }

    if trimmed.starts_with("function ")
        || trimmed.contains("function(")
        || trimmed.starts_with("async function")
    {
        return true;
    }

    if trimmed.contains("=>") {
        return true;
    }

    if count_keyword_occurrences(trimmed, "function") > 0 && trimmed.contains('(') {
        return true;
    }

    // Method shorthand: name(...) { or name: function(
    let open_paren = trimmed.find('(');
    let open_brace = trimmed.find('{');
    if let (Some(p), Some(b)) = (open_paren, open_brace) {
        if p < b && !trimmed.starts_with("if")
            && !trimmed.starts_with("for")
            && !trimmed.starts_with("while")
            && !trimmed.starts_with("switch")
        {
            let head = &trimmed[..p];
            if head.chars().any(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                return true;
            }
        }
    }

    false
}
