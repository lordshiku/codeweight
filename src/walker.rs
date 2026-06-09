use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::analyzer::{
    build_metrics, AnalyzedSource, Analyzer, JavaAnalyzer, JavaScriptAnalyzer, Language,
    PythonAnalyzer, RustAnalyzer,
};
use crate::metrics::FileMetrics;

/// Options controlling directory traversal and analysis.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    /// Root path to scan (file or directory).
    pub root: PathBuf,
    /// Optional language filter; `None` means all supported languages.
    pub language_filter: Option<Language>,
    /// Print warnings for skipped unsupported files.
    pub verbose: bool,
}

/// Collect and analyze all supported source files under a path.
pub fn collect_metrics(options: &WalkOptions) -> Result<Vec<FileMetrics>> {
    let root = &options.root;
    anyhow::ensure!(
        root.exists(),
        "path does not exist: {} (provide a readable file or directory to analyze)",
        root.display()
    );

    let mut metrics = Vec::new();

    if root.is_file() {
        if let Some(file_metrics) = analyze_path(root, root, options)? {
            metrics.push(file_metrics);
        }
        return Ok(metrics);
    }

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_metrics) = analyze_path(path, root, options)? {
                metrics.push(file_metrics);
            }
        }
    }

    Ok(metrics)
}

fn analyze_path(
    path: &Path,
    root: &Path,
    options: &WalkOptions,
) -> Result<Option<FileMetrics>> {
    let extension = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => {
            log_skipped(path, options.verbose, "no file extension");
            return Ok(None);
        }
    };

    let language = match Language::from_extension(extension) {
        Some(lang) => lang,
        None => {
            log_skipped(path, options.verbose, "unsupported file type");
            return Ok(None);
        }
    };

    if let Some(filter) = options.language_filter {
        if language != filter {
            return Ok(None);
        }
    }

    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("warning: skipping unreadable file {}: {err}", path.display());
            return Ok(None);
        }
    };

    let analyzed = analyze_with_language(language, &content)
        .with_context(|| format!("failed to analyze {}", path.display()))?;

    let display_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    Ok(Some(build_metrics(
        language,
        &display_path,
        analyzed,
    )))
}

fn analyze_with_language(language: Language, content: &str) -> Result<AnalyzedSource> {
    match language {
        Language::Python => PythonAnalyzer.analyze_source(content),
        Language::JavaScript => JavaScriptAnalyzer.analyze_source(content),
        Language::Rust => RustAnalyzer.analyze_source(content),
        Language::Java => JavaAnalyzer.analyze_source(content),
    }
}

fn log_skipped(path: &Path, verbose: bool, reason: &str) {
    if verbose {
        eprintln!("skipping {} ({reason})", path.display());
    }
}
