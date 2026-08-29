use std::collections::BTreeSet;

use serde_json::Value;

use crate::corpus::{Artifact, Corpus};

/// Classifies evaluator-owned entities by their observable project role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntityKind {
    /// A user or generated statement describing required behavior or constraints.
    Requirement,
    /// A consequential project or architecture choice.
    Decision,
    /// An inferred choice explicitly presented as an assumption.
    Assumption,
    /// A limit or condition constraining valid decisions.
    Constraint,
    /// A consequential answer supplied during an interactive workflow.
    UserAnswer,
    /// A source or research claim offered in support of a decision.
    EvidenceClaim,
    /// An unresolved question that should remain visible.
    OpenQuestion,
    /// A testable outcome associated with a requirement.
    AcceptanceCriterion,
}

/// Locates one extracted statement in submitted source material.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvidenceRef {
    /// Package-relative artifact name or `ORIGINAL_BRIEF`.
    pub artifact: String,
    /// One-based source line, or one for structured JSON records.
    pub line: usize,
    /// Bounded submitted excerpt supporting the extracted record.
    pub excerpt: String,
    /// Identifier parsed from the same statement when available.
    pub entity_id: Option<String>,
}

/// Represents one normalized evaluator-owned project statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedEntity {
    /// Observable role assigned from an explicit ID or section label.
    pub kind: EntityKind,
    /// Submitted identifier when one was present.
    pub id: Option<String>,
    /// Submitted statement with list syntax removed.
    pub text: String,
    /// Direct source evidence for this entity.
    pub evidence: EvidenceRef,
    /// Other explicit entity identifiers appearing in the statement.
    pub related_ids: Vec<String>,
    /// Whether an inferred assumption was explicitly labeled as such.
    pub explicitly_labeled: bool,
}

/// Represents an explicit relationship between two submitted identifiers.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TraceLink {
    /// Identifier of the statement asserting support or derivation.
    pub from_id: String,
    /// Identifier of the statement receiving that support or derivation.
    pub to_id: String,
    /// Direct source evidence containing both endpoints.
    pub evidence: EvidenceRef,
}

/// Contains normalized entities and validated explicit trace relationships.
#[derive(Clone, Debug, Default)]
pub struct Extraction {
    /// All extracted entities in stable source order.
    pub entities: Vec<ExtractedEntity>,
    /// Explicit links whose two endpoint identifiers both exist.
    pub traces: Vec<TraceLink>,
}

/// Tracks the active semantic section while scanning a text artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SectionKind {
    /// Lines outside a recognized entity section.
    Other,
    /// Requirement statements.
    Requirements,
    /// Decision statements.
    Decisions,
    /// Explicit assumptions.
    Assumptions,
    /// Project constraints.
    Constraints,
    /// User answers.
    UserAnswers,
    /// Evidence and research claims.
    Evidence,
    /// Open questions and unresolved issues.
    OpenQuestions,
    /// Acceptance criteria.
    AcceptanceCriteria,
}

/// Extracts entities from the brief, generated artifacts, and optional project model.
pub fn extract_entities(corpus: &Corpus) -> Extraction {
    let mut entities = Vec::new();
    let mut candidates = Vec::new();
    scan_artifact(&corpus.brief, true, &mut entities, &mut candidates);
    for artifact in &corpus.artifacts {
        scan_artifact(artifact, false, &mut entities, &mut candidates);
    }
    if let Some(model) = &corpus.project_model {
        scan_json_model(model, "project_model", &mut entities, &mut candidates);
    }

    let known_ids = entities
        .iter()
        .filter_map(|entity| entity.id.clone())
        .collect::<BTreeSet<_>>();
    let mut traces = candidates
        .into_iter()
        .filter(|trace| known_ids.contains(&trace.from_id) && known_ids.contains(&trace.to_id))
        .collect::<Vec<_>>();
    traces.sort();
    traces.dedup();
    Extraction { entities, traces }
}

