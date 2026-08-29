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

impl Severity {
    /// Returns the uppercase report label for this severity.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
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

/// Serializes one normalized validation report as stable pretty JSON.
pub fn render_validation_json(report: &ValidationReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report).map(|mut json| {
        json.push('\n');
        json
    })
}

/// Renders every required validation-report Markdown section from one normalized model.
pub fn render_validation_markdown(report: &ValidationReport) -> String {
    let mut output = String::new();
    output.push_str("# Validation Report\n\n## Overall Score\n\n");
    output.push_str(&format!("DRPFS: {:.1} / 100\n\n", report.drpfs));
    output.push_str("## Scorecard\n\n| Dimension | Score |\n|---|---:|\n");
    append_score_rows(&mut output, &report.dimensions);
    output.push_str(&format!("| TOTAL | {:.1}/100 |\n\n", report.drpfs));
    output.push_str("## Deterministic Metrics\n\n");
    append_metrics(&mut output, &report.metrics);
    output.push_str("\n## Critical Findings\n\n");
    append_filtered_findings(&mut output, &report.dimensions, true);
    output.push_str("\n## Detailed Findings\n\n");
    append_filtered_findings(&mut output, &report.dimensions, false);
    output.push_str("\n## Unsupported Decisions\n\n");
    append_list(&mut output, &report.unsupported_decisions);
    output.push_str("\n## Unresolved Assumptions and Questions\n\n");
    append_list(&mut output, &report.unresolved_assumptions_and_questions);
    output.push_str("\n## Strengths\n\n");
    append_list(&mut output, &report.strengths);
    output.push_str("\n## Recommended Improvements\n\n");
    append_numbered_list(&mut output, &report.highest_priority_improvements);
    output.push_str("\n## External Verification Scope\n\n");
    output.push_str(&report.external_verification_note);
    output.push('\n');
    output
}

/// Appends all seven scorecard rows in fixed rubric order.
fn append_score_rows(output: &mut String, dimensions: &Dimensions) {
    let rows = [
        ("Brief Fidelity", &dimensions.brief_fidelity),
        ("Assumption Discipline", &dimensions.assumption_discipline),
        (
            "Cross-Document Consistency",
            &dimensions.cross_document_consistency,
        ),
        ("Evidence Quality", &dimensions.evidence_quality),
        (
            "Requirements → Decision Traceability",
            &dimensions.traceability,
        ),
        ("Actionability", &dimensions.actionability),
        ("Artifact Completeness", &dimensions.artifact_quality),
    ];
    for (label, dimension) in rows {
        output.push_str(&format!(
            "| {label} | {:.1}/{} |\n",
            dimension.score, dimension.max
        ));
    }
}

/// Appends every deterministic metric with explicit unavailable values.
fn append_metrics(output: &mut String, metrics: &DeterministicMetrics) {
    let percentages = [
        (
            "Required Artifact Completion",
            metrics.required_artifact_completion,
        ),
        (
            "Acceptance Criteria Coverage",
            metrics.acceptance_criteria_coverage,
        ),
        (
            "Requirement Traceability Coverage",
            metrics.requirement_traceability_coverage,
        ),
        (
            "Unsupported Decision Rate",
            metrics.unsupported_decision_rate,
        ),
        (
            "High-Impact Assumption Labeling Rate",
            metrics.high_impact_assumption_labeling_rate,
        ),
        ("Evidence Linkage Rate", metrics.evidence_linkage_rate),
        (
            "User Answer Adoption Rate",
            metrics.user_answer_adoption_rate,
        ),
        ("Duplicate Question Rate", metrics.duplicate_question_rate),
        ("Answer Reuse Rate", metrics.answer_reuse_rate),
        (
            "Retrieval Utilization Rate",
            metrics.retrieval_utilization_rate,
        ),
        ("Repeated Research Rate", metrics.repeated_research_rate),
        ("Stale Retrieval Rate", metrics.stale_retrieval_rate),
    ];
    for (label, value) in percentages {
        let rendered =
            value.map_or_else(|| "Unavailable".to_owned(), |value| format!("{value:.1}%"));
        output.push_str(&format!("- {label}: {rendered}\n"));
    }
    append_optional_count(
        output,
        "Broken Internal Links",
        metrics.broken_internal_links,
    );
    append_optional_count(
        output,
        "Unresolved High-Severity Findings",
        metrics.unresolved_high_severity_findings,
    );
    append_optional_count(
        output,
        "Cross-Project Leakage",
        metrics.cross_project_leakage,
    );
    append_optional_u64(output, "Context Supplied", metrics.context_supplied);
    append_optional_u64(output, "Retrieval Calls", metrics.retrieval_calls);
    append_optional_number(
        output,
        "Context Compression Ratio",
        metrics.context_compression_ratio,
    );
    append_optional_number(
        output,
        "Execution Time (seconds)",
        metrics.execution_time_seconds,
    );
    append_optional_number(
        output,
        "Human Interaction Time (seconds)",
        metrics.human_interaction_time_seconds,
    );
    append_optional_u64(output, "User Questions", metrics.user_questions);
    append_optional_u64(output, "Model Calls", metrics.model_calls);
    append_optional_u64(output, "Token Usage", metrics.token_usage);
    append_optional_number(output, "Cost", metrics.cost);
    let names = if metrics.conflicting_project_names.is_empty() {
        "None discovered".to_owned()
    } else {
        metrics.conflicting_project_names.join(", ")
    };
    output.push_str(&format!("- Conflicting Project Names: {names}\n"));
}

