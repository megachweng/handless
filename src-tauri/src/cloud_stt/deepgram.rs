use anyhow::Result;
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;

#[derive(Deserialize)]
struct Alternative {
    transcript: String,
}

#[derive(Deserialize)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Deserialize)]
struct Results {
    channels: Vec<Channel>,
}

#[derive(Deserialize)]
struct DeepgramResponse {
    results: Results,
}

/// Build a URL with properly percent-encoded query parameters.
fn build_url(base: &str, params: &[(&str, &str)]) -> String {
    let query: String = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                utf8_percent_encode(k, NON_ALPHANUMERIC),
                utf8_percent_encode(v, NON_ALPHANUMERIC),
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{}/listen?{}", base.trim_end_matches('/'), query)
}

/// Test API key and model by sending a minimal silent audio clip.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let wav_bytes = super::test_silence_wav()?;

    let url = build_url(base_url, &[("model", model)]);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/wav")
        .body(wav_bytes)
        .send()
        .await?;

    super::check_response(response, "API test failed").await?;

    Ok(())
}

/// Resolve the model for a given language.  Nova-3 does not support Chinese;
/// fall back to nova-2 which does.
pub fn resolve_model_for_language<'a>(model: &'a str, language: Option<&str>) -> &'a str {
    if let Some(lang) = language {
        if lang.starts_with("zh") && model.starts_with("nova-3") {
            return "nova-2";
        }
    }
    model
}

/// Transcribe audio using Deepgram's /v1/listen endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let lang = options
        .and_then(|o| o.get("language"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let effective_model = resolve_model_for_language(model, lang);

    let mut params: Vec<(&str, String)> = vec![("model", effective_model.to_string())];

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                params.push(("language", super::strip_lang_subtag(lang).to_string()));
            }
        }
        if opts
            .get("smart_format")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("smart_format", "true".to_string()));
        }
        if opts
            .get("punctuate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("punctuate", "true".to_string()));
        }
        if opts
            .get("diarize")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("diarize", "true".to_string()));
        }
        if let Some(keyterm) = opts.get("keyterm").and_then(|v| v.as_str()) {
            if !keyterm.is_empty() {
                // nova-2 uses "keywords" instead of "keyterm"
                let param_name = if effective_model.starts_with("nova-2") {
                    "keywords"
                } else {
                    "keyterm"
                };
                for kw in keyterm
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    params.push((param_name, kw.to_string()));
                }
            }
        }
    }

    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let url = build_url(base_url, &param_refs);

    debug!(
        "Deepgram STT request: url={}, model={}, audio_size={}",
        url,
        model,
        audio_wav.len()
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "audio/wav")
        .body(audio_wav)
        .send()
        .await?;

    let response = super::check_response(response, "Deepgram STT API error").await?;

    let result: DeepgramResponse = response.json().await?;
    let transcript = result
        .results
        .channels
        .first()
        .and_then(|ch| ch.alternatives.first())
        .map(|alt| alt.transcript.clone())
        .unwrap_or_default();

    debug!("Deepgram STT result: '{}'", transcript);
    Ok(transcript)
}
