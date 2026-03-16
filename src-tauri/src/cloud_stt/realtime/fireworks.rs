use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket endpoint for fireworks-asr-large (v1).
const FIREWORKS_WS_URL_V1: &str =
    "wss://audio-streaming.api.fireworks.ai/v1/audio/transcriptions/streaming";

/// WebSocket endpoint for fireworks-asr-v2 (v2).
const FIREWORKS_WS_URL_V2: &str =
    "wss://audio-streaming-v2.api.fireworks.ai/v1/audio/transcriptions/streaming";

/// 50ms chunks at 16kHz mono 16-bit = 1600 bytes.
const CHUNK_SIZE: usize = 1600;
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Choose the WebSocket URL based on the model name.
fn ws_url_for_model(model: &str) -> &'static str {
    if model.contains("v2") {
        FIREWORKS_WS_URL_V2
    } else {
        FIREWORKS_WS_URL_V1
    }
}

/// Build the full WebSocket URL with query parameters.
fn build_ws_url(model: &str, options: Option<&serde_json::Value>) -> String {
    let base = ws_url_for_model(model);
    let mut params: Vec<(String, String)> =
        vec![("response_format".to_string(), "verbose_json".to_string())];

    if let Some(opts) = options {
        // Language hints — take the first one as the primary language.
        if let Some(hints) = opts.get("language_hints").and_then(|v| v.as_array()) {
            if let Some(first) = hints.first().and_then(|v| v.as_str()) {
                let code = first.split('-').next().unwrap_or(first);
                params.push(("language".to_string(), code.to_string()));
            }
        }
    }

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, utf8_percent_encode(v, NON_ALPHANUMERIC)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", base, query)
}

/// Build a tungstenite HTTP request with the Authorization header.
fn build_ws_request(
    url: &str,
    api_key: &str,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
    use tokio_tungstenite::tungstenite::http::Request;
    let req = Request::builder()
        .uri(url)
        .header("Authorization", api_key)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .header("Host", extract_host(url))
        .body(())?;
    Ok(req)
}

/// Extract the host from a URL string.
fn extract_host(url: &str) -> String {
    url.replace("wss://", "")
        .replace("ws://", "")
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Build the JSON message signaling end of audio stream.
fn end_of_stream_message() -> String {
    serde_json::json!({
        "event_id": "streaming_complete",
        "object": "stt.input.trace",
        "trace_id": "final"
    })
    .to_string()
}

/// Collect the full transcript text from the segment state map.
fn collect_transcript(segments: &BTreeMap<i64, String>) -> String {
    segments
        .values()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

/// Parse a Fireworks response message and update the segment state.
/// Returns `true` if this was the final trace (stream complete).
fn parse_response(text: &str, segments: &mut BTreeMap<i64, String>) -> Result<bool> {
    let resp: serde_json::Value = serde_json::from_str(text)?;

    // Check for errors.
    if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
        return Err(anyhow::anyhow!("Fireworks RT error: {}", err));
    }

    // Handle trace messages (used for end-of-stream signaling).
    if resp.get("object").and_then(|v| v.as_str()) == Some("stt.output.trace") {
        if resp.get("trace_id").and_then(|v| v.as_str()) == Some("final") {
            return Ok(true);
        }
        return Ok(false);
    }

    // Handle state-cleared acknowledgments.
    if resp.get("object").and_then(|v| v.as_str()) == Some("stt.state.cleared") {
        return Ok(false);
    }

    // Update segment state from the response.
    if let Some(segs) = resp.get("segments").and_then(|v| v.as_array()) {
        for seg in segs {
            if let (Some(id), Some(seg_text)) = (
                seg.get("id").and_then(|v| v.as_i64()),
                seg.get("text").and_then(|v| v.as_str()),
            ) {
                segments.insert(id, seg_text.to_string());
            }
        }
    }

    Ok(false)
}

/// Test API key by opening a WebSocket connection and checking for errors.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let url = build_ws_url(model, None);
    let request = build_ws_request(&url, api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Fireworks RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Send end-of-stream immediately so the server responds.
    write
        .send(Message::Text(end_of_stream_message().into()))
        .await?;

    // Read the first response to check for auth/model errors.
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
                    return Err(anyhow::anyhow!("Fireworks RT error: {}", err));
                }
            }
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Fireworks RT: timed out waiting for response"
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
        "Fireworks RT: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    let url = build_ws_url(model, options);
    let request = build_ws_request(&url, api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Fireworks RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Send audio as binary frames.
    // Skip WAV header (first 44 bytes) to send raw PCM data.
    let pcm_data = if audio_wav.len() > 44 && &audio_wav[..4] == b"RIFF" {
        &audio_wav[44..]
    } else {
        &audio_wav
    };

    for chunk in pcm_data.chunks(CHUNK_SIZE) {
        write.send(Message::Binary(chunk.to_vec().into())).await?;
    }

    // Signal end of audio.
    write
        .send(Message::Text(end_of_stream_message().into()))
        .await?;

    let mut segments: BTreeMap<i64, String> = BTreeMap::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "Fireworks RT: timed out waiting for transcription"
                ));
            }
        };

        if let Message::Text(text) = msg {
            let is_final = parse_response(&text, &mut segments)?;
            if is_final {
                break;
            }
        }
    }

    let result = collect_transcript(&segments);
    debug!("Fireworks RT result: '{}'", result);
    Ok(result)
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
    let url = build_ws_url(model, options.as_ref());
    let request = build_ws_request(&url, api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("Fireworks RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Sender task: reads audio frames from the channel, converts to i16 LE bytes, sends as binary.
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
        // Signal end of audio — send the final trace so the server knows we are done.
        write
            .send(Message::Text(end_of_stream_message().into()))
            .await?;
        Ok(())
    });

    // Reader task: accumulates segments and streams deltas.
    let reader_handle = tokio::spawn(async move {
        let mut segments: BTreeMap<i64, String> = BTreeMap::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Fireworks RT streaming: timed out waiting for transcription"
                    ));
                }
            };

            if let Message::Text(text) = msg {
                let is_final = parse_response(&text, &mut segments)?;

                // Send current state as a delta for immediate UI feedback.
                if let Some(tx) = &delta_tx {
                    let display = collect_transcript(&segments);
                    let _ = tx.send(display);
                }

                if is_final {
                    break;
                }
            }
        }

        let result = collect_transcript(&segments);
        debug!("Fireworks RT streaming result: '{}'", result);
        Ok(result)
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}
