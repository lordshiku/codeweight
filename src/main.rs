use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codeweight::analyzer;
use codeweight::metrics::AggregateMetrics;
use codeweight::reporter::{prepare_files, render_report, SortBy};
use codeweight::walker::{collect_metrics, WalkOptions};

/// Static code complexity analyzer for Python, JavaScript/TypeScript, Rust, and Java.
#[derive(Debug, Parser)]
#[command(
    name = "codeweight",
    version,
    about = "Analyze source code complexity and maintainability metrics",
    long_about = "codeweight scans directories of source files and reports line counts, \
                  function counts, cyclomatic complexity, nesting depth, and a maintainability \
                  score per file and in aggregate."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Analyze source files under a path and report per-file metrics plus a summary.
    Analyze {
        /// File or directory to analyze.
        path: PathBuf,

        /// Restrict analysis to a single language (`python`, `js`, `rust`, `java`, or `all`).
        #[arg(long, value_name = "LANG", default_value = "all")]
        lang: String,

        /// Emit machine-readable JSON instead of a terminal table.
        #[arg(long)]
        json: bool,

        /// Only include files with cyclomatic complexity >= this value.
        #[arg(long, value_name = "N")]
        min_complexity: Option<usize>,

        /// Sort per-file results by the given metric.
        #[arg(long, value_name = "FIELD", default_value = "lines")]
        sort_by: String,

        /// Print messages when skipping unsupported or unreadable files.
        #[arg(long, short)]
        verbose: bool,
    },

    /// Report aggregate metrics only (no per-file breakdown).
    Summary {
        /// File or directory to analyze.
        path: PathBuf,

        /// Restrict analysis to a single language (`python`, `js`, `rust`, `java`, or `all`).
        #[arg(long, value_name = "LANG", default_value = "all")]
        lang: String,

        /// Emit machine-readable JSON instead of a terminal table.
        #[arg(long)]
        json: bool,

        /// Print messages when skipping unsupported or unreadable files.
        #[arg(long, short)]
        verbose: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            path,
            lang,
            json,
            min_complexity,
            sort_by,
            verbose,
        } => {
            let language_filter = parse_language_filter(&lang)?;
            let sort_by = parse_sort_by(&sort_by)?;
            let files = collect_metrics(&WalkOptions {
                root: path,
                language_filter,
                verbose,
            })?;

            let files = prepare_files(files, min_complexity, sort_by);
            let aggregate = AggregateMetrics::from_files(&files);
            let output = render_report(&files, &aggregate, json, true);
            print!("{output}");
        }
        Commands::Summary {
            path,
            lang,
            json,
            verbose,
        } => {
            let language_filter = parse_language_filter(&lang)?;
            let files = collect_metrics(&WalkOptions {
                root: path,
                language_filter,
                verbose,
            })?;

            let aggregate = AggregateMetrics::from_files(&files);
            let output = render_report(&[], &aggregate, json, false);
            print!("{output}");
        }
    }

    Ok(())
}

fn parse_language_filter(value: &str) -> Result<Option<analyzer::Language>> {
    if value.eq_ignore_ascii_case("all") {
        return Ok(None);
    }

    let language = analyzer::Language::from_cli(value).with_context(|| {
        format!("invalid language '{value}'; expected python, js, rust, java, or all")
    })?;
    Ok(Some(language))
}

fn parse_sort_by(value: &str) -> Result<SortBy> {
    SortBy::from_cli(value).with_context(|| {
        format!("invalid sort field '{value}'; expected lines, complexity, functions, or score")
    })
}
