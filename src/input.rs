use std::path::PathBuf;

/// Identifies all submitted and generated paths for one isolated package evaluation.
#[derive(Clone, Debug)]
pub struct PackageInput {
    /// Original user brief used as the fidelity authority.
    pub brief: PathBuf,
    /// Generated project-foundation directory inspected read-only.
    pub generated_dir: PathBuf,
    /// Optional structured project model or database export.
    pub project_model: Option<PathBuf>,
    /// Optional validation and execution metadata.
    pub metadata: Option<PathBuf>,
    /// Stable package identity retained in evaluator-owned records.
    pub package_id: String,
    /// Directory reserved for reports after successful evaluation.
    pub output_dir: PathBuf,
}
