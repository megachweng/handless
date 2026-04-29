use crate::settings::AppSettings;
use log::debug;
use serde::{Deserialize, Serialize};
use specta::Type;

pub(crate) const ATLANTIS_PROVIDER_ID: &str = "atlantis";
pub(crate) const ATLANTIS_BASE_URL: &str = "https://atlantis.azureai-np.swissre.com";
pub(crate) const ATLANTIS_API_VERSION: &str = "2024-12-01-preview";
pub(crate) const ATLANTIS_DEFAULT_MODEL: &str = "gpt-4o";
pub(crate) const ATLANTIS_CLIENT_ID: &str = "beb71cb9-a912-4821-b363-08bd49aa0fb0";
pub(crate) const ATLANTIS_TOKEN_URL: &str =
    "https://login.microsoftonline.com/45597f60-6e37-4be7-acfb-4c9e23b261ea/oauth2/v2.0/token";
pub(crate) const ATLANTIS_SCOPE: &str =
    "api://79f8f8aa-c15a-4b6e-a4d5-f6a420bb1524/.default";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct PostProcessProvider {
    pub id: String,
    pub label: String,
    pub base_url: String,
    #[serde(default)]
    pub allow_base_url_edit: bool,
    #[serde(default)]
    pub models_endpoint: Option<String>,
    #[serde(default)]
    pub supports_structured_output: bool,
}

pub fn default_providers() -> Vec<PostProcessProvider> {
    let mut providers = vec![
        PostProcessProvider {
            id: ATLANTIS_PROVIDER_ID.to_string(),
            label: "Atlantis".to_string(),
            base_url: ATLANTIS_BASE_URL.to_string(),
            allow_base_url_edit: false,
            models_endpoint: None,
            supports_structured_output: false,
        },
        PostProcessProvider {
            id: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
        PostProcessProvider {
            id: "openrouter".to_string(),
            label: "OpenRouter".to_string(),
            base_url: "https://gateway.ai.cloudflare.com/v1/793fcc3066fb7812c082e5eac9dbc75b/gateway/openrouter/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
    ];

    // Custom provider always comes last
    providers.push(PostProcessProvider {
        id: "custom".to_string(),
        label: "Custom".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        allow_base_url_edit: true,
        models_endpoint: Some("/models".to_string()),
        supports_structured_output: false,
    });

    providers
}

pub(crate) fn default_model_for_provider(provider_id: &str) -> String {
    match provider_id {
        ATLANTIS_PROVIDER_ID => ATLANTIS_DEFAULT_MODEL.to_string(),
        _ => String::new(),
    }
}

pub fn ensure_provider_defaults(settings: &mut AppSettings) -> bool {
    let mut changed = false;

    for provider in default_providers() {
        // Use match to do a single lookup - either sync existing or add new
        match settings
            .post_process_providers
            .iter_mut()
            .find(|p| p.id == provider.id)
        {
            Some(existing) => {
                // Sync supports_structured_output field for existing providers (migration)
                if existing.supports_structured_output != provider.supports_structured_output {
                    debug!(
                        "Updating supports_structured_output for provider '{}' from {} to {}",
                        provider.id,
                        existing.supports_structured_output,
                        provider.supports_structured_output
                    );
                    existing.supports_structured_output = provider.supports_structured_output;
                    changed = true;
                }
            }
            None => {
                // Provider doesn't exist, add it
                settings.post_process_providers.push(provider.clone());
                changed = true;
            }
        }

        if !settings.post_process_api_keys.contains_key(&provider.id) {
            settings
                .post_process_api_keys
                .insert(provider.id.clone(), String::new());
            changed = true;
        }

        let default_model = default_model_for_provider(&provider.id);
        match settings.post_process_models.get_mut(&provider.id) {
            Some(existing) => {
                if existing.is_empty() && !default_model.is_empty() {
                    *existing = default_model.clone();
                    changed = true;
                }
            }
            None => {
                settings
                    .post_process_models
                    .insert(provider.id.clone(), default_model);
                changed = true;
            }
        }
    }

    changed
}
