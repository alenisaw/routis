#![forbid(unsafe_code)]
#![deny(warnings)]

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

pub mod trace;
pub use trace::{
    DecisionTrace, DecisionTraceInput, MatchedSignal, PromptMode, ProviderCommandPreview, RepoFact,
    RouteTree, RouteTreeNode, DECISION_TRACE_SCHEMA_VERSION,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    #[error("unknown policy `{0}`; expected cheap, balanced, deep, extradeep, or default")]
    UnknownPolicy(String),
    #[error("task must not be empty")]
    EmptyTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Cheap,
    Balanced,
    Deep,
    ExtraDeep,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskZone {
    Config,
    Auth,
    Schema,
    Workflow,
    Package,
    Tests,
    Docs,
    Ui,
}

impl RiskZone {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Auth => "auth",
            Self::Schema => "schema",
            Self::Workflow => "workflow",
            Self::Package => "package",
            Self::Tests => "tests",
            Self::Docs => "docs",
            Self::Ui => "ui",
        }
    }
}

impl Profile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cheap => "cheap",
            Self::Balanced => "balanced",
            Self::Deep => "deep",
            Self::ExtraDeep => "extradeep",
            Self::Default => "default",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Profile {
    type Err = RoutingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "cheap" => Ok(Self::Cheap),
            "balanced" => Ok(Self::Balanced),
            "deep" => Ok(Self::Deep),
            "extradeep" | "extra-deep" | "extra_deep" => Ok(Self::ExtraDeep),
            "default" | "auto" => Ok(Self::Default),
            other => Err(RoutingError::UnknownPolicy(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub requested_profile: Profile,
    pub effective_profile: Profile,
    pub signals_matched: Vec<String>,
    pub explain: String,
    pub classification: TaskClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageHint {
    English,
    Russian,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentKind {
    Docs,
    Check,
    Fix,
    Debug,
    Create,
    Refactor,
    Analyze,
    Test,
    Ui,
    Setup,
    Release,
    Security,
    Migration,
    Architecture,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AreaKind {
    Docs,
    Tests,
    Ui,
    Api,
    Data,
    Auth,
    Security,
    Config,
    Build,
    Workflow,
    Dependencies,
    Release,
    Routing,
    Context,
    Session,
    Policy,
    Repo,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeKind {
    SingleFile,
    Focused,
    Subsystem,
    RepoWide,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetHint {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEvidence {
    pub kind: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskClassification {
    pub language: LanguageHint,
    pub primary_intent: IntentKind,
    pub secondary_intents: Vec<IntentKind>,
    pub area: AreaKind,
    pub scope: ScopeKind,
    pub risk: RiskLevel,
    pub confidence: Confidence,
    pub targets: Vec<TargetHint>,
    pub evidence: Vec<RouteEvidence>,
}

impl IntentKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Check => "check",
            Self::Fix => "fix",
            Self::Debug => "debug",
            Self::Create => "create",
            Self::Refactor => "refactor",
            Self::Analyze => "analyze",
            Self::Test => "test",
            Self::Ui => "ui",
            Self::Setup => "setup",
            Self::Release => "release",
            Self::Security => "security",
            Self::Migration => "migration",
            Self::Architecture => "architecture",
            Self::Unknown => "unknown",
        }
    }
}

impl AreaKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Tests => "tests",
            Self::Ui => "ui",
            Self::Api => "api",
            Self::Data => "data",
            Self::Auth => "auth",
            Self::Security => "security",
            Self::Config => "config",
            Self::Build => "build",
            Self::Workflow => "workflow",
            Self::Dependencies => "dependencies",
            Self::Release => "release",
            Self::Routing => "routing",
            Self::Context => "context",
            Self::Session => "session",
            Self::Policy => "policy",
            Self::Repo => "repo",
            Self::Unknown => "unknown",
        }
    }
}

impl ScopeKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingleFile => "single-file",
            Self::Focused => "focused",
            Self::Subsystem => "subsystem",
            Self::RepoWide => "repo-wide",
            Self::Unknown => "unknown",
        }
    }
}

