use crate::managers::model::EngineType;
use log::debug;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudProviderOption {
    pub key: String,
    pub label: String,
    pub option_type: CloudOptionType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum CloudOptionType {
    Text,
    Number { min: f64, max: f64, step: f64 },
    Boolean,
    Language,
    LanguageMulti,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum ProviderBackend {
    Local {
        engine_type: EngineType,
        filename: String,
        url: Option<String>,
        size_mb: u64,
        is_downloaded: bool,
        is_downloading: bool,
        partial_size: u64,
        is_directory: bool,
        accuracy_score: f32,
        speed_score: f32,
        is_custom: bool,
    },
    Cloud {
        base_url: String,
        default_model: String,
        console_url: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SttProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub supports_translation: bool,
    pub supports_realtime: bool,
    pub is_recommended: bool,
    pub backend: ProviderBackend,
    #[serde(default)]
    pub available_options: Vec<CloudProviderOption>,
    #[serde(default)]
    pub supports_dictionary_terms: bool,
    #[serde(default)]
    pub supports_dictionary_context: bool,
}

pub fn cloud_provider_registry() -> Vec<SttProviderInfo> {
    vec![
        SttProviderInfo {
            id: "openai_stt".to_string(),
            name: "OpenAI".to_string(),
            description: "onboarding.cloud.openai_stt.description".to_string(),
            supported_languages: vec![
                "af", "ar", "hy", "az", "be", "bs", "bg", "ca", "zh-Hans", "zh-Hant", "hr",
                "cs", "da", "nl", "en", "et", "fi", "fr", "gl", "de", "el",
                "he", "hi", "hu", "is", "id", "it", "ja", "kn", "kk", "ko",
                "lv", "lt", "mk", "ms", "mr", "mi", "ne", "no", "fa", "pl",
                "pt", "ro", "ru", "sr", "sk", "sl", "es", "sw", "sv", "tl",
                "ta", "th", "tr", "uk", "ur", "vi", "cy",
            ].into_iter().map(String::from).collect(),
            supports_translation: true,
            supports_realtime: false,
            is_recommended: false,
            backend: ProviderBackend::Cloud {
                base_url: "https://api.openai.com/v1".to_string(),
                default_model: "gpt-4o-mini-transcribe".to_string(),
                console_url: Some("https://platform.openai.com/api-keys".to_string()),
            },
            available_options: vec![
                CloudProviderOption {
                    key: "language".to_string(),
                    label: "settings.models.cloudProviders.options.language".to_string(),
                    option_type: CloudOptionType::Language,
                    description: String::new(),
                },
                CloudProviderOption {
                    key: "prompt".to_string(),
                    label: "settings.models.cloudProviders.options.prompt".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.options.promptDescription".to_string(),
                },
                CloudProviderOption {
                    key: "temperature".to_string(),
                    label: "settings.models.cloudProviders.options.temperature".to_string(),
                    option_type: CloudOptionType::Number { min: 0.0, max: 1.0, step: 0.1 },
                    description: "settings.models.cloudProviders.options.temperatureDescription".to_string(),
                },
            ],
            supports_dictionary_terms: true,
            supports_dictionary_context: true,
        },
        SttProviderInfo {
            id: "mistral".to_string(),
            name: "Mistral AI".to_string(),
            description: "onboarding.cloud.mistral.description".to_string(),
            supported_languages: vec![
                "en", "zh-Hans", "hi", "es", "ar", "fr", "pt", "ru", "de", "ja", "ko", "it", "nl",
            ].into_iter().map(String::from).collect(),
            supports_translation: false,
            supports_realtime: false,
            is_recommended: false,
            backend: ProviderBackend::Cloud {
                base_url: "https://api.mistral.ai".to_string(),
                default_model: "voxtral-mini-latest".to_string(),
                console_url: Some("https://console.mistral.ai".to_string()),
            },
            available_options: vec![
                CloudProviderOption {
                    key: "language".to_string(),
                    label: "settings.models.cloudProviders.options.language".to_string(),
                    option_type: CloudOptionType::Language,
                    description: String::new(),
                },
                CloudProviderOption {
                    key: "temperature".to_string(),
                    label: "settings.models.cloudProviders.options.temperature".to_string(),
                    option_type: CloudOptionType::Number { min: 0.0, max: 1.0, step: 0.1 },
                    description: "settings.models.cloudProviders.options.temperatureDescription".to_string(),
                },
                CloudProviderOption {
                    key: "diarize".to_string(),
                    label: "settings.models.cloudProviders.options.enableSpeakerDiarization".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.enableSpeakerDiarizationDescription".to_string(),
                },
                CloudProviderOption {
                    key: "context_bias".to_string(),
                    label: "settings.models.cloudProviders.options.contextBias".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.options.contextBiasDescription".to_string(),
                },
            ],
            supports_dictionary_terms: true,
            supports_dictionary_context: false,
        },
        SttProviderInfo {
            id: "soniox".to_string(),
            name: "Soniox".to_string(),
            description: "onboarding.cloud.soniox.description".to_string(),
            supported_languages: vec![
                "af", "sq", "ar", "az", "eu", "be", "bn", "bs", "bg", "ca",
                "zh-Hans", "zh-Hant", "hr", "cs", "da", "nl", "en", "et", "fi", "fr",
                "gl", "de", "el", "gu", "he", "hi", "hu", "id", "it", "ja",
                "kn", "kk", "ko", "lv", "lt", "mk", "ms", "ml", "mr", "no",
                "fa", "pl", "pt", "pa", "ro", "ru", "sr", "sk", "sl", "es",
                "sw", "sv", "tl", "ta", "te", "th", "tr", "uk", "ur", "vi", "cy",
            ].into_iter().map(String::from).collect(),
            supports_translation: false,
            supports_realtime: true,
            is_recommended: false,
            backend: ProviderBackend::Cloud {
                base_url: "https://api.soniox.com/v1".to_string(),
                default_model: "stt-rt-preview".to_string(),
                console_url: Some("https://console.soniox.com".to_string()),
            },
            available_options: vec![
                CloudProviderOption {
                    key: "language_hints".to_string(),
                    label: "settings.models.cloudProviders.options.languageHints".to_string(),
                    option_type: CloudOptionType::LanguageMulti,
                    description: String::new(),
                },
                CloudProviderOption {
                    key: "language_hints_strict".to_string(),
                    label: "settings.models.cloudProviders.options.languageHintsStrict".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.languageHintsStrictDescription".to_string(),
                },
                CloudProviderOption {
                    key: "context_terms".to_string(),
                    label: "settings.models.cloudProviders.options.contextTerms".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.options.contextTermsDescription".to_string(),
                },
                CloudProviderOption {
                    key: "context_description".to_string(),
                    label: "settings.models.cloudProviders.options.contextDescription".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.options.contextDescriptionDescription".to_string(),
                },
                CloudProviderOption {
                    key: "enable_speaker_diarization".to_string(),
                    label: "settings.models.cloudProviders.options.enableSpeakerDiarization".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.enableSpeakerDiarizationDescription".to_string(),
                },
                CloudProviderOption {
                    key: "enable_language_identification".to_string(),
                    label: "settings.models.cloudProviders.options.enableLanguageIdentification".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.enableLanguageIdentificationDescription".to_string(),
                },
            ],
            supports_dictionary_terms: true,
            supports_dictionary_context: true,
        },
    ]
}

/// Merge dictionary terms and context into the provider-specific cloud options.
///
/// For OpenAI: terms are prepended as `"Glossary: term1, term2. "` to the `prompt` field,
/// and context is prepended after the glossary. The user's own prompt text is preserved after.
///
/// For Soniox: terms are prepended to the `context_terms` field (comma-separated),
/// and context is prepended to the `context_description` field.
pub fn inject_dictionary(
    provider_id: &str,
    options: Option<serde_json::Value>,
    dictionary_terms: &[String],
    dictionary_context: &str,
) -> Option<serde_json::Value> {
    if dictionary_terms.is_empty() && dictionary_context.is_empty() {
        return options;
    }

    let mut opts = options.unwrap_or_else(|| serde_json::json!({}));

    match provider_id {
        "openai_stt" => {
            // Build the dictionary prefix for the prompt field
            let mut prefix_parts = Vec::new();
            if !dictionary_terms.is_empty() {
                prefix_parts.push(format!("Glossary: {}.", dictionary_terms.join(", ")));
            }
            if !dictionary_context.is_empty() {
                prefix_parts.push(dictionary_context.to_string());
            }
            let prefix = prefix_parts.join(" ");

            let existing_prompt = opts
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let merged = if existing_prompt.is_empty() {
                prefix
            } else {
                format!("{} {}", prefix, existing_prompt)
            };
            opts["prompt"] = serde_json::json!(merged);
            debug!(
                "Injected dictionary into OpenAI prompt ({} terms, {} chars context)",
                dictionary_terms.len(),
                dictionary_context.len()
            );
        }
        "mistral" => {
            // Merge terms into context_bias (comma-separated)
            if !dictionary_terms.is_empty() {
                let dict_terms_str = dictionary_terms.join(",");
                let existing_bias = opts
                    .get("context_bias")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let merged = if existing_bias.is_empty() {
                    dict_terms_str
                } else {
                    format!("{},{}", dict_terms_str, existing_bias)
                };
                opts["context_bias"] = serde_json::json!(merged);
            }
            debug!(
                "Injected dictionary into Mistral context_bias ({} terms)",
                dictionary_terms.len(),
            );
        }
        "soniox" => {
            // Merge terms into context_terms (comma-separated)
            if !dictionary_terms.is_empty() {
                let dict_terms_str = dictionary_terms.join(", ");
                let existing_terms = opts
                    .get("context_terms")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let merged = if existing_terms.is_empty() {
                    dict_terms_str
                } else {
                    format!("{}, {}", dict_terms_str, existing_terms)
                };
                opts["context_terms"] = serde_json::json!(merged);
            }

            // Merge context into context_description
            if !dictionary_context.is_empty() {
                let existing_desc = opts
                    .get("context_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let merged = if existing_desc.is_empty() {
                    dictionary_context.to_string()
                } else {
                    format!("{} {}", dictionary_context, existing_desc)
                };
                opts["context_description"] = serde_json::json!(merged);
            }
            debug!(
                "Injected dictionary into Soniox options ({} terms, {} chars context)",
                dictionary_terms.len(),
                dictionary_context.len()
            );
        }
        _ => {
            // Unknown provider — no injection
            debug!(
                "Dictionary injection skipped for unknown provider: {}",
                provider_id
            );
        }
    }

    Some(opts)
}
