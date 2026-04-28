use log::debug;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloudProviderOption {
    pub key: String,
    pub label: String,
    pub option_type: CloudOptionType,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<CloudOptionDefault>,
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
#[serde(untagged)]
pub enum CloudOptionDefault {
    Bool(bool),
    Text(String),
    Number(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum ProviderBackend {
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
            id: "soniox".to_string(),
            name: "Soniox".to_string(),
            description: "onboarding.cloud.soniox.description".to_string(),
            supported_languages: vec![
                "af", "sq", "ar", "az", "eu", "be", "bn", "bs", "bg", "ca", "zh-Hans",
                "zh-Hant", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "gl", "de",
                "el", "gu", "he", "hi", "hu", "id", "it", "ja", "kn", "kk", "ko", "lv",
                "lt", "mk", "ms", "ml", "mr", "no", "fa", "pl", "pt", "pa", "ro", "ru",
                "sr", "sk", "sl", "es", "sw", "sv", "tl", "ta", "te", "th", "tr", "uk",
                "ur", "vi", "cy",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            supports_translation: true,
            supports_realtime: true,
            is_recommended: true,
            backend: ProviderBackend::Cloud {
                base_url: "https://api.soniox.com/v1".to_string(),
                default_model: "stt-rt-v4".to_string(),
                console_url: Some("https://console.soniox.com".to_string()),
            },
            available_options: vec![
                CloudProviderOption {
                    key: "language_hints".to_string(),
                    label: "settings.models.cloudProviders.options.languageHints".to_string(),
                    option_type: CloudOptionType::LanguageMulti,
                    description: String::new(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "language_hints_strict".to_string(),
                    label: "settings.models.cloudProviders.options.languageHintsStrict".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description:
                        "settings.models.cloudProviders.options.languageHintsStrictDescription"
                            .to_string(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "context_terms".to_string(),
                    label: "settings.models.cloudProviders.options.contextTerms".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.options.contextTermsDescription"
                        .to_string(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "context_description".to_string(),
                    label: "settings.models.cloudProviders.options.contextDescription".to_string(),
                    option_type: CloudOptionType::Text,
                    description:
                        "settings.models.cloudProviders.options.contextDescriptionDescription"
                            .to_string(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "enable_speaker_diarization".to_string(),
                    label: "settings.models.cloudProviders.options.enableSpeakerDiarization"
                        .to_string(),
                    option_type: CloudOptionType::Boolean,
                    description:
                        "settings.models.cloudProviders.options.enableSpeakerDiarizationDescription"
                            .to_string(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "enable_language_identification".to_string(),
                    label: "settings.models.cloudProviders.options.enableLanguageIdentification"
                        .to_string(),
                    option_type: CloudOptionType::Boolean,
                    description:
                        "settings.models.cloudProviders.options.enableLanguageIdentificationDescription"
                            .to_string(),
                    default_value: None,
                },
            ],
            supports_dictionary_terms: true,
            supports_dictionary_context: true,
        },
        SttProviderInfo {
            id: "doubao".to_string(),
            name: "Doubao".to_string(),
            description: "onboarding.cloud.doubao.description".to_string(),
            supported_languages: vec![
                "zh-Hans", "zh-Hant", "yue", "en", "ja", "ko", "es", "fr", "de", "ru", "pt",
                "it", "ar", "th", "vi", "id", "ms", "bn", "el", "nl", "tr", "pl", "ro", "ne",
                "uk",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            supports_translation: false,
            supports_realtime: true,
            is_recommended: false,
            backend: ProviderBackend::Cloud {
                base_url: "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async".to_string(),
                default_model: "bigmodel".to_string(),
                console_url: Some("https://console.volcengine.com/speech/app".to_string()),
            },
            available_options: vec![
                CloudProviderOption {
                    key: "app_key".to_string(),
                    label: "settings.models.cloudProviders.doubao.appKey".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.doubao.appKeyDescription"
                        .to_string(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "resource_id".to_string(),
                    label: "settings.models.cloudProviders.doubao.resourceId".to_string(),
                    option_type: CloudOptionType::Text,
                    description: "settings.models.cloudProviders.doubao.resourceIdDescription"
                        .to_string(),
                    default_value: Some(CloudOptionDefault::Text(
                        "volc.seedasr.sauc.duration".to_string(),
                    )),
                },
                CloudProviderOption {
                    key: "language".to_string(),
                    label: "settings.models.cloudProviders.options.language".to_string(),
                    option_type: CloudOptionType::Language,
                    description: String::new(),
                    default_value: None,
                },
                CloudProviderOption {
                    key: "enable_itn".to_string(),
                    label: "settings.models.cloudProviders.options.enableItn".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.enableItnDescription"
                        .to_string(),
                    default_value: Some(CloudOptionDefault::Bool(true)),
                },
                CloudProviderOption {
                    key: "enable_punc".to_string(),
                    label: "settings.models.cloudProviders.options.punctuate".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.punctuateDescription"
                        .to_string(),
                    default_value: Some(CloudOptionDefault::Bool(true)),
                },
                CloudProviderOption {
                    key: "enable_ddc".to_string(),
                    label: "settings.models.cloudProviders.options.enableDdc".to_string(),
                    option_type: CloudOptionType::Boolean,
                    description: "settings.models.cloudProviders.options.enableDdcDescription"
                        .to_string(),
                    default_value: Some(CloudOptionDefault::Bool(true)),
                },
            ],
            supports_dictionary_terms: true,
            supports_dictionary_context: true,
        },
    ]
}

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
        "doubao" => {
            if !dictionary_terms.is_empty() {
                let dict_hotwords = dictionary_terms.join(", ");
                let existing_hotwords = opts
                    .get("hotwords")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let merged = if existing_hotwords.is_empty() {
                    dict_hotwords
                } else {
                    format!("{}, {}", dict_hotwords, existing_hotwords)
                };
                opts["hotwords"] = serde_json::json!(merged);
            }
            if !dictionary_context.is_empty() {
                opts["dialog_context"] = serde_json::json!(dictionary_context);
            }
            debug!(
                "Injected dictionary into Doubao options ({} terms, {} chars context)",
                dictionary_terms.len(),
                dictionary_context.len(),
            );
        }
        "soniox" => {
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
            debug!(
                "Dictionary injection skipped for unknown provider: {}",
                provider_id
            );
        }
    }

    Some(opts)
}
