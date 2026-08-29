//! Independent evaluation primitives for generated project-foundation packages.

pub mod cli;
pub mod corpus;
pub mod error;
pub mod extract;
pub mod input;
pub mod metrics;

pub use corpus::{Artifact, Corpus, CorpusLimits, collect_corpus};
pub use error::EvaluatorError;
pub use extract::{
    EntityKind, EvidenceRef, ExtractedEntity, Extraction, TraceLink, extract_entities,
};
pub use input::PackageInput;
pub use metrics::{DeterministicMetrics, calculate_metrics};
