use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::corpus::Corpus;
use crate::extract::{EntityKind, ExtractedEntity, Extraction};

/// Stores deterministic metrics calculated only from authoritative submitted records.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DeterministicMetrics {
    /// Percentage of metadata-declared required artifacts that exist.
    pub required_artifact_completion: Option<f64>,
    /// Percentage of identified requirements referenced by an acceptance criterion.
    pub acceptance_criteria_coverage: Option<f64>,
    /// Percentage of identified requirements participating in an explicit trace.
    pub requirement_traceability_coverage: Option<f64>,
    /// Percentage of identified decisions lacking explicit valid provenance.
    pub unsupported_decision_rate: Option<f64>,
    /// Percentage of detected high-impact inferences explicitly labeled as assumptions.
    pub high_impact_assumption_labeling_rate: Option<f64>,
    /// Percentage of research-dependent decisions linked to identified evidence.
    pub evidence_linkage_rate: Option<f64>,
    /// Count of resolvable submitted internal links whose targets do not exist.
    pub broken_internal_links: Option<usize>,
    /// Distinct submitted project-name variants discovered from explicit fields.
    pub conflicting_project_names: Vec<String>,
    /// Count of unresolved high or critical findings supplied in metadata.
    pub unresolved_high_severity_findings: Option<usize>,
    /// Identifiers of unresolved high or critical findings supplied in metadata.
    pub unresolved_high_severity_finding_ids: Vec<String>,
    /// Percentage of identified user answers participating in an explicit trace.
    pub user_answer_adoption_rate: Option<f64>,
    /// Percentage of shown questions whose answers materially already existed.
    pub duplicate_question_rate: Option<f64>,
    /// Percentage of resolvable candidate questions answered from existing knowledge.
    pub answer_reuse_rate: Option<f64>,
    /// Ratio of available semantic-project tokens to supplied context tokens.
    pub context_compression_ratio: Option<f64>,
    /// Percentage of retrieved records referenced by evaluator output.
    pub retrieval_utilization_rate: Option<f64>,
    /// Percentage of research operations duplicating existing usable evidence.
    pub repeated_research_rate: Option<f64>,
    /// Percentage of retrieved records that were stale or superseded.
    pub stale_retrieval_rate: Option<f64>,
    /// Number of records retrieved from a different project.
    pub cross_project_leakage: Option<usize>,
    /// Supplied context size reported by execution metadata.
    pub context_supplied: Option<u64>,
    /// Retrieval operation count reported by execution metadata.
    pub retrieval_calls: Option<u64>,
    /// Total execution time in seconds when supplied.
    pub execution_time_seconds: Option<f64>,
    /// Human interaction time in seconds when supplied.
    pub human_interaction_time_seconds: Option<f64>,
    /// Number of questions shown to the user when supplied.
    pub user_questions: Option<u64>,
    /// Number of model calls when supplied.
    pub model_calls: Option<u64>,
    /// Total token usage when supplied.
    pub token_usage: Option<u64>,
    /// Reported execution cost when supplied.
    pub cost: Option<f64>,
}

