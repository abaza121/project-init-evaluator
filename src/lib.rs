//! Independent evaluation primitives for generated project-foundation packages.

pub mod cli;
pub mod compare;
pub mod corpus;
pub mod error;
pub mod evaluate;
pub mod extract;
pub mod input;
pub mod metrics;
pub mod report;
pub mod score;

pub use compare::{
    ComparisonInput, ComparisonResult, compare_packages, render_comparison_markdown,
};
pub use corpus::{Artifact, Corpus, CorpusLimits, collect_corpus};
pub use error::EvaluatorError;
pub use evaluate::{build_validation_report, evaluate_package, write_validation_reports};
pub use extract::{
    EntityKind, EvidenceRef, ExtractedEntity, Extraction, TraceLink, extract_entities,
};
pub use input::PackageInput;
pub use metrics::{DeterministicMetrics, calculate_metrics};
pub use report::{
    DimensionScore, Dimensions, Finding, Severity, ValidationReport, render_validation_json,
    render_validation_markdown,
};
pub use score::score_corpus;
