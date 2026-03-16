use anyhow::Result;
use log::debug;
use reqwest::multipart;
use serde::Deserialize;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

/// Test API key and model by sending a minimal silent audio clip.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let wav_bytes = crate::audio_toolkit::audio::encode_wav_bytes(&vec![0.0f32; 1600])?;

    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));

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

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Fireworks STT test error ({}): {}",
            status,
            body
        ));
    }

    Ok(())
}

/// Transcribe audio using Fireworks AI's /v1/audio/transcriptions endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));

    debug!(
        "Fireworks STT request: url={}, model={}, audio_size={}",
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
                let code = lang.split('-').next().unwrap_or(lang);
                form = form.text("language", code.to_string());
            }
        }
        if let Some(prompt) = opts.get("prompt").and_then(|v| v.as_str()) {
            if !prompt.is_empty() {
                form = form.text("prompt", prompt.to_string());
            }
        }
        if let Some(temp) = opts.get("temperature").and_then(|v| v.as_f64()) {
            form = form.text("temperature", temp.to_string());
        }
        if let Some(diarize) = opts.get("diarize").and_then(|v| v.as_bool()) {
            if diarize {
                form = form.text("diarize", "true".to_string());
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

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Fireworks STT API error ({}): {}",
            status,
            body
        ));
    }

    let result: TranscriptionResponse = response.json().await?;
    debug!("Fireworks STT result: '{}'", result.text);
    Ok(result.text)
}
