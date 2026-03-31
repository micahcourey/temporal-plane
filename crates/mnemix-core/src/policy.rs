//! Policy configuration and evaluation for workflow-aware memory requirements.

use std::{collections::BTreeSet, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::ScopeId;

/// Supported host workflow triggers for policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTrigger {
    /// A host is starting a task.
    OnTaskStart,
    /// A commit is about to be created.
    OnGitCommit,
    /// A pull request or equivalent review request is being opened.
    OnPrOpen,
    /// A review workflow is starting.
    OnReviewStart,
    /// Release preparation is underway.
    OnReleasePrep,
    /// A risky change such as a migration is starting.
    OnRiskyChange,
}

impl PolicyTrigger {
    /// Returns the stable string representation used by config and CLI surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OnTaskStart => "on_task_start",
            Self::OnGitCommit => "on_git_commit",
            Self::OnPrOpen => "on_pr_open",
            Self::OnReviewStart => "on_review_start",
            Self::OnReleasePrep => "on_release_prep",
            Self::OnRiskyChange => "on_risky_change",
        }
    }
}

impl FromStr for PolicyTrigger {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "on_task_start" => Ok(Self::OnTaskStart),
            "on_git_commit" => Ok(Self::OnGitCommit),
            "on_pr_open" => Ok(Self::OnPrOpen),
            "on_review_start" => Ok(Self::OnReviewStart),
            "on_release_prep" => Ok(Self::OnReleasePrep),
            "on_risky_change" => Ok(Self::OnRiskyChange),
            other => Err(format!("unsupported policy trigger `{other}`")),
        }
    }
}

/// The actions a policy can require from a host workflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    /// A memory recall step was performed.
    Recall,
    /// A durable writeback was stored.
    Writeback,
    /// A safety checkpoint was created.
    Checkpoint,
    /// An explicit skip reason was recorded.
    SkipReason,
    /// A scope choice was selected.
    ScopeSelected,
    /// A writeback classification was selected.
    ClassificationSelected,
}

impl PolicyAction {
    /// Returns the stable string representation used by config and CLI surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recall => "recall",
            Self::Writeback => "writeback",
            Self::Checkpoint => "checkpoint",
            Self::SkipReason => "skip_reason",
            Self::ScopeSelected => "scope_selected",
            Self::ClassificationSelected => "classification_selected",
        }
    }
}

impl FromStr for PolicyAction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "recall" => Ok(Self::Recall),
            "writeback" => Ok(Self::Writeback),
            "checkpoint" => Ok(Self::Checkpoint),
            "skip_reason" => Ok(Self::SkipReason),
            "scope_selected" => Ok(Self::ScopeSelected),
            "classification_selected" => Ok(Self::ClassificationSelected),
            other => Err(format!("unsupported policy action `{other}`")),
        }
    }
}

/// The behavior mode for a policy rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    /// Missing actions are recommendations only.
    Guided,
    /// Missing actions are required.
    Required,
    /// Missing actions are required unless an explicit skip reason exists.
    RequiredWithSkipReason,
}

impl PolicyMode {
    /// Returns the stable string representation used by config and CLI surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Guided => "guided",
            Self::Required => "required",
            Self::RequiredWithSkipReason => "required_with_skip_reason",
        }
    }
}

/// What the runner should do when a rule is unsatisfied.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyUnsatisfiedBehavior {
    /// Allow the workflow to continue.
    Allow,
    /// Continue, but surface a recommendation.
    AllowWithRecommendation,
    /// Return a required-action result.
    RequireAction,
    /// Block the workflow.
    Block,
}

/// Default scope selection guidance for a store.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ScopeStrategy {
    /// Prefer repository scope.
    #[default]
    Repo,
    /// Prefer workspace scope.
    Workspace,
    /// Prefer session scope.
    Session,
    /// Prefer task scope.
    Task,
}

impl ScopeStrategy {
    /// Returns the stable string representation used by config and CLI surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Workspace => "workspace",
            Self::Session => "session",
            Self::Task => "task",
        }
    }
}

/// How long policy evidence should be considered live.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTtl {
    /// Evidence applies only to the current task.
    #[default]
    Task,
    /// Evidence applies for the current session.
    Session,
    /// Evidence is kept until cleared manually.
    Manual,
}

/// Store-level defaults for policy evaluation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDefaults {
    /// Preferred scope strategy to recommend to hosts.
    #[serde(default)]
    pub scope_strategy: ScopeStrategy,
    /// Lifetime of evidence records.
    #[serde(default)]
    pub evidence_ttl: EvidenceTtl,
}

