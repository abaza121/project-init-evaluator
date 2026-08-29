use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::Corpus;
use crate::extract::{EntityKind, ExtractedEntity, Extraction, TraceLink};
use crate::metrics::DeterministicMetrics;
use crate::report::{DimensionScore, Dimensions, Finding, Severity, ValidationReport, round1};

/// Scores one isolated corpus against the fixed DRPFS rubric.
pub fn score_corpus(
    corpus: &Corpus,
    extraction: &Extraction,
    metrics: DeterministicMetrics,
) -> ValidationReport {
    let dimensions = Dimensions {
        brief_fidelity: score_brief_fidelity(corpus, extraction, &metrics),
        assumption_discipline: score_assumption_discipline(extraction, &metrics),
        cross_document_consistency: score_consistency(corpus, &metrics),
        evidence_quality: score_evidence_quality(extraction, &metrics),
        traceability: score_traceability(extraction, &metrics),
        actionability: score_actionability(corpus, extraction, &metrics),
        artifact_quality: score_artifact_quality(corpus, &metrics),
    };
    let drpfs = dimensions.total();
    let critical_failures = dimensions
        .all_findings()
        .iter()
        .filter(|finding| finding.severity == Severity::Critical)
        .map(|finding| finding.description.clone())
        .collect();
    let strengths = collect_strengths(&dimensions, &metrics);
    let highest_priority_improvements = collect_improvements(&dimensions);
    let unsupported_decisions = unsupported_decisions(extraction, &metrics);
    let unresolved_assumptions_and_questions = unresolved_items(extraction);

    ValidationReport {
        drpfs,
        dimensions,
        metrics,
        critical_failures,
        strengths,
        highest_priority_improvements,
        unsupported_decisions,
        unresolved_assumptions_and_questions,
        external_verification_note: "External source correctness was not independently verified; citation identity and submitted decision linkage were evaluated locally.".to_owned(),
    }
}

/// Scores explicit requirement coverage, contradictions, and user-answer adoption.
fn score_brief_fidelity(
    corpus: &Corpus,
    extraction: &Extraction,
    metrics: &DeterministicMetrics,
) -> DimensionScore {
    let brief_requirements = entities_from(extraction, EntityKind::Requirement, "ORIGINAL_BRIEF");
    let generated_requirements =
        entities_not_from(extraction, EntityKind::Requirement, "ORIGINAL_BRIEF");
    let mut findings = Vec::new();
    let coverage =
        requirement_coverage(&brief_requirements, &generated_requirements, &mut findings);
    let contradictions = contradiction_findings(corpus);
    let contradiction_factor = match contradictions.len() {
        0 => 1.0,
        1 => 2.0 / 3.0,
        2 => 1.0 / 3.0,
        _ => 0.0,
    };
    findings.extend(contradictions);
    let has_answers = extraction
        .entities
        .iter()
        .any(|entity| entity.kind == EntityKind::UserAnswer);
    let score = if has_answers {
        let answer = metrics.user_answer_adoption_rate.unwrap_or(0.0) / 100.0;
        if answer < 1.0 {
            add_unadopted_answer_findings(extraction, &mut findings);
        }
        coverage * 8.0 + contradiction_factor * 6.0 + answer * 6.0
    } else {
        coverage * (8.0 + 6.0 * 8.0 / 14.0) + contradiction_factor * (6.0 + 6.0 * 6.0 / 14.0)
    };
    dimension(score, 20, findings)
}

/// Calculates partial requirement coverage and records every missing explicit requirement.
fn requirement_coverage(
    brief: &[&ExtractedEntity],
    generated: &[&ExtractedEntity],
    findings: &mut Vec<Finding>,
) -> f64 {
    if brief.is_empty() {
        findings.push(finding(
            Severity::High,
            "Brief Fidelity.A",
            "No materially explicit requirement could be extracted from the original brief.",
            "ORIGINAL_BRIEF",
            None,
            "The brief contains no identified or normative requirement statement.",
            "Supply explicit requirements so generated coverage can be evaluated.",
            Some("UnderSpecification"),
        ));
        return 0.0;
    }
    let mut covered = 0.0;
    for requirement in brief {
        let best = generated
            .iter()
            .map(|candidate| token_overlap(&requirement.text, &candidate.text))
            .fold(0.0, f64::max);
        if best >= 0.5 {
            covered += 1.0;
        } else if best >= 0.25 {
            covered += 0.5;
            findings.push(entity_finding(
                Severity::Medium,
                "Brief Fidelity.A",
                "An explicit brief requirement is only partially represented by generated requirements.",
                requirement,
                "Preserve the complete requirement meaning in an identified generated requirement.",
                None,
            ));
        } else {
            findings.push(entity_finding(
                Severity::High,
                "Brief Fidelity.A",
                "An explicit brief requirement has no meaningful generated requirement candidate.",
                requirement,
                "Add a generated requirement that preserves this explicit user need.",
                None,
            ));
        }
    }
    covered / brief.len() as f64
}

