use crate::post_process::providers::{
    default_model_for_provider, PostProcessProvider, ATLANTIS_API_VERSION, ATLANTIS_CLIENT_ID,
    ATLANTIS_PROVIDER_ID, ATLANTIS_SCOPE, ATLANTIS_TOKEN_URL,
};
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
    json_schema: JsonSchema,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub completion_tokens: Option<u64>,
    pub prompt_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AtlantisTokenResponse {
    access_token: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

struct AtlantisTokenCache {
    client_secret: String,
    access_token: String,
    expires_at: Instant,
}

static ATLANTIS_TOKEN_CACHE: Mutex<Option<AtlantisTokenCache>> = Mutex::new(None);
const ATLANTIS_TOKEN_EXPIRY_SKEW: Duration = Duration::from_secs(60);

fn is_atlantis(provider: &PostProcessProvider) -> bool {
    provider.id == ATLANTIS_PROVIDER_ID
}

fn form_value(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn atlantis_token_form(client_secret: &str) -> String {
    [
        ("client_id", ATLANTIS_CLIENT_ID),
        ("client_secret", client_secret),
        ("scope", ATLANTIS_SCOPE),
        ("grant_type", "client_credentials"),
    ]
    .into_iter()
    .map(|(key, value)| format!("{}={}", key, form_value(value)))
    .collect::<Vec<_>>()
    .join("&")
}

async fn acquire_atlantis_token(client_secret: &str) -> Result<String, String> {
    if client_secret.trim().is_empty() {
        return Err("Atlantis client secret is required".to_string());
    }

    if let Ok(cache) = ATLANTIS_TOKEN_CACHE.lock() {
        if let Some(cached) = cache.as_ref() {
            if cached.client_secret == client_secret && Instant::now() < cached.expires_at {
                return Ok(cached.access_token.clone());
            }
        }
    }

    debug!("Acquiring Atlantis Azure AD token");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build Atlantis token client: {}", e))?;

    let response = client
        .post(ATLANTIS_TOKEN_URL)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(atlantis_token_form(client_secret))
        .send()
        .await
        .map_err(|e| format!("Atlantis token request failed: {}", e))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Atlantis token response: {}", e))?;

    let token_response: AtlantisTokenResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse Atlantis token response: {}", e))?;

    if !status.is_success() {
        let message = token_response
            .error_description
            .or(token_response.error)
            .unwrap_or_else(|| format!("HTTP {}", status));
        return Err(format!("Failed to acquire Atlantis token: {}", message));
    }

    let access_token = token_response
        .access_token
        .ok_or_else(|| "Atlantis token response did not include an access token".to_string())?;
    let expires_in = token_response.expires_in.unwrap_or(1800);
    let cache_ttl = Duration::from_secs(expires_in).saturating_sub(ATLANTIS_TOKEN_EXPIRY_SKEW);

    if let Ok(mut cache) = ATLANTIS_TOKEN_CACHE.lock() {
        *cache = Some(AtlantisTokenCache {
            client_secret: client_secret.to_string(),
            access_token: access_token.clone(),
            expires_at: Instant::now() + cache_ttl,
        });
    }

    Ok(access_token)
}

async fn resolve_auth_token(
    provider: &PostProcessProvider,
    api_key: &str,
) -> Result<String, String> {
    if is_atlantis(provider) {
        acquire_atlantis_token(api_key).await
    } else {
        Ok(api_key.to_string())
    }
}

fn chat_completions_url(provider: &PostProcessProvider, model: &str) -> String {
    let base_url = provider.base_url.trim_end_matches('/');
    if is_atlantis(provider) {
        return format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            base_url,
            model.trim(),
            ATLANTIS_API_VERSION
        );
    }

    format!("{}/chat/completions", base_url)
}

fn models_url(provider: &PostProcessProvider) -> Option<String> {
    let endpoint = provider.models_endpoint.as_deref()?;

    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return Some(endpoint.to_string());
    }

    let base_url = provider.base_url.trim_end_matches('/');
    let endpoint = if endpoint.starts_with('/') {
        endpoint.to_string()
    } else {
        format!("/{}", endpoint)
    };

    Some(format!("{}{}", base_url, endpoint))
}

/// Build headers for API requests based on provider type
fn build_headers(provider: &PostProcessProvider, auth_token: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    // Common headers
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/elwin/handless"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Handless/1.0 (+https://github.com/elwin/handless)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("Handless"));

    // Provider-specific auth headers
    if !auth_token.is_empty() {
        if provider.id == "anthropic" {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(auth_token)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        } else {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", auth_token))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

/// Create an HTTP client with provider-specific headers
fn create_client(
    provider: &PostProcessProvider,
    auth_token: &str,
) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, auth_token)?;
    reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(is_atlantis(provider))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Send a chat completion request to an OpenAI-compatible API.
/// Always uses system + user message structure.
/// When json_schema is provided, includes response_format for structured outputs.
pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    user_content: String,
    system_prompt: String,
    json_schema: Option<Value>,
) -> Result<(Option<String>, Option<Usage>), String> {
    let url = chat_completions_url(provider, model);

    debug!("Sending chat completion request to: {}", url);

    let auth_token = resolve_auth_token(provider, &api_key).await?;
    let client = create_client(provider, &auth_token)?;

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_content,
        },
    ];

    // Build response_format if schema is provided
    let response_format = json_schema.map(|schema| ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: JsonSchema {
            name: "transcription_output".to_string(),
            strict: true,
            schema,
        },
    });

    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        response_format,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let content = completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone());
    Ok((content, completion.usage))
}

/// Fetch available models from an OpenAI-compatible API
/// Returns a list of model IDs
pub async fn fetch_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let auth_token = resolve_auth_token(provider, &api_key).await?;

    let Some(url) = models_url(provider) else {
        let default_model = default_model_for_provider(&provider.id);
        return if default_model.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(vec![default_model])
        };
    };

    debug!("Fetching models from: {}", url);

    let client = create_client(provider, &auth_token)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut models = Vec::new();

    // Handle OpenAI format: { data: [ { id: "..." }, ... ] }
    if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|i| i.as_str()) {
                models.push(id.to_string());
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                models.push(name.to_string());
            }
        }
    }
    // Handle array format: [ "model1", "model2", ... ]
    else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            }
        }
    }

    Ok(models)
}