/// A declarative policy configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Config schema version.
    pub version: u16,
    /// Store-level defaults.
    #[serde(default)]
    pub defaults: PolicyDefaults,
    /// Ordered workflow rules.
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

impl PolicyConfig {
    /// Returns an empty policy config using the current version.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: 1,
            defaults: PolicyDefaults::default(),
            rules: Vec::new(),
        }
    }
}

/// A single policy rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Stable rule identifier.
    pub id: String,
    /// Trigger this rule applies to.
    pub trigger: PolicyTrigger,
    /// Enforcement mode for this rule.
    pub mode: PolicyMode,
    /// Actions required by this rule.
    #[serde(default)]
    pub requires: Vec<PolicyAction>,
    /// Whether skip is allowed when the rule is unsatisfied.
    #[serde(default)]
    pub allow_skip: bool,
    /// Conditions for the rule to apply.
    #[serde(default)]
    pub when: PolicyRuleCondition,
    /// Behavior when the rule is unsatisfied.
    pub on_unsatisfied: PolicyUnsatisfiedBehavior,
}

/// Match conditions for a policy rule.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyRuleCondition {
    /// Allowed host labels.
    #[serde(default)]
    pub host: Vec<String>,
    /// Allowed task kinds.
    #[serde(default)]
    pub task_kinds: Vec<String>,
    /// Paths that activate the rule.
    #[serde(default)]
    pub paths_any: Vec<String>,
    /// Paths ignored by the rule.
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

/// The workflow context evaluated against policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyContext {
    /// Trigger being evaluated.
    pub trigger: PolicyTrigger,
    /// Stable workflow key used for evidence correlation.
    pub workflow_key: Option<String>,
    /// Host type label, such as `coding-agent`.
    pub host: Option<String>,
    /// Optional task kind label.
    pub task_kind: Option<String>,
    /// Changed or relevant paths for the workflow.
    pub paths_changed: Vec<String>,
    /// Optional scope associated with the workflow.
    pub scope: Option<ScopeId>,
}

impl PolicyContext {
    /// Creates a context for the provided trigger.
    #[must_use]
    pub fn new(trigger: PolicyTrigger) -> Self {
        Self {
            trigger,
            workflow_key: None,
            host: None,
            task_kind: None,
            paths_changed: Vec::new(),
            scope: None,
        }
    }

    /// Associates a workflow key with the context.
    #[must_use]
    pub fn with_workflow_key(mut self, workflow_key: impl Into<String>) -> Self {
        self.workflow_key = Some(workflow_key.into());
        self
    }

    /// Associates a host label with the context.
    #[must_use]
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Associates a task kind with the context.
    #[must_use]
    pub fn with_task_kind(mut self, task_kind: impl Into<String>) -> Self {
        self.task_kind = Some(task_kind.into());
        self
    }

    /// Adds changed paths to the context.
    #[must_use]
    pub fn with_paths(
        mut self,
        paths_changed: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.paths_changed = paths_changed.into_iter().map(Into::into).collect();
        self
    }

    /// Associates a scope with the context.
    #[must_use]
    pub fn with_scope(mut self, scope: ScopeId) -> Self {
        self.scope = Some(scope);
        self
    }
}

/// Evidence collected for one workflow key.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyEvidence {
    /// Recorded actions that were completed.
    #[serde(default)]
    pub actions: BTreeSet<PolicyAction>,
    /// Optional explicit skip reason.
    #[serde(default)]
    pub skip_reason: Option<String>,
}

impl PolicyEvidence {
    /// Records a completed action.
    pub fn record_action(&mut self, action: PolicyAction) {
        self.actions.insert(action);
    }

    /// Records an explicit skip reason.
    pub fn record_skip_reason(&mut self, reason: impl Into<String>) {
        self.actions.insert(PolicyAction::SkipReason);
        self.skip_reason = Some(reason.into());
    }
}

/// Final policy decision returned to a host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Final decision outcome.
    pub kind: PolicyDecisionKind,
    /// Store-level scope guidance.
    pub scope_strategy: ScopeStrategy,
    /// Rules that matched the current context.
    pub matched_rules: Vec<PolicyRuleEvaluation>,
    /// Union of required actions across matched rules.
    pub required_actions: Vec<PolicyAction>,
    /// Remaining missing actions across matched rules.
    pub missing_actions: Vec<PolicyAction>,
    /// Human-readable evaluation reasons.
    pub reasons: Vec<String>,
}

/// Final severity of policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecisionKind {
    /// Workflow may continue.
    Allow,
    /// Workflow may continue, but guidance should be shown.
    AllowWithRecommendation,
    /// Host should require the missing action before continuing.
    RequireAction,
    /// Host should block the workflow.
    Block,
}