/// Calculates deterministic metrics without using lexical similarity as arithmetic evidence.
pub fn calculate_metrics(corpus: &Corpus, extraction: &Extraction) -> DeterministicMetrics {
    let required_artifact_completion = required_artifact_completion(corpus);
    let acceptance_criteria_coverage = acceptance_criteria_coverage(extraction);
    let requirement_traceability_coverage = requirement_traceability_coverage(extraction);
    let unsupported_decision_rate = unsupported_decision_rate(extraction);
    let high_impact_assumption_labeling_rate = assumption_labeling_rate(corpus, extraction);
    let evidence_linkage_rate = evidence_linkage_rate(extraction);
    let broken_internal_links = Some(count_broken_internal_links(corpus));
    let conflicting_project_names = project_names(corpus);
    let (unresolved_high_severity_findings, unresolved_high_severity_finding_ids) =
        unresolved_findings(corpus.metadata.as_ref());
    let user_answer_adoption_rate = user_answer_adoption_rate(extraction);
    let duplicate_question_rate =
        metadata_rate(corpus.metadata.as_ref(), "duplicate_question_rate");
    let answer_reuse_rate = metadata_rate(corpus.metadata.as_ref(), "answer_reuse_rate");
    let context_compression_ratio =
        metadata_number(corpus.metadata.as_ref(), "context_compression_ratio");
    let retrieval_utilization_rate =
        metadata_rate(corpus.metadata.as_ref(), "retrieval_utilization_rate");
    let repeated_research_rate = metadata_rate(corpus.metadata.as_ref(), "repeated_research_rate");
    let stale_retrieval_rate = metadata_rate(corpus.metadata.as_ref(), "stale_retrieval_rate");
    let cross_project_leakage = metadata_usize(corpus.metadata.as_ref(), "cross_project_leakage");
    let context_supplied = metadata_u64(corpus.metadata.as_ref(), "context_supplied");
    let retrieval_calls = metadata_u64(corpus.metadata.as_ref(), "retrieval_calls");
    let execution_time_seconds =
        metadata_number(corpus.metadata.as_ref(), "execution_time_seconds");
    let human_interaction_time_seconds =
        metadata_number(corpus.metadata.as_ref(), "human_interaction_time_seconds");
    let user_questions = metadata_u64(corpus.metadata.as_ref(), "user_questions");
    let model_calls = metadata_u64(corpus.metadata.as_ref(), "model_calls");
    let token_usage = metadata_u64(corpus.metadata.as_ref(), "token_usage");
    let cost = metadata_number(corpus.metadata.as_ref(), "cost");

    DeterministicMetrics {
        required_artifact_completion,
        acceptance_criteria_coverage,
        requirement_traceability_coverage,
        unsupported_decision_rate,
        high_impact_assumption_labeling_rate,
        evidence_linkage_rate,
        broken_internal_links,
        conflicting_project_names,
        unresolved_high_severity_findings,
        unresolved_high_severity_finding_ids,
        user_answer_adoption_rate,
        duplicate_question_rate,
        answer_reuse_rate,
        context_compression_ratio,
        retrieval_utilization_rate,
        repeated_research_rate,
        stale_retrieval_rate,
        cross_project_leakage,
        context_supplied,
        retrieval_calls,
        execution_time_seconds,
        human_interaction_time_seconds,
        user_questions,
        model_calls,
        token_usage,
        cost,
    }
}

/// Reads one submitted percentage without normalizing or inventing units.
fn metadata_rate(metadata: Option<&Value>, key: &str) -> Option<f64> {
    metadata_number(metadata, key).filter(|value| (0.0..=100.0).contains(value))
}

/// Reads one finite numeric metadata value from a schema-tolerant tree.
fn metadata_number(metadata: Option<&Value>, key: &str) -> Option<f64> {
    find_key(metadata?, key)?
        .as_f64()
        .filter(|value| value.is_finite())
}

/// Reads one unsigned integer metadata value.
fn metadata_u64(metadata: Option<&Value>, key: &str) -> Option<u64> {
    find_key(metadata?, key)?.as_u64()
}

/// Reads one platform-sized count only when it fits exactly.
fn metadata_usize(metadata: Option<&Value>, key: &str) -> Option<usize> {
    usize::try_from(metadata_u64(metadata, key)?).ok()
}

/// Calculates required-artifact completion from an explicit metadata manifest.
fn required_artifact_completion(corpus: &Corpus) -> Option<f64> {
    let metadata = corpus.metadata.as_ref()?;
    let required = find_key(metadata, "required_artifacts")?.as_array()?;
    let paths = required
        .iter()
        .filter_map(required_artifact_path)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return None;
    }
    let present = corpus
        .artifacts
        .iter()
        .map(|artifact| normalize_path(&artifact.relative_path))
        .collect::<BTreeSet<_>>();
    let found = paths
        .iter()
        .filter(|path| present.contains(&normalize_text_path(path)))
        .count();
    percentage(found, paths.len())
}