/// Detects narrow high-confidence contradictions between explicit brief and generated claims.
fn contradiction_findings(corpus: &Corpus) -> Vec<Finding> {
    let brief = corpus.brief.content.to_ascii_lowercase();
    let generated = corpus
        .artifacts
        .iter()
        .map(|artifact| artifact.content.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let conflicts = [
        (
            "offline",
            "mandatory cloud",
            "offline operation",
            "mandatory cloud dependency",
        ),
        (
            "single-player",
            "multiplayer required",
            "single-player scope",
            "required multiplayer scope",
        ),
        (
            "on-device only",
            "cloud-only",
            "on-device-only data",
            "cloud-only data",
        ),
        (
            "no authentication",
            "authentication required",
            "no authentication",
            "required authentication",
        ),
    ];
    conflicts
        .iter()
        .filter(|(brief_term, generated_term, _, _)| {
            brief.contains(brief_term) && generated.contains(generated_term)
        })
        .map(|(_, _, brief_label, generated_label)| {
            finding(
                Severity::High,
                "Brief Fidelity.B",
                &format!("Generated artifacts contradict the brief's {brief_label} with {generated_label}."),
                "generated package",
                None,
                &format!("Brief: {brief_label}; generated package: {generated_label}."),
                "Remove the conflicting generated choice or surface the conflict for user resolution.",
                Some("DocumentDrift"),
            )
        })
        .collect()
}

/// Adds fidelity findings for identified answers not participating in any explicit trace.
fn add_unadopted_answer_findings(extraction: &Extraction, findings: &mut Vec<Finding>) {
    for answer in extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::UserAnswer)
        .filter(|entity| {
            entity
                .id
                .as_ref()
                .is_some_and(|identifier| !trace_touches(extraction, identifier))
        })
    {
        findings.push(entity_finding(
            Severity::High,
            "Brief Fidelity.C",
            "A consequential user answer is not adopted by any requirement or decision trace.",
            answer,
            "Link the answer to the final requirement or decision that reflects it.",
            Some("UserOverrideFailure"),
        ));
    }
}

/// Scores explicit assumption labels, visible unknowns, and decision provenance.
fn score_assumption_discipline(
    extraction: &Extraction,
    metrics: &DeterministicMetrics,
) -> DimensionScore {
    let mut findings = Vec::new();
    let label_rate = metrics
        .high_impact_assumption_labeling_rate
        .unwrap_or(100.0)
        / 100.0;
    let provenance_rate = 1.0 - metrics.unsupported_decision_rate.unwrap_or(0.0) / 100.0;
    let unsupported = unsupported_entities(extraction);
    for entity in &unsupported {
        if contains_high_impact_concept(&entity.text) {
            findings.push(entity_finding(
                Severity::High,
                "Assumption Discipline.A",
                "A consequential inferred choice is presented as a settled decision without labeled provenance.",
                entity,
                "Label the choice as an assumption or link it to user input, a constraint, or evidence.",
                Some("AssumptionPromotion"),
            ));
        }
    }
    let questions = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::OpenQuestion)
        .count();
    let unknown_score = if unsupported.is_empty() || questions > 0 {
        4.0
    } else {
        findings.push(entity_finding(
            Severity::High,
            "Assumption Discipline.B",
            "Consequential unsupported decisions exist without corresponding visible open questions.",
            unsupported[0],
            "Keep unresolved consequential choices explicit until the user or evidence resolves them.",
            Some("FalseCompleteness"),
        ));
        1.0
    };
    dimension(
        label_rate * 7.0 + unknown_score + provenance_rate * 4.0,
        15,
        findings,
    )
}