impl PolicyDecisionKind {
    /// Returns the stable string representation used by config and CLI surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowWithRecommendation => "allow_with_recommendation",
            Self::RequireAction => "require_action",
            Self::Block => "block",
        }
    }
}

/// Evaluation details for one matched rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyRuleEvaluation {
    /// Stable rule identifier.
    pub id: String,
    /// Rule mode.
    pub mode: PolicyMode,
    /// Required actions for this rule.
    pub required_actions: Vec<PolicyAction>,
    /// Missing actions for this rule.
    pub missing_actions: Vec<PolicyAction>,
    /// Whether the rule is satisfied.
    pub satisfied: bool,
    /// Whether skip was used to satisfy the rule.
    pub skipped_via_reason: bool,
    /// Reasons from this rule evaluation.
    pub reasons: Vec<String>,
}

/// Evaluate a policy config against the provided context and evidence.
#[must_use]
pub fn evaluate_policy(
    config: &PolicyConfig,
    context: &PolicyContext,
    evidence: Option<&PolicyEvidence>,
) -> PolicyDecision {
    let mut decision = PolicyDecision {
        kind: PolicyDecisionKind::Allow,
        scope_strategy: config.defaults.scope_strategy,
        matched_rules: Vec::new(),
        required_actions: Vec::new(),
        missing_actions: Vec::new(),
        reasons: Vec::new(),
    };

    let Some(evidence) = evidence else {
        return evaluate_policy(config, context, Some(&PolicyEvidence::default()));
    };

    let mut required_actions = BTreeSet::new();
    let mut missing_actions = BTreeSet::new();

    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.trigger == context.trigger && rule_matches(rule, context))
    {
        let missing = rule
            .requires
            .iter()
            .copied()
            .filter(|action| !evidence.actions.contains(action))
            .collect::<Vec<_>>();
        let skipped_via_reason = rule.allow_skip
            && matches!(rule.mode, PolicyMode::RequiredWithSkipReason)
            && evidence
                .skip_reason
                .as_ref()
                .is_some_and(|reason| !reason.trim().is_empty());
        let satisfied = missing.is_empty() || skipped_via_reason;
        let evaluation = PolicyRuleEvaluation {
            id: rule.id.clone(),
            mode: rule.mode,
            required_actions: rule.requires.clone(),
            missing_actions: if satisfied {
                Vec::new()
            } else {
                missing.clone()
            },
            satisfied,
            skipped_via_reason,
            reasons: rule_reasons(rule, &missing, skipped_via_reason),
        };

        required_actions.extend(rule.requires.iter().copied());
        if !satisfied {
            missing_actions.extend(missing.iter().copied());
            decision.kind = decision.kind.max(rule_outcome(rule));
        } else if matches!(rule.mode, PolicyMode::Guided) && !missing.is_empty() {
            decision.kind = decision
                .kind
                .max(PolicyDecisionKind::AllowWithRecommendation);
        }

        decision.reasons.extend(evaluation.reasons.iter().cloned());
        decision.matched_rules.push(evaluation);
    }

    if decision.matched_rules.is_empty() {
        decision
            .reasons
            .push("No policy rules matched this workflow context.".to_owned());
    }

    decision.required_actions = required_actions.into_iter().collect();
    decision.missing_actions = missing_actions.into_iter().collect();
    decision
}

fn rule_outcome(rule: &PolicyRule) -> PolicyDecisionKind {
    match rule.mode {
        PolicyMode::Guided => PolicyDecisionKind::AllowWithRecommendation,
        PolicyMode::Required | PolicyMode::RequiredWithSkipReason => match rule.on_unsatisfied {
            PolicyUnsatisfiedBehavior::Allow => PolicyDecisionKind::Allow,
            PolicyUnsatisfiedBehavior::AllowWithRecommendation => {
                PolicyDecisionKind::AllowWithRecommendation
            }
            PolicyUnsatisfiedBehavior::RequireAction => PolicyDecisionKind::RequireAction,
            PolicyUnsatisfiedBehavior::Block => PolicyDecisionKind::Block,
        },
    }
}

