use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::errors::CliError;

const PROVIDER_CONFIG_VERSION: u16 = 1;
const PROVIDER_CONFIG_FILENAME: &str = "providers.toml";
const PROVIDER_CONFIG_DIRNAME: &str = "mnemix";
const CONFIG_HOME_OVERRIDE_ENV: &str = "MNEMIX_CONFIG_HOME";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProviderProfilesState {
    pub(crate) path: PathBuf,
    pub(crate) profiles: BTreeMap<String, ProviderProfile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ProviderProfilesFile {
    version: u16,
    #[serde(default)]
    profiles: BTreeMap<String, ProviderProfile>,
}

impl Default for ProviderProfilesFile {
    fn default() -> Self {
        Self {
            version: PROVIDER_CONFIG_VERSION,
            profiles: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ProviderProfile {
    Cloud(CloudProviderProfile),
    Local(LocalProviderProfile),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct CloudProviderProfile {
    pub(crate) model: String,
    pub(crate) base_url: String,
    pub(crate) api_key_env: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalProviderProfile {
    pub(crate) model: String,
    pub(crate) endpoint: String,
    #[serde(default)]
    pub(crate) auth_token_env: Option<String>,
}

impl ProviderProfile {
    pub(crate) const fn kind_name(&self) -> &'static str {
        match self {
            Self::Cloud(_) => "cloud",
            Self::Local(_) => "local",
        }
    }

    pub(crate) fn model(&self) -> &str {
        match self {
            Self::Cloud(profile) => &profile.model,
            Self::Local(profile) => &profile.model,
        }
    }

    pub(crate) fn endpoint(&self) -> &str {
        match self {
            Self::Cloud(profile) => &profile.base_url,
            Self::Local(profile) => &profile.endpoint,
        }
    }

    pub(crate) fn api_key_source(&self) -> Option<String> {
        match self {
            Self::Cloud(profile) => Some(format!("env:{}", profile.api_key_env)),
            Self::Local(_) => None,
        }
    }

    pub(crate) fn auth_token_source(&self) -> Option<String> {
        match self {
            Self::Cloud(_) => None,
            Self::Local(profile) => profile
                .auth_token_env
                .as_ref()
                .map(|value| format!("env:{value}")),
        }
    }
}

pub(crate) fn load_provider_profiles() -> Result<ProviderProfilesState, CliError> {
    let path = provider_config_path()?;
    if !path.exists() {
        return Ok(ProviderProfilesState {
            path,
            profiles: BTreeMap::new(),
        });
    }

    let contents = fs::read_to_string(&path)?;
    let config: ProviderProfilesFile = toml::from_str(&contents)
        .map_err(|error| CliError::ProviderConfigParse(error.to_string()))?;
    if config.version != PROVIDER_CONFIG_VERSION {
        return Err(CliError::UnsupportedProviderConfigVersion {
            actual: config.version,
            expected: PROVIDER_CONFIG_VERSION,
        });
    }

    Ok(ProviderProfilesState {
        path,
        profiles: config.profiles,
    })
}

pub(crate) fn save_provider_profiles(
    path: &Path,
    profiles: &BTreeMap<String, ProviderProfile>,
) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = toml::to_string_pretty(&ProviderProfilesFile {
        version: PROVIDER_CONFIG_VERSION,
        profiles: profiles.clone(),
    })
    .map_err(|error| CliError::ProviderConfigSerialize(error.to_string()))?;
    fs::write(path, payload)?;
    Ok(())
}

fn provider_config_path() -> Result<PathBuf, CliError> {
    Ok(provider_config_dir()?.join(PROVIDER_CONFIG_FILENAME))
}

fn provider_config_dir() -> Result<PathBuf, CliError> {
    if let Some(override_root) = env::var_os(CONFIG_HOME_OVERRIDE_ENV) {
        let root = PathBuf::from(override_root);
        if !root.as_os_str().is_empty() {
            return Ok(root.join(PROVIDER_CONFIG_DIRNAME));
        }
    }

    if let Some(xdg_root) = env::var_os("XDG_CONFIG_HOME") {
        let root = PathBuf::from(xdg_root);
        if !root.as_os_str().is_empty() {
            return Ok(root.join(PROVIDER_CONFIG_DIRNAME));
        }
    }

    #[cfg(target_os = "windows")]
    if let Some(app_data) = env::var_os("APPDATA") {
        let root = PathBuf::from(app_data);
        if !root.as_os_str().is_empty() {
            return Ok(root.join(PROVIDER_CONFIG_DIRNAME));
        }
    }

    if let Some(home) = env::var_os("HOME") {
        let root = PathBuf::from(home);
        if !root.as_os_str().is_empty() {
            return Ok(root.join(".config").join(PROVIDER_CONFIG_DIRNAME));
        }
    }

    Err(CliError::ProviderConfigHomeUnavailable)
}
