use anyhow::Result;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::StreamingHandles;

/// Mistral realtime transcription WebSocket endpoint.
/// The model is passed as a query parameter on the URL.
const MISTRAL_WS_BASE: &str = "wss://api.mistral.ai/v1/audio/transcriptions/realtime";

/// Max decoded PCM bytes per `input_audio.append` message (Mistral limit: 262 144).
const MAX_AUDIO_CHUNK_BYTES: usize = 262_144;

/// How long we wait for a server response before giving up.
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Build a WebSocket request with Bearer auth and the model in the query string.
fn build_ws_request(api_key: &str, model: &str) -> Result<Request<()>> {
    let url = format!("{}?model={}", MISTRAL_WS_BASE, model);
    let request = Request::builder()
        .uri(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Host", "api.mistral.ai")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .body(())?;
    Ok(request)
}

/// Build the `input_audio.append` JSON message with base64-encoded PCM bytes.
fn audio_append_msg(pcm_bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(pcm_bytes);
    serde_json::json!({
        "type": "input_audio.append",
        "audio": b64,
    })
    .to_string()
}

/// `input_audio.flush` — ask the server to process buffered audio.
fn audio_flush_msg() -> String {
    serde_json::json!({"type": "input_audio.flush"}).to_string()
}

/// `input_audio.end` — signal that no more audio will be sent.
fn audio_end_msg() -> String {
    serde_json::json!({"type": "input_audio.end"}).to_string()
}

/// Extract an error string from a Mistral `error` event.
fn extract_error(resp: &serde_json::Value) -> String {
    resp.get("error")
        .and_then(|e| {
            // error.message can be a string or an object with a `detail` field
            let msg = e.get("message")?;
            if let Some(s) = msg.as_str() {
                return Some(s.to_string());
            }
            if let Some(detail) = msg.get("detail").and_then(|d| d.as_str()) {
                return Some(detail.to_string());
            }
            None
        })
        .unwrap_or_else(|| "unknown error".to_string())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Test API key by opening a WebSocket connection and waiting for the
/// `session.created` handshake event.  Any auth or model error is surfaced.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let request = build_ws_request(api_key, model)?;
    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Mistral RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session.created (or an error)
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if msg_type == "error" {
                    return Err(anyhow::anyhow!(
                        "Mistral RT error: {}",
                        extract_error(&resp)
                    ));
                }
                // session.created is the expected happy path
            }
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Mistral RT: timed out waiting for response"
            ));
        }
    }

    // Signal end and close
    let _ = write.send(Message::Text(audio_end_msg().into())).await;
    let _ = write.send(Message::Close(None)).await;
    Ok(())
}

/// Transcribe a complete audio buffer (WAV or raw PCM).
/// The audio is sent as base64-encoded PCM chunks over the WebSocket, then
/// we collect `transcription.text.delta` events until `transcription.done`.
pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    debug!(
        "Mistral RT: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    let request = build_ws_request(api_key, model)?;
    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Mistral RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session.created before sending audio
    wait_for_session_created(&mut read).await?;

    // Optionally update session with language hint
    if let Some(opts) = options {
        if let Some(update) = build_session_update(opts) {
            write.send(Message::Text(update.into())).await?;
        }
    }

    // Send audio in chunks
    for chunk in audio_wav.chunks(MAX_AUDIO_CHUNK_BYTES) {
        write
            .send(Message::Text(audio_append_msg(chunk).into()))
            .await?;
    }
    write.send(Message::Text(audio_flush_msg().into())).await?;
    write.send(Message::Text(audio_end_msg().into())).await?;

    // Collect the transcript
    let mut final_text = String::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "Mistral RT: timed out waiting for transcription"
                ));
            }
        };

        if let Message::Text(text) = msg {
            let resp: serde_json::Value = serde_json::from_str(&text)?;
            let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if msg_type == "error" {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "Mistral RT error: {}",
                    extract_error(&resp)
                ));
            }

            if msg_type == "transcription.text.delta" {
                if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                    final_text.push_str(t);
                }
            }

            if msg_type == "transcription.done" {
                // The done event also carries the full text; prefer it if present
                if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                    if !t.is_empty() {
                        final_text = t.to_string();
                    }
                }
                break;
            }
        }
    }

    debug!("Mistral RT result: '{}'", final_text);
    Ok(final_text.trim().to_string())
}

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
    let request = build_ws_request(api_key, model)?;
    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Mistral RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session.created before sending audio
    wait_for_session_created(&mut read).await?;

    // Optionally update session with language hint
    if let Some(opts) = &options {
        if let Some(update) = build_session_update(opts) {
            write.send(Message::Text(update.into())).await?;
        }
    }

    // Sender task: reads f32 audio frames, converts to i16 LE PCM, base64-encodes,
    // and sends as `input_audio.append` JSON messages.
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

            for chunk in bytes.chunks(MAX_AUDIO_CHUNK_BYTES) {
                write
                    .send(Message::Text(audio_append_msg(chunk).into()))
                    .await?;
            }
        }
        // Flush and signal end of audio — let the reader finish before closing.
        write.send(Message::Text(audio_flush_msg().into())).await?;
        write.send(Message::Text(audio_end_msg().into())).await?;
        Ok(())
    });

    // Reader task: accumulates transcript from text deltas
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Mistral RT streaming: timed out waiting for transcription"
                    ));
                }
            };

            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

                if msg_type == "error" {
                    return Err(anyhow::anyhow!(
                        "Mistral RT error: {}",
                        extract_error(&resp)
                    ));
                }

                if msg_type == "transcription.text.delta" {
                    if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                        final_text.push_str(t);
                        if let Some(tx) = &delta_tx {
                            let _ = tx.send(final_text.clone());
                        }
                    }
                }

                if msg_type == "transcription.done" {
                    if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                        if !t.is_empty() {
                            final_text = t.to_string();
                        }
                    }
                    break;
                }
            }
        }

        debug!("Mistral RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read messages from the WebSocket until we get `session.created` or an error.
async fn wait_for_session_created<S>(read: &mut S) -> Result<()>
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => {
                return Err(anyhow::anyhow!(
                    "Mistral RT: connection closed before session.created"
                ));
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Mistral RT: timed out waiting for session.created"
                ));
            }
        };

        if let Message::Text(text) = msg {
            let resp: serde_json::Value = serde_json::from_str(&text)?;
            let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if msg_type == "error" {
                return Err(anyhow::anyhow!(
                    "Mistral RT error: {}",
                    extract_error(&resp)
                ));
            }

            if msg_type == "session.created" {
                debug!("Mistral RT: session created");
                return Ok(());
            }
        }
    }
}

/// Build a `session.update` message from the provider options (e.g. language hint).
fn build_session_update(opts: &serde_json::Value) -> Option<String> {
    // Mistral supports language hints via session update
    let mut payload = serde_json::Map::new();

    if let Some(hints) = opts.get("language_hints").and_then(|v| v.as_array()) {
        // Mistral uses ISO 639-1 codes; take the first hint's base language
        let lang = hints
            .iter()
            .filter_map(|v| v.as_str())
            .map(|lang| lang.split('-').next().unwrap_or(lang))
            .next();
        if let Some(lang) = lang {
            payload.insert("language".to_string(), serde_json::json!(lang));
        }
    }

    if payload.is_empty() {
        return None;
    }

    Some(
        serde_json::json!({
            "type": "session.update",
            "session": payload,
        })
        .to_string(),
    )
}