fn rule_reasons(
    rule: &PolicyRule,
    missing_actions: &[PolicyAction],
    skipped_via_reason: bool,
) -> Vec<String> {
    if skipped_via_reason {
        return vec![format!(
            "Rule `{}` was satisfied by an explicit skip reason.",
            rule.id
        )];
    }

    if missing_actions.is_empty() {
        return vec![format!("Rule `{}` is already satisfied.", rule.id)];
    }

    vec![format!(
        "Rule `{}` is missing: {}.",
        rule.id,
        missing_actions
            .iter()
            .map(|action| action.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )]
}

fn rule_matches(rule: &PolicyRule, context: &PolicyContext) -> bool {
    let when = &rule.when;

    if !when.host.is_empty()
        && !when
            .host
            .iter()
            .any(|candidate| context.host.as_deref() == Some(candidate.as_str()))
    {
        return false;
    }

    if !when.task_kinds.is_empty()
        && !when
            .task_kinds
            .iter()
            .any(|candidate| context.task_kind.as_deref() == Some(candidate.as_str()))
    {
        return false;
    }

    let included_paths = context
        .paths_changed
        .iter()
        .filter(|path| {
            !when
                .exclude_paths
                .iter()
                .any(|pattern| path_matches(pattern, path))
        })
        .collect::<Vec<_>>();

    if !when.paths_any.is_empty()
        && !included_paths.iter().any(|path| {
            when.paths_any
                .iter()
                .any(|pattern| path_matches(pattern, path))
        })
    {
        return false;
    }

    if when.paths_any.is_empty()
        && !when.exclude_paths.is_empty()
        && !context.paths_changed.is_empty()
    {
        return !included_paths.is_empty();
    }

    true
}

fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    path == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(rules: Vec<PolicyRule>) -> PolicyConfig {
        PolicyConfig {
            version: 1,
            defaults: PolicyDefaults::default(),
            rules,
        }
    }

    fn commit_rule(mode: PolicyMode) -> PolicyRule {
        PolicyRule {
            id: "commit-writeback".to_owned(),
            trigger: PolicyTrigger::OnGitCommit,
            mode,
            requires: vec![PolicyAction::Writeback],
            allow_skip: matches!(mode, PolicyMode::RequiredWithSkipReason),
            when: PolicyRuleCondition {
                host: vec!["coding-agent".to_owned()],
                task_kinds: Vec::new(),
                paths_any: vec!["adapters/**".to_owned()],
                exclude_paths: vec!["docs/**".to_owned()],
            },
            on_unsatisfied: PolicyUnsatisfiedBehavior::Block,
        }
    }

    #[test]
    fn guided_rule_allows_with_recommendation_when_unsatisfied() {
        let config = config(vec![commit_rule(PolicyMode::Guided)]);
        let context = PolicyContext::new(PolicyTrigger::OnGitCommit)
            .with_host("coding-agent")
            .with_paths(["adapters/coding_agent_adapter.py"]);

        let decision = evaluate_policy(&config, &context, None);

        assert_eq!(decision.kind, PolicyDecisionKind::AllowWithRecommendation);
        assert_eq!(decision.missing_actions, vec![PolicyAction::Writeback]);
    }

    #[test]
    fn required_with_skip_reason_allows_when_skip_reason_exists() {
        let config = config(vec![commit_rule(PolicyMode::RequiredWithSkipReason)]);
        let context = PolicyContext::new(PolicyTrigger::OnGitCommit)
            .with_host("coding-agent")
            .with_paths(["adapters/coding_agent_adapter.py"]);
        let mut evidence = PolicyEvidence::default();
        evidence.record_skip_reason("docs-only change");

        let decision = evaluate_policy(&config, &context, Some(&evidence));

        assert_eq!(decision.kind, PolicyDecisionKind::Allow);
        assert!(decision.missing_actions.is_empty());
        assert!(decision.matched_rules[0].skipped_via_reason);
    }

    #[test]
    fn excluded_paths_do_not_match_rule() {
        let config = config(vec![commit_rule(PolicyMode::Required)]);
        let context = PolicyContext::new(PolicyTrigger::OnGitCommit)
            .with_host("coding-agent")
            .with_paths(["docs/policy-runner-design.md"]);

        let decision = evaluate_policy(&config, &context, None);

        assert_eq!(decision.kind, PolicyDecisionKind::Allow);
        assert!(decision.matched_rules.is_empty());
    }

    #[test]
    fn recorded_action_satisfies_required_rule() {
        let config = config(vec![commit_rule(PolicyMode::Required)]);
        let context = PolicyContext::new(PolicyTrigger::OnGitCommit)
            .with_host("coding-agent")
            .with_paths(["adapters/coding_agent_adapter.py"]);
        let mut evidence = PolicyEvidence::default();
        evidence.record_action(PolicyAction::Writeback);

        let decision = evaluate_policy(&config, &context, Some(&evidence));

        assert_eq!(decision.kind, PolicyDecisionKind::Allow);
        assert!(decision.missing_actions.is_empty());
        assert!(decision.matched_rules[0].satisfied);
    }
}
