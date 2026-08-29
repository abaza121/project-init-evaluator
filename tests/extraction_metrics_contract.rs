use std::path::PathBuf;

use project_initiation_evaluator::{
    Artifact, Corpus, EntityKind, calculate_metrics, extract_entities,
};
use serde_json::json;

/// Creates a stable artifact record for extraction and metric tests.
fn artifact(path: &str, content: &str) -> Artifact {
    Artifact {
        relative_path: PathBuf::from(path),
        content: content.to_owned(),
        content_hash: format!("hash-{path}"),
    }
}

/// Creates a corpus with supplied generated artifacts and no optional inputs.
fn corpus(artifacts: Vec<Artifact>) -> Corpus {
    Corpus {
        package_id: "test-package".to_owned(),
        brief: artifact(
            "ORIGINAL_BRIEF",
            "REQ-001: The product must work offline.\nREQ-002: The product must be documented.",
        ),
        artifacts,
        project_model: None,
        metadata: None,
        skipped_files: Vec::new(),
    }
}

/// Proves extracted entities retain source evidence and explicit traces remain meaningful.
#[test]
fn extraction_retains_evidence_and_explicit_trace_endpoints() {
    let corpus = corpus(vec![artifact(
        "Requirements.md",
        "# Requirements\nREQ-001: Work offline.\n\n# Decisions\nADR-001: Use local files; supports REQ-001.\n\n# Assumptions\nASM-001: Desktop is assumed.\n\n# Evidence\nEVD-001: https://example.com/offline\n\n# Open Questions\nQ-001: Which operating systems?",
    )]);

    let extraction = extract_entities(&corpus);
    let decision = extraction
        .entities
        .iter()
        .find(|entity| entity.id.as_deref() == Some("ADR-001"))
        .expect("the identified decision should be extracted");

    assert_eq!(decision.kind, EntityKind::Decision);
    assert_eq!(decision.evidence.artifact, "Requirements.md");
    assert_eq!(decision.evidence.line, 5);
    assert!(
        extraction
            .traces
            .iter()
            .any(|trace| trace.from_id == "ADR-001" && trace.to_id == "REQ-001")
    );
}

/// Proves deterministic metrics use explicit submitted records and preserve unavailable values.
#[test]
fn metrics_calculate_authoritative_rates_without_guessing() {
    let mut corpus = corpus(vec![
        artifact(
            "README.md",
            "[Requirements](Requirements.md)\n[Missing](Missing.md)",
        ),
        artifact(
            "Requirements.md",
            "# Requirements\nREQ-001: Work offline.\nREQ-002: Provide docs.\n# Acceptance Criteria\nAC-001 for REQ-001: Works without a network.",
        ),
        artifact(
            "Decisions.md",
            "# Decisions\nADR-001: Research selects local files; supports REQ-001 and EVD-001.\nADR-002: Market research selects PostgreSQL persistence.\n# Evidence\nEVD-001: Local storage study.\n# User Answers\nANS-001: Offline is consequential. ADR-001 supports ANS-001.\nANS-002: Documentation is consequential.",
        ),
    ]);
    corpus.metadata = Some(json!({
        "required_artifacts": ["README.md", "Architecture.md"],
        "duplicate_question_rate": 25.0,
        "answer_reuse_rate": 75.0,
        "repeated_research_rate": 10.0,
        "stale_retrieval_rate": 5.0,
        "context_supplied": 1200,
        "retrieval_calls": 8,
        "cross_project_leakage": 0,
        "execution_time_seconds": 12.5,
        "model_calls": 4,
        "findings": [
            {"id": "F-1", "severity": "HIGH", "resolved": false},
            {"id": "F-2", "severity": "LOW", "resolved": false}
        ]
    }));

    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);

    assert_eq!(metrics.required_artifact_completion, Some(50.0));
    assert_eq!(metrics.acceptance_criteria_coverage, Some(50.0));
    assert_eq!(metrics.requirement_traceability_coverage, Some(50.0));
    assert_eq!(metrics.unsupported_decision_rate, Some(50.0));
    assert_eq!(metrics.evidence_linkage_rate, Some(50.0));
    assert_eq!(metrics.broken_internal_links, Some(1));
    assert_eq!(metrics.unresolved_high_severity_findings, Some(1));
    assert_eq!(metrics.unresolved_high_severity_finding_ids, ["F-1"]);
    assert_eq!(metrics.user_answer_adoption_rate, Some(50.0));
    assert_eq!(metrics.duplicate_question_rate, Some(25.0));
    assert_eq!(metrics.answer_reuse_rate, Some(75.0));
    assert_eq!(metrics.repeated_research_rate, Some(10.0));
    assert_eq!(metrics.stale_retrieval_rate, Some(5.0));
    assert_eq!(metrics.context_supplied, Some(1200));
    assert_eq!(metrics.retrieval_calls, Some(8));
    assert_eq!(metrics.cross_project_leakage, Some(0));
    assert_eq!(metrics.execution_time_seconds, Some(12.5));
    assert_eq!(metrics.model_calls, Some(4));
}

/// Proves metrics with no authoritative denominator remain unavailable rather than becoming zero.
#[test]
fn metrics_leave_unavailable_denominators_null() {
    let mut corpus = corpus(vec![artifact("README.md", "# Project")]);
    corpus.brief = artifact(
        "ORIGINAL_BRIEF",
        "A project idea without settled requirements.",
    );
    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);

    assert_eq!(metrics.required_artifact_completion, None);
    assert_eq!(metrics.acceptance_criteria_coverage, None);
    assert_eq!(metrics.unsupported_decision_rate, None);
    assert_eq!(metrics.evidence_linkage_rate, None);
    assert_eq!(metrics.user_answer_adoption_rate, None);
    assert_eq!(metrics.broken_internal_links, Some(0));
}

/// Proves syntactically valid co-located IDs do not become traceability theater.
#[test]
fn extraction_does_not_invent_trace_from_id_colocation() {
    let corpus = corpus(vec![artifact(
        "Decisions.md",
        "# Decisions\nADR-009: Use PostgreSQL. REQ-021: Controller navigation.",
    )]);

    let extraction = extract_entities(&corpus);

    assert!(
        !extraction
            .traces
            .iter()
            .any(|trace| trace.from_id == "ADR-009" && trace.to_id == "REQ-021")
    );
}

/// Proves structured project-model records join the same isolated extraction graph.
#[test]
fn extraction_imports_identified_project_model_records() {
    let mut corpus = corpus(Vec::new());
    corpus.project_model = Some(json!({
        "requirements": [{"id": "REQ-010", "text": "Support keyboard navigation."}],
        "decisions": [{"id": "ADR-010", "text": "Use semantic controls; supports REQ-010."}]
    }));

    let extraction = extract_entities(&corpus);

    assert!(
        extraction
            .entities
            .iter()
            .any(|entity| entity.id.as_deref() == Some("REQ-010"))
    );
    assert!(
        extraction
            .traces
            .iter()
            .any(|trace| trace.from_id == "ADR-010" && trace.to_id == "REQ-010")
    );
}
