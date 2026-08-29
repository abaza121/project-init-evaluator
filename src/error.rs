use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;

/// Describes recoverable validation, discovery, and decoding failures.
#[derive(Debug)]
pub enum EvaluatorError {
    /// An input path did not identify the required regular file or directory.
    InvalidPath {
        /// Path that failed validation.
        path: PathBuf,
        /// Human-readable path type expected by the evaluator.
        expected: &'static str,
    },
    /// An output directory would overlap the read-only submitted package.
    UnsafeOutputOverlap {
        /// Requested output directory.
        output: PathBuf,
        /// Generated package directory that must remain read-only.
        generated: PathBuf,
    },
    /// A filesystem operation failed for a known path.
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Original operating-system error.
        source: io::Error,
    },
    /// A submitted text artifact was not valid UTF-8.
    InvalidUtf8 {
        /// Artifact that could not be decoded.
        path: PathBuf,
    },
    /// A submitted JSON input could not be parsed.
    InvalidJson {
        /// JSON file that could not be parsed.
        path: PathBuf,
        /// Parser diagnostic retained as evidence.
        source: serde_json::Error,
    },
    /// One supported artifact exceeded its configured byte limit.
    ArtifactTooLarge {
        /// Artifact that exceeded the limit.
        path: PathBuf,
        /// Configured inclusive byte limit.
        limit: usize,
    },
    /// The collected package exceeded its configured aggregate byte limit.
    PackageTooLarge {
        /// Configured inclusive aggregate byte limit.
        limit: usize,
    },
    /// The package contained more supported artifacts than configured.
    TooManyArtifacts {
        /// Configured inclusive artifact-count limit.
        limit: usize,
    },
    /// A package identifier was empty after trimming whitespace.
    EmptyPackageId,
}

impl Display for EvaluatorError {
    /// Formats a concise diagnostic suitable for command-line display.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path, expected } => {
                write!(formatter, "{} is not {expected}", path.display())
            }
            Self::UnsafeOutputOverlap { output, generated } => write!(
                formatter,
                "output {} overlaps read-only package {}",
                output.display(),
                generated.display()
            ),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::InvalidUtf8 { path } => write!(formatter, "{} is not UTF-8", path.display()),
            Self::InvalidJson { path, source } => {
                write!(
                    formatter,
                    "{} contains invalid JSON: {source}",
                    path.display()
                )
            }
            Self::ArtifactTooLarge { path, limit } => write!(
                formatter,
                "{} exceeds the {limit}-byte artifact limit",
                path.display()
            ),
            Self::PackageTooLarge { limit } => {
                write!(
                    formatter,
                    "package exceeds the {limit}-byte aggregate limit"
                )
            }
            Self::TooManyArtifacts { limit } => {
                write!(formatter, "package exceeds the {limit}-artifact limit")
            }
            Self::EmptyPackageId => write!(formatter, "package identifier cannot be empty"),
        }
    }
}

impl Error for EvaluatorError {
    /// Exposes wrapped I/O and JSON parser failures to diagnostic callers.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidJson { source, .. } => Some(source),
            _ => None,
        }
    }
}