/// Scans one UTF-8 artifact while preserving exact line evidence.
fn scan_artifact(
    artifact: &Artifact,
    is_brief: bool,
    entities: &mut Vec<ExtractedEntity>,
    traces: &mut Vec<TraceLink>,
) {
    let artifact_name = artifact.relative_path.to_string_lossy().replace('\\', "/");
    let mut section = SectionKind::Other;
    for (index, raw_line) in artifact.content.lines().enumerate() {
        let line = raw_line.trim();
        if line.starts_with('#') {
            section = section_from_heading(line);
            continue;
        }
        let Some(kind) = classify_line(line, section, is_brief) else {
            continue;
        };
        let text = clean_statement(line);
        let ids = identifiers_in(&text);
        let id = ids.first().cloned();
        let evidence = EvidenceRef {
            artifact: artifact_name.clone(),
            line: index + 1,
            excerpt: bounded_excerpt(&text),
            entity_id: id.clone(),
        };
        append_trace_candidates(&text, &ids, &evidence, traces);
        entities.push(ExtractedEntity {
            kind,
            related_ids: ids.iter().skip(1).cloned().collect(),
            explicitly_labeled: kind == EntityKind::Assumption,
            id,
            text,
            evidence,
        });
    }
}

/// Maps a Markdown heading to the entity role applied to following lines.
fn section_from_heading(line: &str) -> SectionKind {
    let heading = line.trim_start_matches('#').trim().to_ascii_lowercase();
    if heading.contains("acceptance") {
        SectionKind::AcceptanceCriteria
    } else if heading.contains("requirement") {
        SectionKind::Requirements
    } else if heading.contains("decision") || heading.contains("architecture") {
        SectionKind::Decisions
    } else if heading.contains("assumption") {
        SectionKind::Assumptions
    } else if heading.contains("constraint") {
        SectionKind::Constraints
    } else if heading.contains("user answer") || heading == "answers" {
        SectionKind::UserAnswers
    } else if heading.contains("evidence")
        || heading.contains("research")
        || heading.contains("source")
    {
        SectionKind::Evidence
    } else if heading.contains("open question") || heading.contains("unresolved") {
        SectionKind::OpenQuestions
    } else {
        SectionKind::Other
    }
}

/// Classifies a non-heading line from its explicit ID, section, or normative brief wording.
fn classify_line(line: &str, section: SectionKind, is_brief: bool) -> Option<EntityKind> {
    if line.is_empty() {
        return None;
    }
    if let Some(identifier) = identifiers_in(line).first()
        && let Some(kind) = kind_from_identifier(identifier)
    {
        return Some(kind);
    }
    let section_kind = match section {
        SectionKind::Requirements => Some(EntityKind::Requirement),
        SectionKind::Decisions => Some(EntityKind::Decision),
        SectionKind::Assumptions => Some(EntityKind::Assumption),
        SectionKind::Constraints => Some(EntityKind::Constraint),
        SectionKind::UserAnswers => Some(EntityKind::UserAnswer),
        SectionKind::Evidence => Some(EntityKind::EvidenceClaim),
        SectionKind::OpenQuestions => Some(EntityKind::OpenQuestion),
        SectionKind::AcceptanceCriteria => Some(EntityKind::AcceptanceCriterion),
        SectionKind::Other => None,
    };
    section_kind.or_else(|| is_brief.then(|| normative_kind(line)).flatten())
}

/// Recognizes requirement or question statements in an otherwise unstructured brief.
fn normative_kind(line: &str) -> Option<EntityKind> {
    let lower = line.to_ascii_lowercase();
    if line.ends_with('?') {
        Some(EntityKind::OpenQuestion)
    } else if [" must ", " should ", " need ", " require "]
        .iter()
        .any(|term| format!(" {lower} ").contains(term))
    {
        Some(EntityKind::Requirement)
    } else {
        None
    }
}

/// Maps a recognized identifier prefix to its closed entity role.
fn kind_from_identifier(identifier: &str) -> Option<EntityKind> {
    let prefix = identifier.split_once('-')?.0;
    match prefix {
        "REQ" => Some(EntityKind::Requirement),
        "ADR" | "DEC" => Some(EntityKind::Decision),
        "ASM" | "ASSUMPTION" => Some(EntityKind::Assumption),
        "CON" | "CST" => Some(EntityKind::Constraint),
        "ANS" => Some(EntityKind::UserAnswer),
        "EVD" | "SRC" => Some(EntityKind::EvidenceClaim),
        "Q" | "OQ" => Some(EntityKind::OpenQuestion),
        "AC" => Some(EntityKind::AcceptanceCriterion),
        _ => None,
    }
}

