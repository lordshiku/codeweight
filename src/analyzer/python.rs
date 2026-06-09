use anyhow::Result;

use super::{
    classify_lines, complexity_for_body, count_branch_points, python_nesting_depth,
    split_into_functions, AnalyzedSource, Analyzer, Language,
};

/// Analyzer for Python source files.
pub struct PythonAnalyzer;

impl Analyzer for PythonAnalyzer {
    fn language(&self) -> Language {
        Language::Python
    }

    fn analyze_source(&self, content: &str) -> Result<AnalyzedSource> {
        let lines = classify_lines(content, is_python_comment_line);
        let bodies = split_into_functions(content, is_python_function_start);
        let function_count = bodies.len();
        let complexities: Vec<usize> = bodies.iter().map(|b| complexity_for_body(b)).collect();
        let cyclomatic_complexity: usize = complexities.iter().sum();
        let max_function_complexity = complexities.into_iter().max().unwrap_or(0);

        // Include module-level branch points not inside any function.
        let module_branches: usize = if function_count == 0 {
            content
                .lines()
                .map(count_branch_points)
                .sum()
        } else {
            0
        };

        Ok(AnalyzedSource {
            lines,
            function_count,
            cyclomatic_complexity: cyclomatic_complexity + module_branches,
            max_function_complexity,
            max_nesting_depth: python_nesting_depth(content),
        })
    }
}

fn is_python_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#')
}

fn is_python_function_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("def ") || trimmed.starts_with("async def ")
}
