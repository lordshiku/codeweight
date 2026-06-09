//! codeweight — static code complexity analyzer.
//!
//! Scans source files and reports line metrics, function counts, cyclomatic
//! complexity, nesting depth, and a maintainability score.

pub mod analyzer;
pub mod metrics;
pub mod reporter;
pub mod scorer;
pub mod walker;