/// Extracts a required artifact path from a string or common object fields.
fn required_artifact_path(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned).or_else(|| {
        let object = value.as_object()?;
        ["path", "file", "name", "artifact"]
            .iter()
            .find_map(|key| object.get(*key).and_then(Value::as_str).map(str::to_owned))
    })
}

/// Calculates coverage from explicit requirement IDs referenced by acceptance criteria.
fn acceptance_criteria_coverage(extraction: &Extraction) -> Option<f64> {
    let requirements = ids_of_kind(extraction, EntityKind::Requirement);
    if requirements.is_empty() {
        return None;
    }
    let covered = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::AcceptanceCriterion)
        .flat_map(|entity| entity.related_ids.iter())
        .filter(|identifier| requirements.contains(*identifier))
        .cloned()
        .collect::<BTreeSet<_>>();
    percentage(covered.len(), requirements.len())
}

/// Calculates traceability from explicit links touching identified requirement IDs.
fn requirement_traceability_coverage(extraction: &Extraction) -> Option<f64> {
    let requirements = ids_of_kind(extraction, EntityKind::Requirement);
    if requirements.is_empty() {
        return None;
    }
    let traced = requirements
        .iter()
        .filter(|identifier| trace_touches(extraction, identifier))
        .count();
    percentage(traced, requirements.len())
}

/// Calculates the share of identified decisions with no explicit provenance endpoint.
fn unsupported_decision_rate(extraction: &Extraction) -> Option<f64> {
    let decisions = entities_by_id(extraction, EntityKind::Decision);
    if decisions.is_empty() {
        return None;
    }
    let unsupported = decisions
        .iter()
        .filter(|(identifier, entities)| {
            !entities
                .iter()
                .any(|entity| entity_has_valid_provenance(entity, extraction, identifier))
        })
        .count();
    percentage(unsupported, decisions.len())
}

/// Determines whether a decision has a valid explicit source, requirement, answer, or assumption.
fn entity_has_valid_provenance(
    entity: &ExtractedEntity,
    extraction: &Extraction,
    identifier: &str,
) -> bool {
    entity
        .related_ids
        .iter()
        .any(|related| is_provenance_identifier(related))
        || extraction.traces.iter().any(|trace| {
            (trace.from_id == identifier && is_provenance_identifier(&trace.to_id))
                || (trace.to_id == identifier && is_provenance_identifier(&trace.from_id))
        })
}

/// Recognizes identifier prefixes accepted as decision provenance.
fn is_provenance_identifier(identifier: &str) -> bool {
    [
        "REQ-",
        "ANS-",
        "ASM-",
        "ASSUMPTION-",
        "CON-",
        "CST-",
        "EVD-",
        "SRC-",
    ]
    .iter()
    .any(|prefix| identifier.starts_with(prefix))
}

/// Calculates labeling coverage for explicit and implicit high-impact inferred choices.
fn assumption_labeling_rate(corpus: &Corpus, extraction: &Extraction) -> Option<f64> {
    let brief = corpus.brief.content.to_ascii_lowercase();
    let labeled = extraction
        .entities
        .iter()
        .filter(|entity| {
            entity.kind == EntityKind::Assumption && contains_high_impact_concept(&entity.text)
        })
        .count();
    let implicit = extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Decision)
        .filter(|entity| contains_high_impact_concept(&entity.text))
        .filter(|entity| !concept_is_present_in_brief(&entity.text, &brief))
        .filter(|entity| entity.related_ids.is_empty())
        .count();
    percentage(labeled, labeled + implicit)
}

/// Detects consequential platform, persistence, identity, deployment, and business choices.
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

/// Checks whether a high-impact concept in a decision was explicitly present in the brief.
fn concept_is_present_in_brief(text: &str, brief: &str) -> bool {
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
    .any(|concept| lower.contains(concept) && brief.contains(concept))
}

