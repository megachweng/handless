use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

const DEEPGRAM_WS_URL: &str = "wss://api.deepgram.com/v1/listen";
const CHUNK_SIZE: usize = 3840;
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Build the WebSocket URL with query parameters for Deepgram's live transcription API.
fn build_ws_url(model: &str, encoding: &str, options: Option<&serde_json::Value>) -> String {
    let lang = options
        .and_then(|o| o.get("language"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let effective_model =
        crate::cloud_stt::deepgram::resolve_model_for_language(model, lang);

    let mut params = vec![
        ("model".to_string(), effective_model.to_string()),
        ("encoding".to_string(), encoding.to_string()),
        ("sample_rate".to_string(), "16000".to_string()),
        ("channels".to_string(), "1".to_string()),
    ];

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                params.push((
                    "language".to_string(),
                    crate::cloud_stt::strip_lang_subtag(lang).to_string(),
                ));
            }
        }
        if opts
            .get("smart_format")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("smart_format".to_string(), "true".to_string()));
        }
        if opts
            .get("punctuate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("punctuate".to_string(), "true".to_string()));
        }
        if opts
            .get("diarize")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("diarize".to_string(), "true".to_string()));
        }
        if opts
            .get("interim_results")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            params.push(("interim_results".to_string(), "true".to_string()));
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
                    params.push((param_name.to_string(), kw.to_string()));
                }
            }
        }
    }

    let query: String = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                percent_encoding::utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC),
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", DEEPGRAM_WS_URL, query)
}

/// Build a WebSocket request with the Authorization header for the Deepgram WebSocket.
fn build_ws_request(
    api_key: &str,
    url: &str,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
    let mut request = url.into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Token {}", api_key).parse()?);
    Ok(request)
}

/// Extract the transcript text from a Deepgram "Results" response message.
/// Returns `(transcript, is_final)`.
fn extract_transcript(resp: &serde_json::Value) -> (String, bool) {
    let transcript = resp
        .get("channel")
        .and_then(|ch| ch.get("alternatives"))
        .and_then(|alts| alts.as_array())
        .and_then(|alts| alts.first())
        .and_then(|alt| alt.get("transcript"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let is_final = resp
        .get("is_final")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    (transcript, is_final)
}

/// Check a Deepgram response for error fields and return an appropriate error.
fn check_error(resp: &serde_json::Value) -> Option<anyhow::Error> {
    // Deepgram errors may come as {"err_code": "...", "err_msg": "..."} or
    // {"type": "Error", "message": "..."} depending on the error source.
    if let Some(code) = resp.get("err_code").and_then(|v| v.as_str()) {
        let msg = resp
            .get("err_msg")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Some(anyhow::anyhow!("Deepgram RT error ({}): {}", code, msg));
    }
    if resp.get("type").and_then(|v| v.as_str()) == Some("Error") {
        let msg = resp
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Some(anyhow::anyhow!("Deepgram RT error: {}", msg));
    }
    None
}

/// Test API key by opening a WebSocket connection and checking for auth errors.
/// A successful handshake validates the key; Deepgram will reject invalid tokens
/// during the HTTP upgrade.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let url = build_ws_url(model, "linear16", None);
    let request = build_ws_request(api_key, &url)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Deepgram RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Send CloseStream immediately — we only need to verify auth
    write
        .send(Message::Text(
            serde_json::json!({"type": "CloseStream"})
                .to_string()
                .into(),
        ))
        .await?;

    // Read the first response to check for errors
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                if let Some(err) = check_error(&resp) {
                    return Err(err);
                }
            }
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Deepgram RT: timed out waiting for response"
            ));
        }
    }

    let _ = write.send(Message::Close(None)).await;
    Ok(())
}

pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    debug!(
        "Deepgram RT: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    let url = build_ws_url(model, "linear16", options);
    let request = build_ws_request(api_key, &url)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Deepgram RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Send audio in chunks
    for chunk in audio_wav.chunks(CHUNK_SIZE) {
        write.send(Message::Binary(chunk.to_vec().into())).await?;
    }

    // Signal end of audio
    write
        .send(Message::Text(
            serde_json::json!({"type": "CloseStream"})
                .to_string()
                .into(),
        ))
        .await?;

    let mut final_text = String::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "Deepgram RT: timed out waiting for transcription"
                ));
            }
        };

        if let Message::Text(text) = msg {
            let resp: serde_json::Value = serde_json::from_str(&text)?;

            if let Some(err) = check_error(&resp) {
                let _ = write.send(Message::Close(None)).await;
                return Err(err);
            }

            let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if msg_type == "Results" {
                let (transcript, is_final) = extract_transcript(&resp);
                if is_final && !transcript.is_empty() {
                    if !final_text.is_empty() {
                        final_text.push(' ');
                    }
                    final_text.push_str(&transcript);
                }
            }
        }
    }

    debug!("Deepgram RT result: '{}'", final_text);
    Ok(final_text.trim().to_string())
}

use super::StreamingHandles;

/// Start a streaming WebSocket session. Returns handles for the sender and reader tasks.
/// The caller should drop `audio_rx` (by dropping the sender side) to signal end-of-audio,
/// then await the handles.
pub async fn start_streaming(
    api_key: &str,
    model: &str,
    mut audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    options: Option<serde_json::Value>,
    delta_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<StreamingHandles> {
    // Enable interim_results for streaming so we can show partial transcripts
    let mut opts = options.unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = opts.as_object_mut() {
        obj.entry("interim_results")
            .or_insert(serde_json::json!(true));
    }

    let url = build_ws_url(model, "linear16", Some(&opts));
    let request = build_ws_request(api_key, &url)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Deepgram RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Sender task: reads audio frames from the channel, converts to i16 LE bytes, sends as binary
    let sender_handle = tokio::spawn(async move {
        while let Some(frame) = audio_rx.recv().await {
            let bytes: Vec<u8> = frame
                .iter()
                .map(|&s| {
                    let clamped = s.clamp(-1.0, 1.0);
                    (clamped * i16::MAX as f32) as i16
                })
                .flat_map(|s| s.to_le_bytes())
                .collect();

            for chunk in bytes.chunks(CHUNK_SIZE) {
                write.send(Message::Binary(chunk.to_vec().into())).await?;
            }
        }
        // Signal end of audio — send CloseStream so the server flushes
        // any remaining results before closing.
        write
            .send(Message::Text(
                serde_json::json!({"type": "CloseStream"})
                    .to_string()
                    .into(),
            ))
            .await?;
        Ok(())
    });

    // Reader task: accumulates final transcript segments
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Deepgram RT streaming: timed out waiting for transcription"
                    ));
                }
            };

            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;

                if let Some(err) = check_error(&resp) {
                    return Err(err);
                }

                let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

                if msg_type == "Results" {
                    let (transcript, is_final) = extract_transcript(&resp);

                    if is_final && !transcript.is_empty() {
                        if !final_text.is_empty() {
                            final_text.push(' ');
                        }
                        final_text.push_str(&transcript);
                    }

                    // Show final + interim text for immediate feedback
                    if let Some(tx) = &delta_tx {
                        let display = if is_final {
                            final_text.clone()
                        } else {
                            let mut display = final_text.clone();
                            if !transcript.is_empty() {
                                if !display.is_empty() {
                                    display.push(' ');
                                }
                                display.push_str(&transcript);
                            }
                            display
                        };
                        let _ = tx.send(display);
                    }
                }
            }
        }

        debug!("Deepgram RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}
