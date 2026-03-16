use anyhow::Result;
use log::debug;
use reqwest::multipart;

/// Test API key and model by sending a minimal silent audio clip.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let wav_bytes = super::test_silence_wav()?;

    let url = format!("{}/speech-to-text", base_url.trim_end_matches('/'));

    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("test.wav")
        .mime_str("audio/wav")?;

    let form = multipart::Form::new()
        .part("file", file_part)
        .text("model_id", model.to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    super::check_response(response, "API test failed").await?;

    Ok(())
}

/// Transcribe audio using ElevenLabs' /v1/speech-to-text endpoint.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let url = format!("{}/speech-to-text", base_url.trim_end_matches('/'));

    debug!(
        "ElevenLabs STT request: url={}, model={}, audio_size={}",
        url,
        model,
        audio_wav.len()
    );

    let file_part = multipart::Part::bytes(audio_wav)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model_id", model.to_string());

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                form = form.text("language_code", lang.to_string());
            }
        }
        if opts
            .get("enable_speaker_diarization")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            form = form.text("diarize", "true");
        }
        if let Some(keyterms) = opts.get("keyterms").and_then(|v| v.as_array()) {
            let terms_json = serde_json::to_string(keyterms).unwrap_or_default();
            if !terms_json.is_empty() {
                form = form.text("keyterms", terms_json);
            }
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    let response = super::check_response(response, "ElevenLabs STT API error").await?;

    let result: super::TranscriptionResponse = response.json().await?;
    debug!("ElevenLabs STT result: '{}'", result.text);
    Ok(result.text)
}
