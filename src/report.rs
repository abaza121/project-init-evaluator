use serde::Serialize;

use crate::metrics::DeterministicMetrics;

/// Classifies the implementation impact of an evidence-backed negative finding.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    /// Minor navigation, wording, or completeness friction.
    Low,
    /// Material quality gap that should be corrected before implementation.
    Medium,
    /// Consequential contradiction, unsupported choice, or hidden blocker.
    High,
    /// Failure that invalidates evaluator isolation or the submitted package.
    Critical,
}

/// Records one evidence-backed deduction and its recommended package correction.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Finding {
    /// Implementation impact of the observed issue.
    pub severity: Severity,
    /// Rubric criterion used to make the deduction.
    pub criterion: String,
    /// Concise explanation of the observed quality gap.
    pub description: String,
    /// Submitted artifact containing the supporting evidence.
    pub artifact: String,
    /// Related requirement, decision, answer, or finding ID when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_id: Option<String>,
    /// Bounded submitted excerpt supporting the deduction.
    pub supporting_evidence: String,
    /// Concrete change recommended for the generator or package author.
    pub recommended_correction: String,
    /// Named special failure classification from the evaluator brief.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_mode: Option<String>,
}

impl Finding {
    /// Confirms all mandatory negative-finding fields contain usable text.
    pub fn is_complete(&self) -> bool {
        !self.criterion.trim().is_empty()
            && !self.description.trim().is_empty()
            && !self.artifact.trim().is_empty()
            && !self.supporting_evidence.trim().is_empty()
            && !self.recommended_correction.trim().is_empty()
    }
}

/// Stores one bounded dimension score and all evidence-backed deductions.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DimensionScore {
    /// Awarded points rounded to one decimal place.
    pub score: f64,
    /// Immutable rubric maximum for this dimension.
    pub max: u8,
    /// Evidence-backed deductions associated with this dimension.
    pub findings: Vec<Finding>,
}

/// Stores all seven fixed DRPFS dimensions using required JSON field names.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Dimensions {
    /// Preservation of explicit user intent and answers.
    pub brief_fidelity: DimensionScore,
    /// Separation of user facts, inferences, and unresolved unknowns.
    pub assumption_discipline: DimensionScore,
    /// Agreement among generated project artifacts.
    pub cross_document_consistency: DimensionScore,
    /// Relevance, support, linkage, and usability of evidence.
    pub evidence_quality: DimensionScore,
    /// Meaningful requirement-to-decision provenance.
    pub traceability: DimensionScore,
    /// Readiness for another developer to continue implementation.
    pub actionability: DimensionScore,
    /// Required artifact presence and usable navigation.
    pub artifact_quality: DimensionScore,
}

impl Dimensions {
    /// Returns dimension maxima in canonical rubric order.
    pub fn maxima(&self) -> [u8; 7] {
        [
            self.brief_fidelity.max,
            self.assumption_discipline.max,
            self.cross_document_consistency.max,
            self.evidence_quality.max,
            self.traceability.max,
            self.actionability.max,
            self.artifact_quality.max,
        ]
    }

    /// Returns awarded dimension scores in canonical rubric order.
    pub fn scores(&self) -> [f64; 7] {
        [
            self.brief_fidelity.score,
            self.assumption_discipline.score,
            self.cross_document_consistency.score,
            self.evidence_quality.score,
            self.traceability.score,
            self.actionability.score,
            self.artifact_quality.score,
        ]
    }

    /// Returns the one-decimal sum of all fixed dimension scores.
    pub fn total(&self) -> f64 {
        round1(self.scores().iter().sum())
    }

    /// Returns borrowed findings from every dimension in rubric order.
    pub fn all_findings(&self) -> Vec<&Finding> {
        [
            &self.brief_fidelity,
            &self.assumption_discipline,
            &self.cross_document_consistency,
            &self.evidence_quality,
            &self.traceability,
            &self.actionability,
            &self.artifact_quality,
        ]
        .into_iter()
        .flat_map(|dimension| dimension.findings.iter())
        .collect()
    }
}

/// Represents the complete normalized JSON and Markdown report model.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ValidationReport {
    /// Decision-Ready Project Foundation Score out of 100.
    pub drpfs: f64,
    /// Seven immutable scoring dimensions.
    pub dimensions: Dimensions,
    /// Deterministic metrics available from authoritative submitted records.
    pub metrics: DeterministicMetrics,
    /// Critical failure descriptions requiring immediate attention.
    pub critical_failures: Vec<String>,
    /// Evidence-based positive observations.
    pub strengths: Vec<String>,
    /// Deduplicated corrections ranked by finding severity and rubric order.
    pub highest_priority_improvements: Vec<String>,
    /// High-impact decisions lacking valid submitted provenance.
    pub unsupported_decisions: Vec<String>,
    /// Consequential assumptions and questions still unresolved.
    pub unresolved_assumptions_and_questions: Vec<String>,
    /// Scope note for submitted external citations.
    pub external_verification_note: String,
}

/// Rounds a finite score to one decimal place.
pub(crate) fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