/// Extracts normalized known identifiers in their submitted order.
fn identifiers_in(line: &str) -> Vec<String> {
    line.split_whitespace()
        .map(|word| {
            word.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && character != '-' && character != '_'
            })
            .to_ascii_uppercase()
        })
        .filter(|word| kind_from_identifier(word).is_some())
        .collect()
}

/// Removes common Markdown list syntax without rewriting submitted wording.
fn clean_statement(line: &str) -> String {
    let trimmed = line.trim_start_matches(['-', '*', '+']).trim();
    let without_number = trimmed
        .split_once(". ")
        .filter(|(prefix, _)| prefix.chars().all(|character| character.is_ascii_digit()))
        .map_or(trimmed, |(_, remainder)| remainder);
    without_number.to_owned()
}

/// Limits evidence excerpts so reports cannot reproduce entire submitted documents.
fn bounded_excerpt(text: &str) -> String {
    text.chars().take(240).collect()
}

/// Creates trace candidates only when the statement uses explicit relationship language.
fn append_trace_candidates(
    text: &str,
    ids: &[String],
    evidence: &EvidenceRef,
    traces: &mut Vec<TraceLink>,
) {
    if ids.len() < 2 || !has_relationship_language(text) {
        return;
    }
    for target in ids.iter().skip(1) {
        if target != &ids[0] {
            traces.push(TraceLink {
                from_id: ids[0].clone(),
                to_id: target.clone(),
                evidence: evidence.clone(),
            });
        }
    }
}

/// Detects explicit wording that asserts a relationship rather than co-location alone.
fn has_relationship_language(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "supports",
        "supported by",
        "traces to",
        "addresses",
        "derived from",
        "because",
        "evidence",
        "->",
        "→",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

/// Imports identified records from a schema-tolerant project-model JSON tree.
fn scan_json_model(
    value: &Value,
    context: &str,
    entities: &mut Vec<ExtractedEntity>,
    traces: &mut Vec<TraceLink>,
) {
    match value {
        Value::Object(object) => {
            import_json_object(object, context, entities, traces);
            for (key, child) in object {
                scan_json_model(child, key, entities, traces);
            }
        }
        Value::Array(values) => {
            for child in values {
                scan_json_model(child, context, entities, traces);
            }
        }
        Value::String(text) => import_json_string(text, context, entities, traces),
        _ => {}
    }
}

/// Imports one structured object when it supplies an identifier and descriptive text.
fn import_json_object(
    object: &serde_json::Map<String, Value>,
    context: &str,
    entities: &mut Vec<ExtractedEntity>,
    traces: &mut Vec<TraceLink>,
) {
    let id = object.get("id").and_then(Value::as_str);
    let text = [
        "text",
        "description",
        "title",
        "statement",
        "decision",
        "requirement",
    ]
    .iter()
    .find_map(|key| object.get(*key).and_then(Value::as_str));
    let (Some(id), Some(text)) = (id, text) else {
        return;
    };
    import_json_string(&format!("{id}: {text}"), context, entities, traces);
}

/// Imports one structured string when its context or identifier defines an entity role.
fn import_json_string(
    text: &str,
    context: &str,
    entities: &mut Vec<ExtractedEntity>,
    traces: &mut Vec<TraceLink>,
) {
    let section = section_from_heading(&format!("# {context}"));
    let Some(kind) = classify_line(text, section, false) else {
        return;
    };
    let ids = identifiers_in(text);
    let id = ids.first().cloned();
    let evidence = EvidenceRef {
        artifact: "PROJECT_MODEL.json".to_owned(),
        line: 1,
        excerpt: bounded_excerpt(text),
        entity_id: id.clone(),
    };
    append_trace_candidates(text, &ids, &evidence, traces);
    entities.push(ExtractedEntity {
        kind,
        related_ids: ids.iter().skip(1).cloned().collect(),
        explicitly_labeled: kind == EntityKind::Assumption,
        id,
        text: text.to_owned(),
        evidence,
    });
}
