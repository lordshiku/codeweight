use crate::metrics::FileMetrics;

/// Input values used to compute a maintainability score.
#[derive(Debug, Clone, Copy)]
pub struct ScoreInput {
    /// Total cyclomatic complexity for the file.
    pub cyclomatic_complexity: usize,
    /// Number of functions detected in the file.
    pub function_count: usize,
    /// Maximum nesting depth in the file.
    pub max_nesting_depth: usize,
    /// Number of code lines in the file.
    pub code_lines: usize,
}

impl ScoreInput {
    /// Build score input from an analyzed file's metrics.
    pub fn from_metrics(metrics: &FileMetrics) -> Self {
        Self {
            cyclomatic_complexity: metrics.cyclomatic_complexity,
            function_count: metrics.function_count,
            max_nesting_depth: metrics.max_nesting_depth,
            code_lines: metrics.code_lines,
        }
    }
}

/// Compute a maintainability score between 0 and 100.
///
/// The score starts at 100 and subtracts penalties for complexity density,
/// deep nesting, large file size, and files with substantial code but no
/// detected functions.
pub fn compute_maintainability_score(input: ScoreInput) -> f64 {
    let function_divisor = input.function_count.max(1) as f64;
    let complexity_density = input.cyclomatic_complexity as f64 / function_divisor;

    let complexity_penalty = (complexity_density * 5.0).min(45.0);
    let nesting_penalty = (input.max_nesting_depth as f64 * 4.0).min(25.0);

    let length_factor = if input.code_lines == 0 {
        0.0
    } else {
        ((1.0 + input.code_lines as f64).ln()) / ((1.0 + 500.0_f64).ln())
    };
    let length_penalty = (length_factor * 20.0).min(20.0);

    let sparsity_penalty = if input.code_lines > 50 && input.function_count == 0 {
        10.0
    } else {
        0.0
    };

    let raw = 100.0 - complexity_penalty - nesting_penalty - length_penalty - sparsity_penalty;
    raw.clamp(0.0, 100.0)
}

/// Apply the maintainability score to a [`FileMetrics`] instance in place.
pub fn apply_score(metrics: &mut FileMetrics) {
    metrics.maintainability_score = compute_maintainability_score(ScoreInput::from_metrics(metrics));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_file_scores_high() {
        let score = compute_maintainability_score(ScoreInput {
            cyclomatic_complexity: 2,
            function_count: 2,
            max_nesting_depth: 1,
            code_lines: 20,
        });
        assert!(score > 80.0);
    }

    #[test]
    fn complex_file_scores_lower() {
        let score = compute_maintainability_score(ScoreInput {
            cyclomatic_complexity: 40,
            function_count: 2,
            max_nesting_depth: 6,
            code_lines: 800,
        });
        assert!(score < 50.0);
    }
}