/// Scores explicit project-name and high-confidence concept consistency.
fn score_consistency(corpus: &Corpus, metrics: &DeterministicMetrics) -> DimensionScore {
    let mut findings = Vec::new();
    if metrics.conflicting_project_names.len() > 1 {
        findings.push(finding(
            Severity::High,
            "Cross-Document Consistency",
            "Generated artifacts contain multiple explicit project-name variants.",
            "generated package",
            None,
            &metrics.conflicting_project_names.join(", "),
            "Use one canonical project name across all artifacts and structured records.",
            Some("DocumentDrift"),
        ));
    }
    findings.extend(cross_document_concept_conflicts(corpus));
    let score = match findings.len() {
        0 => 15.0,
        1 => 8.0,
        2 => 4.0,
        _ => 0.0,
    };
    dimension(score, 15, findings)
}

/// Detects incompatible scoped concept claims occurring in different artifacts.
fn cross_document_concept_conflicts(corpus: &Corpus) -> Vec<Finding> {
    let pairs = [
        ("single-player only", "multiplayer required", "player mode"),
        ("local-only storage", "cloud-only storage", "persistence"),
        (
            "no authentication",
            "authentication required",
            "authentication",
        ),
    ];
    let mut findings = Vec::new();
    for (left, right, concept) in pairs {
        let left_artifact = artifact_containing(corpus, left);
        let right_artifact = artifact_containing(corpus, right);
        if let (Some(left_file), Some(right_file)) = (left_artifact, right_artifact)
            && left_file != right_file
        {
            findings.push(finding(
                Severity::High,
                "Cross-Document Consistency",
                &format!("Artifacts make incompatible {concept} claims."),
                &format!("{left_file}; {right_file}"),
                None,
                &format!("{left_file}: {left}; {right_file}: {right}."),
                "Resolve the conflict and propagate one confirmed choice across the package.",
                Some("DocumentDrift"),
            ));
        }
    }
    findings
}

/// Finds the first artifact containing one case-insensitive submitted phrase.
fn artifact_containing<'a>(corpus: &'a Corpus, phrase: &str) -> Option<&'a str> {
    corpus.artifacts.iter().find_map(|artifact| {
        artifact
            .content
            .to_ascii_lowercase()
            .contains(phrase)
            .then(|| artifact.relative_path.to_str())
            .flatten()
    })
}

/// Scores research targeting, claim support, explicit linkage, and citation identity.
fn score_evidence_quality(
    extraction: &Extraction,
    metrics: &DeterministicMetrics,
) -> DimensionScore {
    let research = research_decisions(extraction);
    if research.is_empty() {
        return dimension(15.0, 15, Vec::new());
    }
    let evidence = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::EvidenceClaim)
        .collect::<Vec<_>>();
    let linkage = metrics.evidence_linkage_rate.unwrap_or(0.0) / 100.0;
    let mut findings = evidence_misuse_findings(extraction);
    let misuse_penalty = findings.len() as f64 * 3.0;
    for decision in research.iter().filter(|decision| {
        decision
            .id
            .as_ref()
            .is_none_or(|identifier| !trace_links_evidence(extraction, identifier))
    }) {
        findings.push(entity_finding(
            Severity::High,
            "Evidence Quality.C",
            "A research-dependent decision has no explicit evidence relationship.",
            decision,
            "Link the decision to identifiable evidence that actually informed it.",
            Some("ResearchAfterDecision"),
        ));
    }
    for claim in evidence.iter().filter(|claim| {
        claim
            .id
            .as_ref()
            .is_none_or(|identifier| !trace_touches(extraction, identifier))
    }) {
        findings.push(entity_finding(
            Severity::Medium,
            "Evidence Quality.A",
            "An evidence claim is not connected to any consequential decision.",
            claim,
            "Connect useful evidence to a decision or remove decorative research.",
            Some("DecorativeResearch"),
        ));
    }
    let targeting = if evidence.is_empty() { 0.0 } else { 4.0 };
    let citation_quality = if evidence.iter().any(|claim| {
        claim.text.contains("http://") || claim.text.contains("https://") || claim.id.is_some()
    }) {
        2.0
    } else {
        0.0
    };
    dimension(
        targeting + linkage * 5.0 + linkage * 4.0 + citation_quality - misuse_penalty,
        15,
        findings,
    )
}

