use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use mnemix_core::{
    PolicyAction, PolicyConfig, PolicyContext, PolicyDecision, PolicyEvidence, evaluate_policy,
};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{PolicyArgs, PolicyCheckArgs, PolicyCommand, PolicyRecordArgs},
    cmd::policy_result,
    errors::CliError,
    output::CommandOutput,
};

const POLICY_CONFIG_VERSION: u16 = 1;
const POLICY_CONFIG_FILENAME: &str = "policy.toml";
const POLICY_STATE_FILENAME: &str = "policy-state.json";

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct PolicyStateFile {
    #[serde(default)]
    workflows: BTreeMap<String, PolicyEvidence>,
}

pub(crate) fn run(store_path: &Path, args: &PolicyArgs) -> Result<CommandOutput, CliError> {
    match &args.command {
        PolicyCommand::Check(args) => check(store_path, args, "check"),
        PolicyCommand::Explain(args) => check(store_path, args, "explain"),
        PolicyCommand::Record(args) => record(store_path, args),
    }
}

fn check(
    store_path: &Path,
    args: &PolicyCheckArgs,
    action: &'static str,
) -> Result<CommandOutput, CliError> {
    let config = load_policy_config(store_path)?;
    let context = build_context(args);
    let evidence = load_policy_state(store_path)?
        .workflows
        .get(args.workflow_key.as_deref().unwrap_or_default())
        .cloned();
    let decision = evaluate_policy(&config, &context, evidence.as_ref());

    Ok(policy_result(
        action,
        args.trigger.as_str().to_owned(),
        args.workflow_key.clone(),
        &with_missing_config_reason(&config, decision),
    ))
}

fn record(store_path: &Path, args: &PolicyRecordArgs) -> Result<CommandOutput, CliError> {
    fs::create_dir_all(store_path)?;
    let mut state = load_policy_state(store_path)?;
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
            entry.record_skip_reason(reason.clone());
        }
        action => {
            entry.record_action(action);
        }
    }

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
    serde_json::from_str(&contents).map_err(|error| CliError::PolicyStateParse(error.to_string()))
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
