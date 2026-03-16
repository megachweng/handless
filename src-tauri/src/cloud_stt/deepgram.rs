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

/// Transcribe audio using Deepgram's /v1/listen endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let mut params: Vec<(&str, String)> = vec![("model", model.to_string())];

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                params.push(("language", lang.to_string()));
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
        if let Some(keywords) = opts.get("keywords").and_then(|v| v.as_str()) {
            if !keywords.is_empty() {
                for kw in keywords
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    params.push(("keywords", kw.to_string()));
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
