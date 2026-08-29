//! Independent evaluation primitives for generated project-foundation packages.

pub mod cli;
pub mod corpus;
pub mod error;
pub mod input;

pub use corpus::{Artifact, Corpus, CorpusLimits, collect_corpus};
pub use error::EvaluatorError;
pub use input::PackageInput;