/// Appends one optional integer metric without inventing a value.
fn append_optional_count(output: &mut String, label: &str, value: Option<usize>) {
    let rendered = value.map_or_else(|| "Unavailable".to_owned(), |value| value.to_string());
    output.push_str(&format!("- {label}: {rendered}\n"));
}

/// Appends one optional 64-bit count without inventing a value.
fn append_optional_u64(output: &mut String, label: &str, value: Option<u64>) {
    let rendered = value.map_or_else(|| "Unavailable".to_owned(), |value| value.to_string());
    output.push_str(&format!("- {label}: {rendered}\n"));
}

/// Appends one optional finite number without inventing a value or unit.
fn append_optional_number(output: &mut String, label: &str, value: Option<f64>) {
    let rendered = value.map_or_else(|| "Unavailable".to_owned(), |value| format!("{value:.1}"));
    output.push_str(&format!("- {label}: {rendered}\n"));
}

/// Appends either high-impact findings or all detailed findings.
fn append_filtered_findings(output: &mut String, dimensions: &Dimensions, critical_only: bool) {
    let findings = dimensions
        .all_findings()
        .into_iter()
        .filter(|finding| {
            !critical_only || matches!(finding.severity, Severity::High | Severity::Critical)
        })
        .collect::<Vec<_>>();
    if findings.is_empty() {
        output.push_str("- None.\n");
        return;
    }
    for finding in findings {
        output.push_str(&format!(
            "### [{}] {}\n\n- Criterion: {}\n- Artifact: {}\n",
            finding.severity.as_str(),
            finding.description,
            finding.criterion,
            finding.artifact
        ));
        if let Some(identifier) = &finding.related_id {
            output.push_str(&format!("- Related ID: {identifier}\n"));
        }
        if let Some(mode) = &finding.failure_mode {
            output.push_str(&format!("- Failure mode: {mode}\n"));
        }
        output.push_str(&format!(
            "- Evidence: {}\n- Recommended correction: {}\n\n",
            finding.supporting_evidence, finding.recommended_correction
        ));
    }
}

/// Appends a Markdown bullet list or an explicit empty state.
fn append_list(output: &mut String, values: &[String]) {
    if values.is_empty() {
        output.push_str("- None.\n");
    } else {
        for value in values {
            output.push_str(&format!("- {value}\n"));
        }
    }
}

/// Appends a ranked Markdown list or an explicit empty state.
fn append_numbered_list(output: &mut String, values: &[String]) {
    if values.is_empty() {
        output.push_str("1. None.\n");
    } else {
        for (index, value) in values.iter().enumerate() {
            output.push_str(&format!("{}. {value}\n", index + 1));
        }
    }
}

/// Rounds a finite score to one decimal place.
pub(crate) fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
