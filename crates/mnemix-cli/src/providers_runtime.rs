use std::{env, sync::Arc, time::Duration};

use reqwest::blocking::Client;
use serde::Deserialize;

use mnemix_lancedb::{EmbeddingProvider, EmbeddingProviderError};

use crate::{
    config::{ProviderProfile, load_provider_profiles},
    errors::CliError,
};

const VALIDATION_PROBE_TEXT: &str = "mnemix provider validation probe";

#[derive(Clone)]
pub(crate) struct ResolvedProvider {
    pub(crate) model_id: String,
    pub(crate) dimensions: u32,
    pub(crate) provider: Arc<dyn EmbeddingProvider>,
}

#[derive(Clone)]
struct OpenAiCompatibleEmbeddingProvider {
    model_id: String,
    endpoint: String,
    auth_token: Option<String>,
    dimensions: u32,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    model: Option<String>,
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl EmbeddingProvider for OpenAiCompatibleEmbeddingProvider {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingProviderError> {
        request_embedding(
            &self.client,
            &self.endpoint,
            &self.model_id,
            self.auth_token.as_deref(),
            text,
        )
        .map_err(|message| EmbeddingProviderError::Message { message })
    }
}

pub(crate) fn resolve_named_provider(name: &str) -> Result<ResolvedProvider, CliError> {
    let state = load_provider_profiles()?;
    let profile =
        state
            .profiles
            .get(name)
            .cloned()
            .ok_or_else(|| CliError::ProviderProfileNotFound {
                name: name.to_owned(),
            })?;
    build_provider(name, &profile)
}

fn build_provider(name: &str, profile: &ProviderProfile) -> Result<ResolvedProvider, CliError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| CliError::ProviderRuntime {
            name: name.to_owned(),
            details: error.to_string(),
        })?;

    let (model, endpoint, auth_token) = match profile {
        ProviderProfile::Cloud(profile) => (
            profile.model.clone(),
            normalize_endpoint(&profile.base_url),
            Some(resolve_env_secret(name, &profile.api_key_env)?),
        ),
        ProviderProfile::Local(profile) => (
            profile.model.clone(),
            normalize_endpoint(&profile.endpoint),
            profile
                .auth_token_env
                .as_ref()
                .map(|env_name| resolve_env_secret(name, env_name))
                .transpose()?,
        ),
    };

    let probe_embedding = request_embedding(
        &client,
        &endpoint,
        &model,
        auth_token.as_deref(),
        VALIDATION_PROBE_TEXT,
    )
    .map_err(|details| CliError::ProviderRuntime {
        name: name.to_owned(),
        details,
    })?;
    let dimensions =
        u32::try_from(probe_embedding.len()).map_err(|_| CliError::ProviderRuntime {
            name: name.to_owned(),
            details: "embedding dimensions exceed supported range".to_owned(),
        })?;
    if dimensions == 0 {
        return Err(CliError::ProviderRuntime {
            name: name.to_owned(),
            details: "provider returned an empty embedding".to_owned(),
        });
    }

    Ok(ResolvedProvider {
        model_id: model.clone(),
        dimensions,
        provider: Arc::new(OpenAiCompatibleEmbeddingProvider {
            model_id: model,
            endpoint,
            auth_token,
            dimensions,
            client,
        }),
    })
}

fn resolve_env_secret(profile_name: &str, env_name: &str) -> Result<String, CliError> {
    env::var(env_name).map_err(|_| CliError::ProviderSecretMissing {
        name: profile_name.to_owned(),
        env: env_name.to_owned(),
    })
}

fn normalize_endpoint(value: &str) -> String {
    value.trim_end_matches('/').to_owned()
}

fn request_embedding(
    client: &Client,
    endpoint: &str,
    model: &str,
    auth_token: Option<&str>,
    text: &str,
) -> Result<Vec<f32>, String> {
    let url = format!("{endpoint}/embeddings");
    let mut request = client.post(&url).json(&serde_json::json!({
        "model": model,
        "input": text,
    }));
    if let Some(token) = auth_token {
        request = request.bearer_auth(token);
    }

    let response = request.send().map_err(|error| error.to_string())?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        let body_suffix = if body.trim().is_empty() {
            String::new()
        } else {
            format!(": {}", body.trim())
        };
        return Err(format!(
            "embedding request failed with status {status}{body_suffix}"
        ));
    }

    let payload: EmbeddingResponse = response.json().map_err(|error| error.to_string())?;
    let Some(first) = payload.data.into_iter().next() else {
        return Err("embedding response did not include any vectors".to_owned());
    };
    if first.embedding.is_empty() {
        return Err("embedding response contained an empty vector".to_owned());
    }

    let response_model = payload.model.unwrap_or_else(|| model.to_owned());
    if response_model != model {
        return Err(format!(
            "embedding response model `{response_model}` did not match configured model `{model}`"
        ));
    }

    Ok(first.embedding)
}