/// Detects decision-to-evidence traces whose endpoint statements share no meaningful subject.
fn evidence_misuse_findings(extraction: &Extraction) -> Vec<Finding> {
    let by_id = extraction
        .entities
        .iter()
        .filter_map(|entity| entity.id.as_ref().map(|id| (id, entity)))
        .collect::<BTreeMap<_, _>>();
    extraction
        .traces
        .iter()
        .filter_map(|trace| {
            let from = by_id.get(&trace.from_id)?;
            let to = by_id.get(&trace.to_id)?;
            let is_decision_evidence = matches!(
                (from.kind, to.kind),
                (EntityKind::Decision, EntityKind::EvidenceClaim)
                    | (EntityKind::EvidenceClaim, EntityKind::Decision)
            );
            (is_decision_evidence && token_overlap(&from.text, &to.text) == 0.0).then(|| {
                finding(
                    Severity::High,
                    "Evidence Quality.B",
                    "Cited evidence has no meaningful subject overlap with the decision it claims to support.",
                    &trace.evidence.artifact,
                    Some(&trace.from_id),
                    &trace.evidence.excerpt,
                    "Replace the evidence relationship with a source that actually supports the decision, or remove the claim.",
                    Some("EvidenceMisuse"),
                )
            })
        })
        .collect()
}

/// Scores requirement trace coverage and decision provenance, then audits link meaning.
fn score_traceability(extraction: &Extraction, metrics: &DeterministicMetrics) -> DimensionScore {
    let coverage = metrics.requirement_traceability_coverage.unwrap_or(0.0);
    let provenance = 1.0 - metrics.unsupported_decision_rate.unwrap_or(0.0) / 100.0;
    let mut findings = trace_theater_findings(extraction);
    let trace_points = traceability_points(coverage);
    if coverage < 100.0 {
        for requirement in untraced_requirements(extraction) {
            findings.push(entity_finding(
                Severity::Medium,
                "Requirements → Decision Traceability",
                "An identified important requirement has no explicit decision trace.",
                requirement,
                "Link the requirement to at least one relevant decision or documented constraint.",
                None,
            ));
        }
    }
    let theater_penalty = findings
        .iter()
        .filter(|finding| finding.failure_mode.as_deref() == Some("TraceabilityTheater"))
        .count() as f64
        * 2.0;
    dimension(
        trace_points + provenance * 6.0 - theater_penalty,
        15,
        findings,
    )
}

/// Maps traceability coverage percentages to the rubric's fixed nine-point scale.
fn traceability_points(coverage: f64) -> f64 {
    match coverage {
        value if value >= 100.0 => 9.0,
        value if value >= 90.0 => 8.0,
        value if value >= 80.0 => 7.0,
        value if value >= 70.0 => 6.0,
        value if value >= 60.0 => 5.0,
        value if value >= 50.0 => 4.0,
        value if value >= 30.0 => 2.0,
        value if value > 0.0 => 1.0,
        _ => 0.0,
    }
}

/// Detects explicit trace endpoints whose submitted statements share no meaningful terms.
fn trace_theater_findings(extraction: &Extraction) -> Vec<Finding> {
    let by_id = extraction
        .entities
        .iter()
        .filter_map(|entity| entity.id.as_ref().map(|id| (id, entity)))
        .collect::<BTreeMap<_, _>>();
    extraction
        .traces
        .iter()
        .filter_map(|trace| implausible_trace_finding(trace, &by_id))
        .collect()
}

/// Creates one traceability-theater finding when endpoint statements have no shared terms.
fn implausible_trace_finding(
    trace: &TraceLink,
    by_id: &BTreeMap<&String, &ExtractedEntity>,
) -> Option<Finding> {
    let from = by_id.get(&trace.from_id)?;
    let to = by_id.get(&trace.to_id)?;
    if token_overlap(&from.text, &to.text) > 0.0 {
        return None;
    }
    Some(finding(
        Severity::High,
        "Requirements → Decision Traceability",
        "An explicit trace links statements with no meaningful subject overlap.",
        &trace.evidence.artifact,
        Some(&trace.from_id),
        &trace.evidence.excerpt,
        "Remove the unrelated link or replace it with a relationship whose rationale is substantively meaningful.",
        Some("TraceabilityTheater"),
    ))
}