impl RiskLevel {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl Confidence {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

pub fn route_task(task: &str, requested_profile: Profile) -> Result<RoutingDecision, RoutingError> {
    let task = task.trim();
    if task.is_empty() {
        return Err(RoutingError::EmptyTask);
    }

    let classification = classify_task_for_profile(task);
    let effective_profile = match requested_profile {
        Profile::Default => classification.profile,
        fixed => fixed,
    };

    let explain = explain_decision(
        requested_profile,
        effective_profile,
        &classification.signals_matched,
        &classification.task,
    );

    Ok(RoutingDecision {
        requested_profile,
        effective_profile,
        signals_matched: classification.signals_matched,
        explain,
        classification: classification.task,
    })
}

pub fn route_task_with_repo_context(
    task: &str,
    requested_profile: Profile,
    risk_zones: &[RiskZone],
    changed_file_count: usize,
) -> Result<RoutingDecision, RoutingError> {
    let mut decision = route_task(task, requested_profile)?;
    if requested_profile != Profile::Default {
        return Ok(decision);
    }

    let repo_profile = repo_context_min_profile(risk_zones, changed_file_count);
    if let Some(profile) = repo_profile {
        decision.effective_profile = max_profile(decision.effective_profile, profile);
        for zone in risk_zones {
            decision
                .signals_matched
                .push(format!("risk-zone:{}", zone.as_str()));
        }
        if changed_file_count >= 6 {
            decision
                .signals_matched
                .push(format!("changed-files:{changed_file_count}"));
        }
        decision.explain = explain_decision(
            requested_profile,
            decision.effective_profile,
            &decision.signals_matched,
            &decision.classification,
        );
    }

    Ok(decision)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Classification {
    profile: Profile,
    signals_matched: Vec<String>,
    task: TaskClassification,
}

#[must_use]
pub fn classify_task(task: &str) -> TaskClassification {
    classify_task_for_profile(task).task
}

fn classify_task_for_profile(task: &str) -> Classification {
    let normalized = task.to_ascii_lowercase();
    let language = detect_language(task);
    let segments = split_task_segments(&normalized);
    let mut matched = Vec::new();
    let mut evidence = Vec::new();
    let mut intent_scores: Vec<(IntentKind, i32)> = Vec::new();
    let mut area_scores: Vec<(AreaKind, i32)> = Vec::new();

    for segment in &segments {
        score_intents(segment, &mut intent_scores, &mut evidence);
        score_areas(segment, &mut area_scores, &mut evidence);
    }

    let primary_intent = best_intent(&intent_scores).unwrap_or(IntentKind::Unknown);
    let area = best_area(&area_scores).unwrap_or_else(|| area_from_intent(primary_intent));
    let secondary_intents = secondary_intents(&intent_scores, primary_intent);
    let scope = classify_scope(&normalized);
    let targets = target_hints(task);
    let risk = classify_risk(primary_intent, area, &normalized);
    let confidence = classify_confidence(&evidence, primary_intent, area, scope);
    let mut resolved_score = profile_score(primary_intent, area, scope, risk);

    if contains_any(
        &normalized,
        &["quickly", "quick", "just this file", "only this file"],
    ) && !matches!(risk, RiskLevel::High)
    {
        matched.push("down-modifier".to_string());
        resolved_score -= 1;
    }

    if contains_any(
        &normalized,
        &["carefully", "thoroughly", "deeply", "comprehensively"],
    ) {
        matched.push("up-modifier".to_string());
        resolved_score += 1;
    }

    if matches!(scope, ScopeKind::RepoWide) {
        matched.push("scope:repo-wide".to_string());
    }

    if matches!(primary_intent, IntentKind::Check | IntentKind::Docs)
        && matches!(area, AreaKind::Docs)
        && !has_stronger_non_docs_intent(&intent_scores)
    {
        matched.push("docs-primary-cap".to_string());
        resolved_score = resolved_score.min(0);
    }

    if matches!(area, AreaKind::Routing | AreaKind::Context)
        && matches!(
            primary_intent,
            IntentKind::Fix | IntentKind::Refactor | IntentKind::Analyze
        )
    {
        matched.push(format!("area:{}", area.as_str()));
        resolved_score = resolved_score.max(2);
    }

    for item in &evidence {
        matched.push(format!("{}:{}", item.kind, item.value));
    }

    if matched.is_empty() {
        matched.push("balanced-baseline".to_string());
    }

    let profile = match resolved_score.clamp(0, 3) {
        0 => Profile::Cheap,
        1 => Profile::Balanced,
        2 => Profile::Deep,
        _ => Profile::ExtraDeep,
    };

    let task = TaskClassification {
        language,
        primary_intent,
        secondary_intents,
        area,
        scope,
        risk,
        confidence,
        targets,
        evidence,
    };

    Classification {
        profile,
        signals_matched: matched,
        task,
    }
}

fn contains_any(text: &str, signals: &[&str]) -> bool {
    signals.iter().any(|signal| term_matches(text, signal))
}

fn term_matches(text: &str, signal: &str) -> bool {
    if signal.contains(' ') || signal.contains('/') || signal.len() > 3 {
        return text.contains(signal);
    }
    text.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .any(|token| token == signal)
}

fn detect_language(task: &str) -> LanguageHint {
    let has_ascii = task.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_cyrillic = task
        .chars()
        .any(|ch| ('\u{0400}'..='\u{04ff}').contains(&ch));
    match (has_ascii, has_cyrillic) {
        (true, true) => LanguageHint::Mixed,
        (true, false) => LanguageHint::English,
        (false, true) => LanguageHint::Russian,
        (false, false) => LanguageHint::Unknown,
    }
}

fn split_task_segments(task: &str) -> Vec<String> {
    let normalized = task
        .replace("\r\n", "\n")
        .replace(" however ", "\n")
        .replace(" but ", "\n")
        .replace(" also ", "\n");
    normalized
        .split(['\n', '.', ';', ':', '!', '?'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn score_intents(
    segment: &str,
    scores: &mut Vec<(IntentKind, i32)>,
    evidence: &mut Vec<RouteEvidence>,
) {
    const INTENTS: &[(IntentKind, i32, &[&str])] = &[
        (
            IntentKind::Docs,
            1,
            &[
                "readme",
                "docs",
                "documentation",
                "changelog",
                "документация",
            ],
        ),
        (
            IntentKind::Check,
            1,
            &[
                "check",
                "review",
                "inspect",
                "verify",
                "validate",
                "проверь",
                "чекни",
            ],
        ),
        (
            IntentKind::Fix,
            3,
            &[
                "fix",
                "repair",
                "resolve",
                "bug",
                "issue",
                "broken",
                "исправь",
                "фикс",
                "пофикси",
            ],
        ),
        (
            IntentKind::Debug,
            4,
            &["debug", "stack trace", "investigate", "root cause"],
        ),
        (
            IntentKind::Create,
            3,
            &["create", "add", "implement", "new module", "feature"],
        ),
        (
            IntentKind::Refactor,
            4,
            &["refactor", "rework", "rewrite", "redesign", "overhaul"],
        ),
        (
            IntentKind::Analyze,
            3,
            &["analyze", "analyse", "study", "understand", "audit"],
        ),
        (
            IntentKind::Test,
            2,
            &["test", "tests", "coverage", "regression", "тест", "тесты"],
        ),
        (
            IntentKind::Ui,
            2,
            &["ui", "tui", "layout", "screen", "theme", "colors"],
        ),
        (
            IntentKind::Setup,
            2,
            &["setup", "wizard", "onboarding", "provider check"],
        ),
        (
            IntentKind::Release,
            3,
            &[
                "release",
                "version",
                "changelog",
                "publish",
                "релиз",
                "версия",
            ],
        ),
        (
            IntentKind::Security,
            5,
            &[
                "security",
                "secret",
                "token",
                "auth leak",
                "безопасность",
                "уязвимость",
            ],
        ),
        (
            IntentKind::Migration,
            5,
            &["migration", "schema", "database migration"],
        ),
        (
            IntentKind::Architecture,
            6,
            &[
                "architecture",
                "system design",
                "core design",
                "архитектура",
            ],
        ),
    ];
    for (intent, weight, terms) in INTENTS {
        if contains_any(segment, terms) {
            scores.push((*intent, *weight));
            evidence.push(RouteEvidence {
                kind: "intent".to_string(),
                value: intent.as_str().to_string(),
                weight: *weight,
            });
        }
    }
}

fn score_areas(
    segment: &str,
    scores: &mut Vec<(AreaKind, i32)>,
    evidence: &mut Vec<RouteEvidence>,
) {
    const AREAS: &[(AreaKind, i32, &[&str])] = &[
        (
            AreaKind::Docs,
            1,
            &["readme", "docs", "documentation", "changelog"],
        ),
        (
            AreaKind::Tests,
            2,
            &["test", "tests", "coverage", "regression", "тест", "тесты"],
        ),
        (
            AreaKind::Ui,
            3,
            &[
                "ui",
                "tui",
                "layout",
                "screen",
                "theme",
                "colors",
                "mascot",
                "интерфейс",
                "экран",
            ],
        ),
        (
            AreaKind::Api,
            2,
            &["api", "endpoint", "request", "response"],
        ),
        (
            AreaKind::Data,
            3,
            &["database", "schema", "sql", "migration", "data"],
        ),
        (
            AreaKind::Auth,
            5,
            &["auth", "login", "permission", "session token"],
        ),
        (
            AreaKind::Security,
            5,
            &[
                "security",
                "secret",
                "token",
                "private key",
                "безопасность",
                "уязвимость",
            ],
        ),
        (
            AreaKind::Config,
            3,
            &[
                "config",
                "configuration",
                "settings",
                "policy file",
                "конфиг",
                "настройка",
            ],
        ),
        (
            AreaKind::Build,
            3,
            &["build", "compile", "cargo", "dependency"],
        ),
        (
            AreaKind::Workflow,
            4,
            &["ci", "workflow", "github actions", "release workflow"],
        ),
        (
            AreaKind::Dependencies,
            3,
            &["dependency", "dependencies", "lockfile", "package"],
        ),
        (
            AreaKind::Release,
            3,
            &[
                "release",
                "version",
                "changelog",
                "publish",
                "релиз",
                "версия",
            ],
        ),
        (
            AreaKind::Routing,
            5,
            &[
                "routing",
                "route",
                "classifier",
                "classification",
                "profile",
                "роутинг",
                "маршрутизация",
            ],
        ),
        (
            AreaKind::Context,
            5,
            &["context", "repo map", "repomap", "repository context"],
        ),
        (AreaKind::Session, 3, &["session", "history", "resume"]),
        (AreaKind::Policy, 4, &["policy", "rules", "risk zone"]),
        (
            AreaKind::Repo,
            2,
            &["repo", "repository", "project", "codebase"],
        ),
    ];
    for (area, weight, terms) in AREAS {
        if contains_any(segment, terms) {
            scores.push((*area, *weight));
            evidence.push(RouteEvidence {
                kind: "area".to_string(),
                value: area.as_str().to_string(),
                weight: *weight,
            });
        }
    }
}

fn best_intent(scores: &[(IntentKind, i32)]) -> Option<IntentKind> {
    scores
        .iter()
        .max_by_key(|(_, score)| *score)
        .map(|(kind, _)| *kind)
}

fn best_area(scores: &[(AreaKind, i32)]) -> Option<AreaKind> {
    scores
        .iter()
        .fold(None, |best, (kind, score)| match best {
            Some((best_kind, best_score)) if best_score >= *score => Some((best_kind, best_score)),
            _ => Some((*kind, *score)),
        })
        .map(|(kind, _)| kind)
}

fn secondary_intents(scores: &[(IntentKind, i32)], primary: IntentKind) -> Vec<IntentKind> {
    let mut values = scores
        .iter()
        .filter_map(|(kind, _)| (*kind != primary).then_some(*kind))
        .collect::<Vec<_>>();
    values.sort_by_key(|kind| kind.as_str());
    values.dedup();
    values
}

fn area_from_intent(intent: IntentKind) -> AreaKind {
    match intent {
        IntentKind::Docs => AreaKind::Docs,
        IntentKind::Test => AreaKind::Tests,
        IntentKind::Ui => AreaKind::Ui,
        IntentKind::Setup => AreaKind::Config,
        IntentKind::Release => AreaKind::Release,
        IntentKind::Security => AreaKind::Security,
        IntentKind::Migration => AreaKind::Data,
        _ => AreaKind::Unknown,
    }
}

fn classify_scope(task: &str) -> ScopeKind {
    if contains_any(
        task,
        &[
            "whole repo",
            "entire repo",
            "repository",
            "codebase",
            "all files",
            "whole project",
        ],
    ) {
        ScopeKind::RepoWide
    } else if contains_any(task, &["subsystem", "module", "package", "area", "flow"]) {
        ScopeKind::Subsystem
    } else if contains_any(
        task,
        &["this file", "single file", ".rs", ".ts", ".js", ".md"],
    ) {
        ScopeKind::SingleFile
    } else {
        ScopeKind::Focused
    }
}

fn classify_risk(intent: IntentKind, area: AreaKind, task: &str) -> RiskLevel {
    if matches!(
        intent,
        IntentKind::Security | IntentKind::Migration | IntentKind::Architecture
    ) || matches!(
        area,
        AreaKind::Auth | AreaKind::Security | AreaKind::Data | AreaKind::Workflow
    ) {
        RiskLevel::High
    } else if contains_any(
        task,
        &["refactor", "rewrite", "routing", "context", "policy"],
    ) {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

fn classify_confidence(
    evidence: &[RouteEvidence],
    intent: IntentKind,
    area: AreaKind,
    scope: ScopeKind,
) -> Confidence {
    let strength = evidence.iter().map(|item| item.weight.max(0)).sum::<i32>();
    if !matches!(intent, IntentKind::Unknown) && !matches!(area, AreaKind::Unknown) && strength >= 8
    {
        Confidence::High
    } else if !matches!(scope, ScopeKind::Unknown) && strength >= 3 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

fn target_hints(task: &str) -> Vec<TargetHint> {
    task.split_whitespace()
        .filter(|part| {
            let value =
                part.trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == ',' || ch == '.');
            value.contains('/')
                || value.ends_with(".rs")
                || value.ends_with(".md")
                || value.ends_with(".toml")
                || value.ends_with(".yaml")
                || value.ends_with(".yml")
                || value.ends_with(".json")
        })
        .map(|value| TargetHint {
            value: value
                .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == ',' || ch == '.')
                .to_string(),
        })
        .collect()
}

fn profile_score(intent: IntentKind, area: AreaKind, scope: ScopeKind, risk: RiskLevel) -> i32 {
    let mut score = match intent {
        IntentKind::Docs | IntentKind::Check => 0,
        IntentKind::Create
        | IntentKind::Fix
        | IntentKind::Test
        | IntentKind::Ui
        | IntentKind::Setup => 1,
        IntentKind::Debug | IntentKind::Analyze | IntentKind::Refactor | IntentKind::Release => 2,
        IntentKind::Security | IntentKind::Migration | IntentKind::Architecture => 3,
        IntentKind::Unknown => 1,
    };
    if matches!(
        area,
        AreaKind::Auth
            | AreaKind::Security
            | AreaKind::Data
            | AreaKind::Workflow
            | AreaKind::Routing
            | AreaKind::Context
            | AreaKind::Policy
    ) {
        score = score.max(2);
    }
    if matches!(scope, ScopeKind::RepoWide) {
        score += 1;
    }
    if matches!(scope, ScopeKind::Subsystem)
        && matches!(
            intent,
            IntentKind::Create
                | IntentKind::Refactor
                | IntentKind::Analyze
                | IntentKind::Architecture
        )
    {
        score = score.max(2);
    }
    if matches!(area, AreaKind::Docs)
        && matches!(
            intent,
            IntentKind::Fix | IntentKind::Check | IntentKind::Docs
        )
    {
        score = score.min(0);
    }
    if matches!(risk, RiskLevel::High) {
        score = score.max(2);
    }
    score
}

fn has_stronger_non_docs_intent(scores: &[(IntentKind, i32)]) -> bool {
    scores.iter().any(|(intent, score)| {
        !matches!(intent, IntentKind::Docs | IntentKind::Check) && *score >= 3
    })
}

fn explain_decision(
    requested: Profile,
    effective: Profile,
    signals: &[String],
    task: &TaskClassification,
) -> String {
    let route = format!(
        "{} / {} / scope {} / confidence {}",
        task.primary_intent.as_str(),
        task.area.as_str(),
        task.scope.as_str(),
        task.confidence.as_str()
    );
    if requested == Profile::Default {
        format!(
            "Auto-selected `{}` for {route} from signals: {}.",
            effective,
            signals.join(", ")
        )
    } else {
        format!(
            "Using explicit `{}` policy for {route}; classifier signals were: {}.",
            effective,
            signals.join(", ")
        )
    }
}

fn repo_context_min_profile(risk_zones: &[RiskZone], changed_file_count: usize) -> Option<Profile> {
    let high_risk = risk_zones.iter().any(|zone| {
        matches!(
            zone,
            RiskZone::Auth | RiskZone::Schema | RiskZone::Workflow | RiskZone::Package
        )
    });
    if high_risk {
        return Some(Profile::Deep);
    }
    if changed_file_count >= 12 {
        return Some(Profile::ExtraDeep);
    }
    if changed_file_count >= 6 {
        return Some(Profile::Deep);
    }
    None
}

fn max_profile(left: Profile, right: Profile) -> Profile {
    if profile_rank(left) >= profile_rank(right) {
        left
    } else {
        right
    }
}

fn profile_rank(profile: Profile) -> u8 {
    match profile {
        Profile::Cheap => 0,
        Profile::Balanced | Profile::Default => 1,
        Profile::Deep => 2,
        Profile::ExtraDeep => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheap_signal_routes_to_cheap() {
        let decision = route_task("small fix in README", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn balanced_signal_routes_to_balanced() {
        let decision = route_task("implement a new parser option", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Balanced);
    }

    #[test]
    fn deep_signal_routes_to_deep() {
        let decision = route_task("debug auth flow", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Deep);
    }

    #[test]
    fn up_modifier_elevates_deep_to_extradeep() {
        let decision = route_task("debug auth edge case carefully", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::ExtraDeep);
    }

    #[test]
    fn extradeep_signal_routes_to_extradeep() {
        let decision = route_task("redesign the architecture", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::ExtraDeep);
    }

    #[test]
    fn down_modifier_can_lower_routing_profile() {
        let decision = route_task("debug just this file", Profile::Default).unwrap();
        assert_eq!(decision.effective_profile, Profile::Balanced);
    }

    #[test]
    fn explicit_policy_overrides_classifier() {
        let decision = route_task("redesign the architecture", Profile::Cheap).unwrap();
        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn repo_risk_zone_elevates_default_routing_profile() {
        let decision =
            route_task_with_repo_context("small fix", Profile::Default, &[RiskZone::Auth], 1)
                .unwrap();

        assert_eq!(decision.effective_profile, Profile::Deep);
        assert!(decision
            .signals_matched
            .contains(&"risk-zone:auth".to_string()));
    }

    #[test]
    fn explicit_policy_is_not_elevated_by_repo_context() {
        let decision =
            route_task_with_repo_context("small fix", Profile::Cheap, &[RiskZone::Auth], 1)
                .unwrap();

        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn parses_policy_aliases() {
        assert_eq!("extra-deep".parse::<Profile>().unwrap(), Profile::ExtraDeep);
        assert_eq!("auto".parse::<Profile>().unwrap(), Profile::Default);
    }

    #[test]
    fn readme_check_stays_cheap() {
        let decision = route_task("check README.md for typos", Profile::Default).unwrap();

        assert_eq!(decision.effective_profile, Profile::Cheap);
    }

    #[test]
    fn long_prompt_keeps_primary_routing_work_over_docs_mention() {
        let decision = route_task(
            "check README later, but the main task is to refactor routing and repo context so long prompts classify correctly",
            Profile::Default,
        )
        .unwrap();

        assert_eq!(decision.effective_profile, Profile::Deep);
        assert_eq!(decision.classification.area, AreaKind::Routing);
        assert_eq!(decision.classification.primary_intent, IntentKind::Refactor);
    }

    #[test]
    fn universal_ui_area_does_not_require_routis_paths() {
        let decision = route_task(
            "fix the dashboard layout and theme colors",
            Profile::Default,
        )
        .unwrap();

        assert_eq!(decision.classification.area, AreaKind::Ui);
        assert_eq!(decision.classification.primary_intent, IntentKind::Fix);
    }
}
