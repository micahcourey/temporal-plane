use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use humantime::parse_duration;
use mnemix_core::{
    EvidenceTtl, PolicyAction, PolicyConfig, PolicyContext, PolicyDecision, PolicyEvidence,
    evaluate_policy,
};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{
        PolicyArgs, PolicyCheckArgs, PolicyCleanupArgs, PolicyClearArgs, PolicyCommand,
        PolicyRecordArgs,
    },
    cmd::policy_result,
    errors::CliError,
    output::CommandOutput,
};

const POLICY_CONFIG_VERSION: u16 = 1;
const POLICY_CONFIG_FILENAME: &str = "policy.toml";
const POLICY_STATE_FILENAME: &str = "policy-state.json";
const TASK_EVIDENCE_MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
const SESSION_EVIDENCE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct PolicyStateFile {
    #[serde(default)]
    workflows: BTreeMap<String, PolicyStateEntry>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct PolicyStateEntry {
    #[serde(default)]
    evidence: PolicyEvidence,
    #[serde(default)]
    created_at_unix: Option<u64>,
    #[serde(default)]
    updated_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
enum PolicyStateEntryCompat {
    Current(PolicyStateEntry),
    Legacy(PolicyEvidence),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
struct PolicyStateFileCompat {
    #[serde(default)]
    workflows: BTreeMap<String, PolicyStateEntryCompat>,
}

impl From<PolicyStateEntryCompat> for PolicyStateEntry {
    fn from(value: PolicyStateEntryCompat) -> Self {
        match value {
            PolicyStateEntryCompat::Current(entry) => entry,
            PolicyStateEntryCompat::Legacy(evidence) => Self {
                evidence,
                created_at_unix: None,
                updated_at_unix: None,
            },
        }
    }
}

impl PolicyStateEntry {
    fn touch(&mut self, now_unix: u64) {
        if self.created_at_unix.is_none() {
            self.created_at_unix = Some(now_unix);
        }
        self.updated_at_unix = Some(now_unix);
    }

    fn is_empty(&self) -> bool {
        self.evidence.actions.is_empty()
            && self
                .evidence
                .skip_reason
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
    }

    fn last_updated_unix(&self) -> Option<u64> {
        self.updated_at_unix.or(self.created_at_unix)
    }
}

pub(crate) fn run(store_path: &Path, args: &PolicyArgs) -> Result<CommandOutput, CliError> {
    match &args.command {
        PolicyCommand::Check(args) => check(store_path, args, "check"),
        PolicyCommand::Explain(args) => check(store_path, args, "explain"),
        PolicyCommand::Record(args) => record(store_path, args),
        PolicyCommand::Clear(args) => clear(store_path, args),
        PolicyCommand::Cleanup(args) => cleanup(store_path, args),
    }
}

fn check(
    store_path: &Path,
    args: &PolicyCheckArgs,
    action: &'static str,
) -> Result<CommandOutput, CliError> {
    let config = load_policy_config(store_path)?;
    let context = build_context(args);
    let state = load_policy_state(store_path)?;
    let (evidence, lifecycle_reason) = args
        .workflow_key
        .as_ref()
        .and_then(|key| state.workflows.get(key))
        .map_or((None, None), |entry| {
            resolve_evidence(entry, config.defaults.evidence_ttl)
        });
    let decision = evaluate_policy(&config, &context, evidence.as_ref());
    let mut decision = with_missing_config_reason(&config, decision);
    if let Some(reason) = lifecycle_reason {
        decision.reasons.insert(0, reason);
    }

    Ok(policy_result(
        action,
        args.trigger.as_str().to_owned(),
        args.workflow_key.clone(),
        &decision,
    ))
}

fn record(store_path: &Path, args: &PolicyRecordArgs) -> Result<CommandOutput, CliError> {
    fs::create_dir_all(store_path)?;
    let mut state = load_policy_state(store_path)?;
    let now_unix = current_unix_timestamp();
    let entry = state
        .workflows
        .entry(args.workflow_key.clone())
        .or_default();

    match args.action {
        PolicyAction::SkipReason => {
            let reason = args
                .reason
                .as_ref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    CliError::PolicyStateParse(
                        "policy record --action skip_reason requires --reason".to_owned(),
                    )
                })?;
            entry.evidence.record_skip_reason(reason.clone());
        }
        action => {
            entry.evidence.record_action(action);
        }
    }
    entry.touch(now_unix);

    save_policy_state(store_path, &state)?;
    Ok(super::status_result(
        "policy",
        "recorded",
        format!(
            "Recorded `{}` for workflow `{}`",
            args.action.as_str(),
            args.workflow_key
        ),
        Some(store_path.display().to_string()),
        None,
    ))
}

fn clear(store_path: &Path, args: &PolicyClearArgs) -> Result<CommandOutput, CliError> {
    fs::create_dir_all(store_path)?;
    let mut state = load_policy_state(store_path)?;
    let Some(mut entry) = state.workflows.remove(&args.workflow_key) else {
        return Ok(super::status_result(
            "policy",
            "unchanged",
            format!(
                "No policy evidence exists for workflow `{}`",
                args.workflow_key
            ),
            Some(store_path.display().to_string()),
            None,
        ));
    };

    let message = if let Some(action) = args.action {
        clear_action(&mut entry.evidence, action);
        if entry.is_empty() {
            format!(
                "Cleared `{}` and removed now-empty workflow `{}`",
                action.as_str(),
                args.workflow_key
            )
        } else {
            entry.touch(current_unix_timestamp());
            state.workflows.insert(args.workflow_key.clone(), entry);
            format!(
                "Cleared `{}` for workflow `{}`",
                action.as_str(),
                args.workflow_key
            )
        }
    } else {
        format!(
            "Cleared all policy evidence for workflow `{}`",
            args.workflow_key
        )
    };

    save_policy_state(store_path, &state)?;
    Ok(super::status_result(
        "policy",
        "cleared",
        message,
        Some(store_path.display().to_string()),
        None,
    ))
}

fn cleanup(store_path: &Path, args: &PolicyCleanupArgs) -> Result<CommandOutput, CliError> {
    let config = load_policy_config(store_path)?;
    let ttl = args.ttl.unwrap_or(config.defaults.evidence_ttl);
    let older_than = cleanup_window(ttl, args.older_than.as_deref())?;
    let now_unix = current_unix_timestamp();

    let mut state = load_policy_state(store_path)?;
    let before = state.workflows.len();
    state.workflows.retain(|_, entry| {
        if entry.is_empty() {
            return false;
        }
        !is_expired(entry, ttl, older_than, now_unix)
    });
    let removed = before.saturating_sub(state.workflows.len());

    if !args.dry_run {
        fs::create_dir_all(store_path)?;
        save_policy_state(store_path, &state)?;
    }

    let lifecycle = if let Some(window) = older_than {
        format!(
            "using `{}` TTL and `{}` age threshold",
            ttl.as_str(),
            humantime::format_duration(window)
        )
    } else {
        format!("using `{}` TTL with empty-entry cleanup only", ttl.as_str())
    };

    Ok(super::status_result(
        "policy",
        if args.dry_run { "dry_run" } else { "cleaned" },
        format!("Removed {removed} workflow evidence entries {lifecycle}"),
        Some(store_path.display().to_string()),
        None,
    ))
}

fn build_context(args: &PolicyCheckArgs) -> PolicyContext {
    let context = PolicyContext::new(args.trigger);
    let context = if let Some(workflow_key) = &args.workflow_key {
        context.with_workflow_key(workflow_key.clone())
    } else {
        context
    };
    let context = if let Some(host) = &args.host {
        context.with_host(host.clone())
    } else {
        context
    };
    let context = if let Some(task_kind) = &args.task_kind {
        context.with_task_kind(task_kind.clone())
    } else {
        context
    };
    let context = if let Some(scope) = &args.scope {
        context.with_scope(scope.clone())
    } else {
        context
    };
    context.with_paths(args.path.iter().cloned())
}

fn with_missing_config_reason(
    config: &PolicyConfig,
    mut decision: PolicyDecision,
) -> PolicyDecision {
    if config.rules.is_empty() {
        decision.reasons.insert(
            0,
            format!(
                "No `{POLICY_CONFIG_FILENAME}` file was found or it did not contain any rules; defaulting to allow."
            ),
        );
    }
    decision
}

fn load_policy_config(store_path: &Path) -> Result<PolicyConfig, CliError> {
    let path = policy_config_path(store_path);
    if !path.exists() {
        return Ok(PolicyConfig::empty());
    }

    let contents = fs::read_to_string(path)?;
    let config: PolicyConfig = toml::from_str(&contents)
        .map_err(|error| CliError::PolicyConfigParse(error.to_string()))?;
    if config.version != POLICY_CONFIG_VERSION {
        return Err(CliError::UnsupportedPolicyConfigVersion {
            actual: config.version,
            expected: POLICY_CONFIG_VERSION,
        });
    }
    Ok(config)
}

fn load_policy_state(store_path: &Path) -> Result<PolicyStateFile, CliError> {
    let path = policy_state_path(store_path);
    if !path.exists() {
        return Ok(PolicyStateFile::default());
    }

    let contents = fs::read_to_string(path)?;
    let compat: PolicyStateFileCompat = serde_json::from_str(&contents)
        .map_err(|error| CliError::PolicyStateParse(error.to_string()))?;
    Ok(PolicyStateFile {
        workflows: compat
            .workflows
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect(),
    })
}

fn save_policy_state(store_path: &Path, state: &PolicyStateFile) -> Result<(), CliError> {
    let path = policy_state_path(store_path);
    let payload = serde_json::to_string_pretty(state)?;
    fs::write(path, payload)?;
    Ok(())
}

fn policy_config_path(store_path: &Path) -> PathBuf {
    store_path.join(POLICY_CONFIG_FILENAME)
}

fn policy_state_path(store_path: &Path) -> PathBuf {
    store_path.join(POLICY_STATE_FILENAME)
}

fn clear_action(evidence: &mut PolicyEvidence, action: PolicyAction) {
    evidence.actions.remove(&action);
    if matches!(action, PolicyAction::SkipReason) {
        evidence.skip_reason = None;
    }
}

fn resolve_evidence(
    entry: &PolicyStateEntry,
    ttl: EvidenceTtl,
) -> (Option<PolicyEvidence>, Option<String>) {
    if is_expired(entry, ttl, ttl_max_age(ttl), current_unix_timestamp()) {
        return (
            None,
            Some(format!(
                "Stored policy evidence expired under `{}` TTL and was ignored for this decision.",
                ttl.as_str()
            )),
        );
    }

    (Some(entry.evidence.clone()), None)
}

fn cleanup_window(
    ttl: EvidenceTtl,
    older_than: Option<&str>,
) -> Result<Option<Duration>, CliError> {
    match older_than {
        Some(value) => parse_duration(value)
            .map(Some)
            .map_err(|error| CliError::PolicyStateParse(error.to_string())),
        None => Ok(ttl_max_age(ttl)),
    }
}

fn ttl_max_age(ttl: EvidenceTtl) -> Option<Duration> {
    match ttl {
        EvidenceTtl::Task => Some(TASK_EVIDENCE_MAX_AGE),
        EvidenceTtl::Session => Some(SESSION_EVIDENCE_MAX_AGE),
        EvidenceTtl::Manual => None,
    }
}

fn is_expired(
    entry: &PolicyStateEntry,
    ttl: EvidenceTtl,
    max_age: Option<Duration>,
    now_unix: u64,
) -> bool {
    if matches!(ttl, EvidenceTtl::Manual) {
        return false;
    }

    let Some(max_age) = max_age else {
        return false;
    };
    let Some(last_updated_unix) = entry.last_updated_unix() else {
        return false;
    };
    let age = now_unix.saturating_sub(last_updated_unix);
    age >= max_age.as_secs()
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