/// Scores requirement criteria, architecture concreteness, and blocker visibility.
fn score_actionability(
    corpus: &Corpus,
    extraction: &Extraction,
    metrics: &DeterministicMetrics,
) -> DimensionScore {
    let mut findings = Vec::new();
    let requirement_score = metrics.acceptance_criteria_coverage.unwrap_or(0.0) / 100.0 * 5.0;
    if requirement_score < 5.0 {
        findings.push(finding(
            Severity::High,
            "Actionability.A",
            "Not all identified requirements have explicit acceptance criteria.",
            "generated package",
            None,
            &format!(
                "Acceptance criteria coverage: {}.",
                format_optional_percentage(metrics.acceptance_criteria_coverage)
            ),
            "Add testable acceptance criteria for every implementable important requirement.",
            Some("UnderSpecification"),
        ));
    }
    let architecture_score = architecture_score(corpus, extraction);
    if architecture_score < 5.0 {
        findings.push(finding(
            Severity::Medium,
            "Actionability.B",
            "Architecture lacks enough concrete responsibility or data-flow guidance to begin implementation confidently.",
            "generated package",
            None,
            "No complete combination of decisions, modules/components, responsibilities, and data flow was found.",
            "Describe the smallest concrete modules, boundaries, data flow, and unresolved implementation choices.",
            Some("UnderSpecification"),
        ));
    }
    add_over_engineering_finding(corpus, extraction, &mut findings);
    let unsupported = unsupported_entities(extraction);
    let questions = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::OpenQuestion)
        .count();
    let blocker_score = if unsupported.is_empty() {
        5.0
    } else if questions > 0 {
        3.0
    } else {
        findings.push(entity_finding(
            Severity::High,
            "Actionability.C",
            "Important unsupported decisions remain but the package does not expose them as blockers.",
            unsupported[0],
            "Move unresolved consequential choices into a visible questions or blockers section.",
            Some("FalseCompleteness"),
        ));
        1.0
    };
    dimension(
        requirement_score + architecture_score + blocker_score,
        15,
        findings,
    )
}

/// Scores architecture concreteness from submitted decisions and implementation structure terms.
fn architecture_score(corpus: &Corpus, extraction: &Extraction) -> f64 {
    let decisions = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Decision)
        .count();
    let generated = corpus
        .artifacts
        .iter()
        .map(|artifact| artifact.content.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let concrete_terms = [
        "module",
        "component",
        "responsibility",
        "data flow",
        "integration",
    ]
    .iter()
    .filter(|term| generated.contains(*term))
    .count();
    if decisions > 0 && concrete_terms >= 2 {
        5.0
    } else if decisions > 0 {
        3.0
    } else if generated.contains("architecture") {
        1.0
    } else {
        0.0
    }
}

/// Adds an over-engineering finding only for concentrated unsupported complexity evidence.
fn add_over_engineering_finding(
    corpus: &Corpus,
    extraction: &Extraction,
    findings: &mut Vec<Finding>,
) {
    let generated = corpus
        .artifacts
        .iter()
        .map(|artifact| artifact.content.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let complexity = [
        "microservice",
        "kubernetes",
        "event bus",
        "service mesh",
        "distributed",
    ]
    .iter()
    .filter(|term| generated.contains(*term))
    .count();
    let brief_requirements =
        entities_from(extraction, EntityKind::Requirement, "ORIGINAL_BRIEF").len();
    if complexity >= 3 && brief_requirements <= 2 {
        findings.push(finding(
            Severity::High,
            "Actionability.B",
            "Architecture introduces several major distributed-system mechanisms without corresponding brief scope.",
            "generated package",
            None,
            "At least three of microservices, Kubernetes, event bus, service mesh, or distributed architecture are prescribed for a minimal brief.",
            "Remove unsupported complexity or link each mechanism to a confirmed requirement and constraint.",
            Some("OverEngineering"),
        ));
    }
}

/// Scores required artifact completion and internal navigation integrity.
fn score_artifact_quality(corpus: &Corpus, metrics: &DeterministicMetrics) -> DimensionScore {
    let broken = metrics.broken_internal_links.unwrap_or(0);
    let mut findings = Vec::new();
    if broken > 0 {
        findings.push(finding(
            Severity::Low,
            "Artifact Completeness & Navigation",
            "One or more submitted internal artifact links are broken.",
            "generated package",
            None,
            &format!("Broken internal links: {broken}."),
            "Repair or remove every broken internal artifact link.",
            None,
        ));
    }
    let score = if let Some(completion) = metrics.required_artifact_completion {
        completion / 100.0 * 5.0 - broken.min(2) as f64 * 0.5
    } else {
        let has_readme = corpus.artifacts.iter().any(|artifact| {
            artifact
                .relative_path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("README.md"))
        });
        if has_readme && broken == 0 {
            5.0
        } else if has_readme {
            4.0
        } else if broken == 0 {
            3.0
        } else {
            2.0
        }
    };
    dimension(score, 5, findings)
}

