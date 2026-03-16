use anyhow::Result;
use log::debug;
use reqwest::multipart;

const CARTESIA_API_VERSION: &str = "2026-03-01";

/// Test API key and model by sending a minimal silent audio clip.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let wav_bytes = super::test_silence_wav()?;

    let url = format!("{}/stt", base_url.trim_end_matches('/'));

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
        .header("Cartesia-Version", CARTESIA_API_VERSION)
        .multipart(form)
        .send()
        .await?;

    super::check_response(response, "API test failed").await?;

    Ok(())
}

/// Transcribe audio using Cartesia's /stt endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let url = format!("{}/stt", base_url.trim_end_matches('/'));

    debug!(
        "Cartesia STT request: url={}, model={}, audio_size={}",
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
                let code = super::strip_lang_subtag(lang);
                form = form.text("language", code.to_string());
            }
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .header("Cartesia-Version", CARTESIA_API_VERSION)
        .multipart(form)
        .send()
        .await?;

    let response = super::check_response(response, "Cartesia STT API error").await?;

    let result: super::TranscriptionResponse = response.json().await?;
    debug!("Cartesia STT result: '{}'", result.text);
    Ok(result.text)
}
