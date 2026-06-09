use serde::Serialize;

use crate::metrics::{AggregateMetrics, FileMetrics};

/// JSON report envelope for machine-readable output.
#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    /// Per-file metrics (omitted in summary-only mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<&'a [FileMetrics]>,
    /// Aggregate metrics across all analyzed files.
    aggregate: AggregateMetrics,
}

/// Serialize metrics as pretty-printed JSON.
pub fn render_json(
    files: &[FileMetrics],
    aggregate: &AggregateMetrics,
    include_files: bool,
) -> String {
    let report = JsonReport {
        files: if include_files { Some(files) } else { None },
        aggregate: aggregate.clone(),
    };

    match serde_json::to_string_pretty(&report) {
        Ok(json) => json,
        Err(err) => format!("{{\"error\": \"failed to serialize report: {err}\"}}"),
    }
}
