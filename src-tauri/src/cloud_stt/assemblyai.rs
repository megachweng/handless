use anyhow::Result;
use log::debug;
use serde::Deserialize;

#[derive(Deserialize)]
struct UploadResponse {
    upload_url: String,
}

#[derive(Deserialize)]
struct TranscriptCreateResponse {
    id: String,
}

#[derive(Deserialize)]
struct TranscriptPollResponse {
    status: String,
    text: Option<String>,
    error: Option<String>,
}

/// Test API key and model by uploading a minimal WAV and creating a transcript request.
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()> {
    let base = base_url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let wav_bytes = super::test_silence_wav()?;

    // 1. Upload file (validates API key)
    let response = client
        .post(format!("{}/v2/upload", base))
        .header("Authorization", api_key)
        .header("Content-Type", "application/octet-stream")
        .body(wav_bytes)
        .send()
        .await?;

    let response = super::check_response(response, "AssemblyAI upload error").await?;

    let upload: UploadResponse = response.json().await?;

    // 2. Create transcript (validates model)
    let body = serde_json::json!({
        "audio_url": upload.upload_url,
        "speech_model": model,
    });

    let response = client
        .post(format!("{}/v2/transcript", base))
        .header("Authorization", api_key)
        .json(&body)
        .send()
        .await?;

    super::check_response(response, "AssemblyAI transcript create error").await?;

    Ok(())
}

/// Transcribe audio using the AssemblyAI API.
///
/// Flow: upload audio -> create transcript -> poll until complete -> return text.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let base = base_url.trim_end_matches('/');
    let client = reqwest::Client::new();

    debug!(
        "AssemblyAI STT request: base_url={}, model={}, audio_size={}",
        base,
        model,
        audio_wav.len()
    );

    // 1. Upload the audio file
    let response = client
        .post(format!("{}/v2/upload", base))
        .header("Authorization", api_key)
        .header("Content-Type", "application/octet-stream")
        .body(audio_wav)
        .send()
        .await?;

    let response = super::check_response(response, "AssemblyAI upload error").await?;

    let upload: UploadResponse = response.json().await?;
    debug!("AssemblyAI file uploaded: url={}", upload.upload_url);

    // 2. Create a transcript
    let mut body = serde_json::json!({
        "audio_url": upload.upload_url,
        "speech_model": model,
    });
    if let Some(opts) = options {
        if let Some(lang) = opts.get("language_code").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                body["language_code"] = serde_json::json!(lang);
            }
        }
        if opts
            .get("speaker_labels")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            body["speaker_labels"] = serde_json::json!(true);
        }
        if let Some(word_boost) = opts.get("word_boost").and_then(|v| v.as_array()) {
            if !word_boost.is_empty() {
                body["word_boost"] = serde_json::json!(word_boost);
            }
        }
    }

    let response = client
        .post(format!("{}/v2/transcript", base))
        .header("Authorization", api_key)
        .json(&body)
        .send()
        .await?;

    let response = super::check_response(response, "AssemblyAI transcript create error").await?;

    let transcript: TranscriptCreateResponse = response.json().await?;
    debug!("AssemblyAI transcript created: id={}", transcript.id);

    // 3. Poll until the transcript completes
    let poll_url = format!("{}/v2/transcript/{}", base, transcript.id);
    let max_polls = 600; // 600 * 500ms = 5 minutes
    for _ in 0..max_polls {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let response = client
            .get(&poll_url)
            .header("Authorization", api_key)
            .send()
            .await?;

        let response = super::check_response(response, "AssemblyAI transcript poll error").await?;

        let poll: TranscriptPollResponse = response.json().await?;
        debug!("AssemblyAI transcript status: {}", poll.status);

        match poll.status.as_str() {
            "completed" => {
                let text = poll.text.unwrap_or_default();
                debug!("AssemblyAI STT result: '{}'", text);
                return Ok(text);
            }
            "error" => {
                let msg = poll.error.unwrap_or_else(|| "unknown error".to_string());
                return Err(anyhow::anyhow!("AssemblyAI transcription failed: {}", msg));
            }
            _ => continue, // "queued" or "processing"
        }
    }

    // If we exhausted all polls without completing
    Err(anyhow::anyhow!(
        "AssemblyAI transcription timed out after 5 minutes"
    ))
}
