use std::path::PathBuf;

use project_initiation_evaluator::{
    Artifact, Corpus, calculate_metrics, extract_entities, score_corpus,
};

/// Creates a stable in-memory artifact for scoring tests.
fn artifact(path: &str, content: &str) -> Artifact {
    Artifact {
        relative_path: PathBuf::from(path),
        content: content.to_owned(),
        content_hash: format!("hash-{path}"),
    }
}

/// Creates an isolated corpus with supplied brief and generated artifact content.
fn corpus(brief: &str, artifacts: Vec<Artifact>) -> Corpus {
    Corpus {
        package_id: "score-test".to_owned(),
        brief: artifact("ORIGINAL_BRIEF", brief),
        artifacts,
        project_model: None,
        metadata: None,
        skipped_files: Vec::new(),
    }
}

/// Proves fixed rubric maxima, bounded scores, total consistency, and complete findings.
#[test]
fn scoring_preserves_fixed_weights_and_complete_negative_findings() {
    let corpus = corpus(
        "REQ-001: The product must work offline.",
        vec![artifact(
            "Decisions.md",
            "# Decisions\nADR-001: Use mandatory cloud persistence.",
        )],
    );
    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);

    let report = score_corpus(&corpus, &extraction, metrics);
    let maxima = report.dimensions.maxima();
    let scores = report.dimensions.scores();

    assert_eq!(maxima, [20, 15, 15, 15, 15, 15, 5]);
    assert!(
        scores
            .iter()
            .zip(maxima)
            .all(|(score, max)| *score >= 0.0 && *score <= f64::from(max))
    );
    assert_eq!(report.drpfs, report.dimensions.total());
    assert!(
        report
            .dimensions
            .all_findings()
            .iter()
            .all(|finding| finding.is_complete())
    );
}

/// Proves syntactically valid but semantically unrelated links are reported as traceability theater.
#[test]
fn scoring_flags_evidence_backed_traceability_theater() {
    let corpus = corpus(
        "REQ-021: The pause menu must support controller navigation.",
        vec![artifact(
            "Decisions.md",
            "# Requirements\nREQ-021: The pause menu supports controller navigation.\n# Decisions\nADR-009: Use PostgreSQL for hosted persistence; supports REQ-021.",
        )],
    );
    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);

    let report = score_corpus(&corpus, &extraction, metrics);

    assert!(
        report
            .dimensions
            .traceability
            .findings
            .iter()
            .any(|finding| finding.failure_mode.as_deref() == Some("TraceabilityTheater"))
    );
}

/// Proves independently evidenced special failure modes remain visible in the report.
#[test]
fn scoring_classifies_multiple_special_failure_modes() {
    let corpus = corpus(
        "REQ-001: The product must work offline.",
        vec![artifact(
            "Foundation.md",
            "# Requirements\nREQ-001: Work offline.\n# Decisions\nADR-001: Market research selects PostgreSQL persistence; supports EVD-001.\n# Evidence\nEVD-001: A color palette preference survey.\n# User Answers\nANS-001: The user rejected cloud storage.",
        )],
    );
    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);

    let report = score_corpus(&corpus, &extraction, metrics);
    let modes = report
        .dimensions
        .all_findings()
        .iter()
        .filter_map(|finding| finding.failure_mode.as_deref())
        .collect::<Vec<_>>();

    assert!(modes.contains(&"EvidenceMisuse"));
    assert!(modes.contains(&"UserOverrideFailure"));
    assert!(modes.contains(&"UnderSpecification"));
}
