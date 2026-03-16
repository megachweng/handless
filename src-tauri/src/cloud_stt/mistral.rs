use anyhow::Result;
use log::debug;
use reqwest::multipart;

/// Test API key and model by sending a minimal silent audio clip.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let wav_bytes = super::test_silence_wav()?;

    let url = format!("{}/v1/audio/transcriptions", base_url.trim_end_matches('/'));

    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("test.wav")
        .mime_str("audio/wav")?;

    let form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    super::check_response(response, "API test failed").await?;

    Ok(())
}

/// Transcribe audio using Mistral's /v1/audio/transcriptions endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let url = format!("{}/v1/audio/transcriptions", base_url.trim_end_matches('/'));

    debug!(
        "Mistral STT request: url={}, model={}, audio_size={}",
        url,
        model,
        audio_wav.len()
    );

    let file_part = multipart::Part::bytes(audio_wav)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string());

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                form = form.text("language", lang.to_string());
            }
        }
        if let Some(temp) = opts.get("temperature").and_then(|v| v.as_f64()) {
            form = form.text("temperature", temp.to_string());
        }
        if opts
            .get("diarize")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            form = form.text("diarize", "true".to_string());
        }
        if let Some(bias) = opts.get("context_bias").and_then(|v| v.as_str()) {
            if !bias.is_empty() {
                let terms: Vec<&str> = bias
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                let json_array = serde_json::to_string(&terms).unwrap_or_default();
                form = form.text("context_bias", json_array);
            }
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    let response = super::check_response(response, "Mistral STT API error").await?;

    let result: super::TranscriptionResponse = response.json().await?;
    debug!("Mistral STT result: '{}'", result.text);
    Ok(result.text)
}