/// Constructs one bounded dimension and rounds its awarded score.
fn dimension(score: f64, max: u8, findings: Vec<Finding>) -> DimensionScore {
    DimensionScore {
        score: round1(score.clamp(0.0, f64::from(max))),
        max,
        findings,
    }
}

/// Constructs a complete finding from direct submitted evidence.
#[allow(clippy::too_many_arguments)]
fn finding(
    severity: Severity,
    criterion: &str,
    description: &str,
    artifact: &str,
    related_id: Option<&str>,
    evidence: &str,
    correction: &str,
    failure_mode: Option<&str>,
) -> Finding {
    Finding {
        severity,
        criterion: criterion.to_owned(),
        description: description.to_owned(),
        artifact: artifact.to_owned(),
        related_id: related_id.map(str::to_owned),
        supporting_evidence: evidence.to_owned(),
        recommended_correction: correction.to_owned(),
        failure_mode: failure_mode.map(str::to_owned),
    }
}

/// Constructs a finding directly from one extracted entity's source evidence.
fn entity_finding(
    severity: Severity,
    criterion: &str,
    description: &str,
    entity: &ExtractedEntity,
    correction: &str,
    failure_mode: Option<&str>,
) -> Finding {
    finding(
        severity,
        criterion,
        description,
        &entity.evidence.artifact,
        entity.id.as_deref(),
        &entity.evidence.excerpt,
        correction,
        failure_mode,
    )
}

/// Returns entities of one kind originating from one exact artifact.
fn entities_from<'a>(
    extraction: &'a Extraction,
    kind: EntityKind,
    artifact: &str,
) -> Vec<&'a ExtractedEntity> {
    extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == kind && entity.evidence.artifact == artifact)
        .collect()
}

/// Returns entities of one kind excluding one exact artifact.
fn entities_not_from<'a>(
    extraction: &'a Extraction,
    kind: EntityKind,
    artifact: &str,
) -> Vec<&'a ExtractedEntity> {
    extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == kind && entity.evidence.artifact != artifact)
        .collect()
}

/// Calculates symmetric meaningful-token overlap between two statements.
fn token_overlap(left: &str, right: &str) -> f64 {
    let left = meaningful_tokens(left);
    let right = meaningful_tokens(right);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f64;
    intersection / left.len().min(right.len()) as f64
}

/// Normalizes meaningful lowercase terms while removing IDs and common glue words.
fn meaningful_tokens(text: &str) -> BTreeSet<String> {
    let stop = [
        "the", "a", "an", "and", "or", "to", "for", "of", "in", "on", "with", "use", "using",
        "supports", "support", "required", "must", "should", "is", "be", "as", "from",
    ];
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .map(str::to_ascii_lowercase)
        .filter(|token| token.len() > 2)
        .filter(|token| {
            !token.contains('-') || !token.chars().any(|character| character.is_ascii_digit())
        })
        .filter(|token| !stop.contains(&token.as_str()))
        .collect()
}

/// Returns decisions whose submitted text explicitly claims research dependence.
fn research_decisions(extraction: &Extraction) -> Vec<&ExtractedEntity> {
    extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Decision)
        .filter(|entity| {
            let lower = entity.text.to_ascii_lowercase();
            [
                "research",
                "evidence",
                "study",
                "benchmark",
                "market data",
                "regulation",
            ]
            .iter()
            .any(|term| lower.contains(term))
        })
        .collect()
}

/// Checks whether a decision has an explicit trace to an evidence identifier.
fn trace_links_evidence(extraction: &Extraction, identifier: &str) -> bool {
    extraction.traces.iter().any(|trace| {
        (trace.from_id == identifier && is_evidence_id(&trace.to_id))
            || (trace.to_id == identifier && is_evidence_id(&trace.from_id))
    })
}