/// Calculates explicit evidence linkage for decisions whose text claims research dependence.
fn evidence_linkage_rate(extraction: &Extraction) -> Option<f64> {
    let decisions = entities_by_id(extraction, EntityKind::Decision);
    let research_dependent = decisions
        .iter()
        .filter(|(_, entities)| entities.iter().any(|entity| research_dependent(entity)))
        .collect::<Vec<_>>();
    if research_dependent.is_empty() {
        return None;
    }
    let linked = research_dependent
        .iter()
        .filter(|(identifier, entities)| {
            entities.iter().any(|entity| {
                entity
                    .related_ids
                    .iter()
                    .any(|related| is_evidence_identifier(related))
                    || trace_links_evidence(extraction, identifier)
            })
        })
        .count();
    percentage(linked, research_dependent.len())
}

/// Detects decisions that explicitly claim research, evidence, or externally verified support.
fn research_dependent(entity: &ExtractedEntity) -> bool {
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
        || entity
            .related_ids
            .iter()
            .any(|identifier| is_evidence_identifier(identifier))
}

/// Recognizes evidence-record identifier prefixes.
fn is_evidence_identifier(identifier: &str) -> bool {
    identifier.starts_with("EVD-") || identifier.starts_with("SRC-")
}

/// Checks whether an explicit trace connects a decision to evidence.
fn trace_links_evidence(extraction: &Extraction, identifier: &str) -> bool {
    extraction.traces.iter().any(|trace| {
        (trace.from_id == identifier && is_evidence_identifier(&trace.to_id))
            || (trace.to_id == identifier && is_evidence_identifier(&trace.from_id))
    })
}

/// Counts submitted relative Markdown links whose normalized target is absent.
fn count_broken_internal_links(corpus: &Corpus) -> usize {
    let present = corpus
        .artifacts
        .iter()
        .map(|artifact| normalize_path(&artifact.relative_path))
        .collect::<BTreeSet<_>>();
    corpus
        .artifacts
        .iter()
        .map(|artifact| {
            internal_link_targets(&artifact.content)
                .into_iter()
                .filter(|target| {
                    let resolved = resolve_link(&artifact.relative_path, target);
                    !present.contains(&resolved)
                })
                .count()
        })
        .sum()
}

/// Extracts non-external, non-anchor Markdown link targets from submitted text.
fn internal_link_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut remainder = content;
    while let Some(start) = remainder.find("](") {
        remainder = &remainder[start + 2..];
        let Some(end) = remainder.find(')') else {
            break;
        };
        let raw = remainder[..end].trim().trim_matches('<').trim_matches('>');
        let target = raw.split(" \"").next().unwrap_or(raw).trim();
        if !is_external_link(target) {
            let path = target.split(['#', '?']).next().unwrap_or_default();
            if !path.is_empty() {
                targets.push(path.to_owned());
            }
        }
        remainder = &remainder[end + 1..];
    }
    targets
}

/// Distinguishes web, mail, data, and same-document links from file targets.
fn is_external_link(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    target.starts_with('#')
        || lower.contains("://")
        || ["mailto:", "tel:", "data:", "javascript:"]
            .iter()
            .any(|prefix| lower.starts_with(prefix))
}

/// Resolves one relative link against its source artifact using lexical components.
fn resolve_link(source: &Path, target: &str) -> String {
    let parent = source.parent().unwrap_or_else(|| Path::new(""));
    normalize_path(&parent.join(target.replace('\\', "/")))
}

/// Normalizes a package-relative path for stable case-insensitive comparison.
fn normalize_path(path: &Path) -> String {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().to_string()),
            Component::ParentDir => {
                components.pop();
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    components.join("/").to_ascii_lowercase()
}

/// Normalizes a textual artifact path using the same component rules as submitted files.
fn normalize_text_path(path: &str) -> String {
    normalize_path(&PathBuf::from(path.replace('\\', "/")))
}

/// Extracts explicit project-name fields without inferring names from directory identity.
fn project_names(corpus: &Corpus) -> Vec<String> {
    let mut names = BTreeSet::new();
    if let Some(model) = &corpus.project_model {
        collect_named_values(model, &mut names);
    }
    for artifact in &corpus.artifacts {
        for line in artifact.content.lines() {
            let trimmed = line.trim().trim_start_matches('#').trim();
            let lower = trimmed.to_ascii_lowercase();
            if let Some((_, name)) = trimmed.split_once(':')
                && (lower.starts_with("project name:") || lower.starts_with("project:"))
            {
                names.insert(name.trim().to_owned());
            }
        }
    }
    names.into_iter().filter(|name| !name.is_empty()).collect()
}

