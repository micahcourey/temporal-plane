use std::path::Path;

use crate::{
    cli::{
        ProviderRemoveArgs, ProviderSetCloudArgs, ProviderSetLocalArgs, ProviderShowArgs,
        ProvidersArgs, ProvidersCommand,
    },
    cmd::{
        provider_profile_list_result, provider_profile_result, provider_store_compatibility,
        status_result,
    },
    config::{
        CloudProviderProfile, LocalProviderProfile, ProviderProfile, load_provider_profiles,
        save_provider_profiles,
    },
    errors::CliError,
    output::{CommandOutput, ProviderProfileView},
    providers_runtime::resolve_named_provider,
};

pub(crate) fn run(store_path: &Path, args: &ProvidersArgs) -> Result<CommandOutput, CliError> {
    match &args.command {
        ProvidersCommand::List => list_profiles(),
        ProvidersCommand::Show(args) => show_profile(args),
        ProvidersCommand::Validate(args) => validate_profile(store_path, args),
        ProvidersCommand::SetCloud(args) => set_cloud_profile(args),
        ProvidersCommand::SetLocal(args) => set_local_profile(args),
        ProvidersCommand::Remove(args) => remove_profile(args),
    }
}

fn list_profiles() -> Result<CommandOutput, CliError> {
    let state = load_provider_profiles()?;
    let profiles = state
        .profiles
        .iter()
        .map(|(name, profile)| provider_profile_view(name, profile))
        .collect();

    Ok(provider_profile_list_result(
        "providers list",
        state.path.display().to_string(),
        profiles,
    ))
}

fn show_profile(args: &ProviderShowArgs) -> Result<CommandOutput, CliError> {
    let state = load_provider_profiles()?;
    let profile =
        state
            .profiles
            .get(&args.name)
            .ok_or_else(|| CliError::ProviderProfileNotFound {
                name: args.name.clone(),
            })?;

    Ok(provider_profile_result(
        "providers show",
        "show",
        state.path.display().to_string(),
        provider_profile_view(&args.name, profile),
    ))
}

fn validate_profile(store_path: &Path, args: &ProviderShowArgs) -> Result<CommandOutput, CliError> {
    let state = load_provider_profiles()?;
    if !state.profiles.contains_key(&args.name) {
        return Err(CliError::ProviderProfileNotFound {
            name: args.name.clone(),
        });
    }
    let provider = resolve_named_provider(&args.name)?;
    let compatibility = provider_store_compatibility(store_path, &provider)?;
    let status = if compatibility.is_compatible() {
        "ok"
    } else {
        match compatibility.label() {
            "model_mismatch" | "dimension_mismatch" => "mismatch",
            _ => "ok",
        }
    };

    Ok(status_result(
        "providers validate",
        status,
        format!(
            "validated provider profile `{}` with model={} dimensions={} store_compatibility={} compatibility_detail={}",
            args.name,
            provider.model_id,
            provider.dimensions,
            compatibility.label(),
            compatibility.detail(),
        ),
        Some(state.path.display().to_string()),
        None,
    ))
}

fn set_cloud_profile(args: &ProviderSetCloudArgs) -> Result<CommandOutput, CliError> {
    let mut state = load_provider_profiles()?;
    state.profiles.insert(
        args.name.clone(),
        ProviderProfile::Cloud(CloudProviderProfile {
            model: args.model.clone(),
            base_url: args.base_url.clone(),
            api_key_env: args.api_key_env.clone(),
        }),
    );
    save_provider_profiles(&state.path, &state.profiles)?;

    Ok(status_result(
        "providers set-cloud",
        "ok",
        format!(
            "saved cloud provider profile `{}` with model={} endpoint={} api_key_source=env:{}",
            args.name, args.model, args.base_url, args.api_key_env
        ),
        Some(state.path.display().to_string()),
        None,
    ))
}

fn set_local_profile(args: &ProviderSetLocalArgs) -> Result<CommandOutput, CliError> {
    let mut state = load_provider_profiles()?;
    state.profiles.insert(
        args.name.clone(),
        ProviderProfile::Local(LocalProviderProfile {
            model: args.model.clone(),
            endpoint: args.endpoint.clone(),
            auth_token_env: args.auth_token_env.clone(),
        }),
    );
    save_provider_profiles(&state.path, &state.profiles)?;

    let auth_token_source = args
        .auth_token_env
        .as_ref()
        .map_or_else(|| "<none>".to_owned(), |value| format!("env:{value}"));
    Ok(status_result(
        "providers set-local",
        "ok",
        format!(
            "saved local provider profile `{}` with model={} endpoint={} auth_token_source={}",
            args.name, args.model, args.endpoint, auth_token_source
        ),
        Some(state.path.display().to_string()),
        None,
    ))
}

fn remove_profile(args: &ProviderRemoveArgs) -> Result<CommandOutput, CliError> {
    let mut state = load_provider_profiles()?;
    if state.profiles.remove(&args.name).is_none() {
        return Err(CliError::ProviderProfileNotFound {
            name: args.name.clone(),
        });
    }
    save_provider_profiles(&state.path, &state.profiles)?;

    Ok(status_result(
        "providers remove",
        "ok",
        format!("removed provider profile `{}`", args.name),
        Some(state.path.display().to_string()),
        None,
    ))
}

fn provider_profile_view(name: &str, profile: &ProviderProfile) -> ProviderProfileView {
    ProviderProfileView {
        name: name.to_owned(),
        kind: profile.kind_name(),
        model: profile.model().to_owned(),
        endpoint: profile.endpoint().to_owned(),
        api_key_source: profile.api_key_source(),
        auth_token_source: profile.auth_token_source(),
    }
}