/// Recognizes explicit evidence identifier prefixes.
fn is_evidence_id(identifier: &str) -> bool {
    identifier.starts_with("EVD-") || identifier.starts_with("SRC-")
}

/// Returns identified requirements that do not participate in a validated trace.
fn untraced_requirements(extraction: &Extraction) -> Vec<&ExtractedEntity> {
    extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Requirement)
        .filter(|entity| {
            entity
                .id
                .as_ref()
                .is_some_and(|identifier| !trace_touches(extraction, identifier))
        })
        .collect()
}

/// Checks whether an identifier participates in a validated explicit trace.
fn trace_touches(extraction: &Extraction, identifier: &str) -> bool {
    extraction
        .traces
        .iter()
        .any(|trace| trace.from_id == identifier || trace.to_id == identifier)
}

/// Returns one representative entity for every identified unsupported decision.
fn unsupported_entities(extraction: &Extraction) -> Vec<&ExtractedEntity> {
    let mut representatives = BTreeMap::<&str, &ExtractedEntity>::new();
    for entity in extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Decision)
    {
        if let Some(identifier) = entity.id.as_deref()
            && !decision_has_provenance(entity, extraction, identifier)
        {
            representatives.entry(identifier).or_insert(entity);
        }
    }
    representatives.into_values().collect()
}

/// Checks explicit IDs and validated traces for accepted decision provenance.
fn decision_has_provenance(
    entity: &ExtractedEntity,
    extraction: &Extraction,
    identifier: &str,
) -> bool {
    entity.related_ids.iter().any(|related| {
        ["REQ-", "ANS-", "ASM-", "CON-", "CST-", "EVD-", "SRC-"]
            .iter()
            .any(|prefix| related.starts_with(prefix))
    }) || extraction
        .traces
        .iter()
        .any(|trace| trace.from_id == identifier || trace.to_id == identifier)
}

/// Detects high-impact project choices used in assumption-promotion checks.
fn contains_high_impact_concept(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "platform",
        "framework",
        "multiplayer",
        "authentication",
        "database",
        "postgres",
        "persistence",
        "deployment",
        "monetization",
        "regulatory",
        "cloud",
    ]
    .iter()
    .any(|concept| lower.contains(concept))
}

/// Extracts unsupported decision descriptions for the required report section.
fn unsupported_decisions(extraction: &Extraction, metrics: &DeterministicMetrics) -> Vec<String> {
    if metrics.unsupported_decision_rate.unwrap_or(0.0) == 0.0 {
        return Vec::new();
    }
    unsupported_entities(extraction)
        .into_iter()
        .map(|entity| entity.text.clone())
        .collect()
}

/// Extracts explicit assumptions and open questions for the required report section.
fn unresolved_items(extraction: &Extraction) -> Vec<String> {
    extraction
        .entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                EntityKind::Assumption | EntityKind::OpenQuestion
            )
        })
        .map(|entity| entity.text.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Collects evidence-based strengths from fully satisfied dimension outcomes.
fn collect_strengths(dimensions: &Dimensions, metrics: &DeterministicMetrics) -> Vec<String> {
    let mut strengths = Vec::new();
    if dimensions.brief_fidelity.score == 20.0 {
        strengths.push("All extracted explicit brief requirements are represented without detected contradiction.".to_owned());
    }
    if metrics.requirement_traceability_coverage == Some(100.0) {
        strengths.push(
            "Every identified requirement participates in an explicit validated trace.".to_owned(),
        );
    }
    if metrics.broken_internal_links == Some(0) {
        strengths.push("No broken submitted internal artifact links were found.".to_owned());
    }
    strengths
}

/// Ranks and deduplicates corrections by severity and rubric encounter order.
fn collect_improvements(dimensions: &Dimensions) -> Vec<String> {
    let mut findings = dimensions.all_findings();
    findings.sort_by_key(|finding| std::cmp::Reverse(finding.severity));
    let mut seen = BTreeSet::new();
    findings
        .into_iter()
        .filter(|finding| seen.insert(finding.recommended_correction.clone()))
        .map(|finding| finding.recommended_correction.clone())
        .take(7)
        .collect()
}

/// Formats one optional percentage without replacing missing data with zero.
fn format_optional_percentage(value: Option<f64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{value:.1}%"))
}