/// Recursively collects explicit `project_name` values from structured model data.
fn collect_named_values(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            if let Some(name) = object.get("project_name").and_then(Value::as_str) {
                names.insert(name.trim().to_owned());
            }
            for child in object.values() {
                collect_named_values(child, names);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_named_values(child, names);
            }
        }
        _ => {}
    }
}

/// Calculates unresolved high-severity finding counts and IDs from metadata.
fn unresolved_findings(metadata: Option<&Value>) -> (Option<usize>, Vec<String>) {
    let Some(metadata) = metadata else {
        return (None, Vec::new());
    };
    let Some(findings) = find_key(metadata, "findings").and_then(Value::as_array) else {
        return (None, Vec::new());
    };
    let unresolved = findings
        .iter()
        .filter_map(Value::as_object)
        .filter(|finding| is_unresolved_high_severity(finding))
        .collect::<Vec<_>>();
    let ids = unresolved
        .iter()
        .filter_map(|finding| finding.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect();
    (Some(unresolved.len()), ids)
}

/// Classifies one metadata finding by severity and resolution state.
fn is_unresolved_high_severity(finding: &serde_json::Map<String, Value>) -> bool {
    let severity = finding
        .get("severity")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_uppercase();
    let high = severity == "HIGH" || severity == "CRITICAL";
    let resolved = finding
        .get("resolved")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            finding
                .get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status.eq_ignore_ascii_case("resolved"))
        });
    high && !resolved
}

/// Calculates adoption from identified user-answer IDs participating in explicit traces.
fn user_answer_adoption_rate(extraction: &Extraction) -> Option<f64> {
    let answers = ids_of_kind(extraction, EntityKind::UserAnswer);
    if answers.is_empty() {
        return None;
    }
    let adopted = answers
        .iter()
        .filter(|identifier| trace_touches(extraction, identifier))
        .count();
    percentage(adopted, answers.len())
}

/// Returns unique explicit IDs for one entity kind.
fn ids_of_kind(extraction: &Extraction, kind: EntityKind) -> BTreeSet<String> {
    extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == kind)
        .filter_map(|entity| entity.id.clone())
        .collect()
}

/// Groups identified entities by stable ID for denominator-safe calculations.
fn entities_by_id(
    extraction: &Extraction,
    kind: EntityKind,
) -> BTreeMap<String, Vec<&ExtractedEntity>> {
    let mut grouped = BTreeMap::<String, Vec<&ExtractedEntity>>::new();
    for entity in extraction
        .entities
        .iter()
        .filter(|entity| entity.kind == kind)
    {
        if let Some(identifier) = &entity.id {
            grouped.entry(identifier.clone()).or_default().push(entity);
        }
    }
    grouped
}

/// Checks whether an identifier is an endpoint of any validated trace.
fn trace_touches(extraction: &Extraction, identifier: &str) -> bool {
    extraction
        .traces
        .iter()
        .any(|trace| trace.from_id == identifier || trace.to_id == identifier)
}

/// Recursively locates the first named key in schema-tolerant metadata.
fn find_key<'a>(value: &'a Value, target: &str) -> Option<&'a Value> {
    match value {
        Value::Object(object) => object
            .get(target)
            .or_else(|| object.values().find_map(|child| find_key(child, target))),
        Value::Array(values) => values.iter().find_map(|child| find_key(child, target)),
        _ => None,
    }
}

/// Converts a numerator and non-zero denominator into a one-decimal percentage.
fn percentage(numerator: usize, denominator: usize) -> Option<f64> {
    if denominator == 0 {
        return None;
    }
    Some(((numerator as f64 / denominator as f64) * 1_000.0).round() / 10.0)
}
