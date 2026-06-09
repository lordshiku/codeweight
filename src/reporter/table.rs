use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};

use crate::metrics::{AggregateMetrics, FileMetrics};

/// Render metrics as a terminal table with color-coded complexity.
pub fn render_table(
    files: &[FileMetrics],
    aggregate: &AggregateMetrics,
    include_files: bool,
) -> String {
    let mut output = String::new();

    if include_files {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "File",
                "Lang",
                "Lines",
                "Code",
                "Fns",
                "Complexity",
                "Max Fn",
                "Nest",
                "Score",
            ]);

        for file in files {
            table.add_row(vec![
                Cell::new(&file.path),
                Cell::new(&file.language),
                Cell::new(file.total_lines),
                Cell::new(file.code_lines),
                Cell::new(file.function_count),
                Cell::new(color_complexity(file.cyclomatic_complexity)),
                Cell::new(color_complexity(file.max_function_complexity)),
                Cell::new(file.max_nesting_depth),
                Cell::new(color_score(file.maintainability_score)),
            ]);
        }

        output.push_str(&table.to_string());
        output.push('\n');
        output.push('\n');
    }

    let mut summary = Table::new();
    summary
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Files",
            "Lines",
            "Code",
            "Fns",
            "Complexity",
            "Max Fn",
            "Max Nest",
            "Avg Score",
        ]);

    summary.add_row(vec![
        Cell::new(aggregate.file_count),
        Cell::new(aggregate.total_lines),
        Cell::new(aggregate.code_lines),
        Cell::new(aggregate.function_count),
        Cell::new(color_complexity(aggregate.cyclomatic_complexity)),
        Cell::new(color_complexity(aggregate.max_function_complexity)),
        Cell::new(aggregate.max_nesting_depth),
        Cell::new(color_score(aggregate.avg_maintainability_score)),
    ]);

    if include_files {
        output.push_str("Summary\n");
    } else {
        output.push_str("Aggregate Summary\n");
    }
    output.push_str(&summary.to_string());
    output
}

fn color_complexity(value: usize) -> String {
    let text = value.to_string();
    if value <= 10 {
        text.green().to_string()
    } else if value <= 25 {
        text.yellow().to_string()
    } else {
        text.red().to_string()
    }
}

fn color_score(value: f64) -> String {
    let text = format!("{value:.1}");
    if value >= 70.0 {
        text.green().to_string()
    } else if value >= 40.0 {
        text.yellow().to_string()
    } else {
        text.red().to_string()
    }
}
