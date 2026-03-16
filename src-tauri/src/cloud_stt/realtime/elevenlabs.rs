use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::StreamingHandles;

/// ElevenLabs realtime STT WebSocket endpoint.
const ELEVENLABS_WS_URL: &str = "wss://api.elevenlabs.io/v1/speech-to-text/realtime";

/// Chunk size in bytes for sending PCM audio (3840 = 240ms at 16kHz mono s16le).
const CHUNK_SIZE: usize = 3840;

/// Timeout for reading messages from the WebSocket.
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Audio format query parameter: 16-bit PCM at 16kHz.
const AUDIO_FORMAT: &str = "pcm_16000";

/// Build the WebSocket URL with query parameters.
fn build_ws_url(api_key: &str, model: &str, options: Option<&serde_json::Value>) -> String {
    let mut url = format!(
        "{}?model_id={}&audio_format={}&xi-api-key={}",
        ELEVENLABS_WS_URL, model, AUDIO_FORMAT, api_key
    );

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                url.push_str(&format!("&language_code={}", lang));
            }
        }
    }

    url
}

/// Build an `input_audio_chunk` JSON message from raw PCM i16 LE bytes.
fn audio_chunk_message(pcm_bytes: &[u8], commit: bool) -> String {
    let b64 = BASE64.encode(pcm_bytes);
    serde_json::json!({
        "message_type": "input_audio_chunk",
        "audio_base_64": b64,
        "commit": commit,
    })
    .to_string()
}

/// Check a response message for errors. Returns `Err` if the message indicates
/// an error, `Ok(())` otherwise.
fn check_error(resp: &serde_json::Value) -> Result<()> {
    let msg_type = resp
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // ElevenLabs error message types all contain "error" or specific known types
    if msg_type.contains("error")
        || msg_type == "quota_exceeded"
        || msg_type == "rate_limited"
        || msg_type == "queue_overflow"
        || msg_type == "resource_exhausted"
        || msg_type == "session_time_limit_exceeded"
        || msg_type == "chunk_size_exceeded"
    {
        let err_msg = resp
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow::anyhow!(
            "ElevenLabs RT error ({}): {}",
            msg_type,
            err_msg
        ));
    }

    Ok(())
}

/// Test API key by opening a WebSocket connection and waiting for `session_started`.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let url = build_ws_url(api_key, model, None);

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("ElevenLabs RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session_started or error
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                check_error(&resp)?;
                // session_started means auth + model are valid
            }
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow::anyhow!(
                "ElevenLabs RT: timed out waiting for response"
            ));
        }
    }

    let _ = write.send(Message::Close(None)).await;
    Ok(())
}

/// Transcribe a complete WAV audio buffer via the realtime WebSocket.
/// Opens a connection, streams the audio, and collects the final transcript.
pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    debug!(
        "ElevenLabs RT: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    // Use VAD commit strategy so the server auto-commits segments
    let mut url = build_ws_url(api_key, model, options);
    url.push_str("&commit_strategy=vad");

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("ElevenLabs RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session_started
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                check_error(&resp)?;
            }
        }
        Ok(None) => {
            return Err(anyhow::anyhow!(
                "ElevenLabs RT: connection closed before session_started"
            ));
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "ElevenLabs RT: timed out waiting for session_started"
            ));
        }
    }

    // Convert WAV to raw PCM i16 LE bytes for sending.
    // The WAV header is typically 44 bytes; strip it so we send raw PCM.
    let pcm_bytes = wav_to_pcm_i16le(&audio_wav)?;

    // Send audio in chunks (last chunk with commit=true)
    let chunks: Vec<&[u8]> = pcm_bytes.chunks(CHUNK_SIZE).collect();
    let last_idx = chunks.len().saturating_sub(1);
    for (i, chunk) in chunks.iter().enumerate() {
        let commit = i == last_idx;
        let msg = audio_chunk_message(chunk, commit);
        write.send(Message::Text(msg.into())).await?;
    }

    // Collect committed transcripts
    let mut final_text = String::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "ElevenLabs RT: timed out waiting for transcription"
                ));
            }
        };

        match msg {
            Message::Text(text) => {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                check_error(&resp)?;

                let msg_type = resp
                    .get("message_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if msg_type == "committed_transcript"
                    || msg_type == "committed_transcript_with_timestamps"
                {
                    if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                        if !final_text.is_empty() && !t.is_empty() {
                            final_text.push(' ');
                        }
                        final_text.push_str(t);
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    let _ = write.send(Message::Close(None)).await;
    debug!("ElevenLabs RT result: '{}'", final_text);
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
    // Use VAD commit strategy for streaming so transcripts auto-commit on pauses
    let mut url = build_ws_url(api_key, model, options.as_ref());
    url.push_str("&commit_strategy=vad");

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("ElevenLabs RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session_started before spawning tasks
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                check_error(&resp)?;
            }
        }
        Ok(None) => {
            return Err(anyhow::anyhow!(
                "ElevenLabs RT: connection closed before session_started"
            ));
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "ElevenLabs RT: timed out waiting for session_started"
            ));
        }
    }

    // Sender task: reads audio frames from the channel, converts to i16 LE PCM,
    // sends as base64-encoded input_audio_chunk messages.
    let sender_handle = tokio::spawn(async move {
        while let Some(frame) = audio_rx.recv().await {
            let pcm_bytes: Vec<u8> = frame
                .iter()
                .map(|&s| {
                    let clamped = s.clamp(-1.0, 1.0);
                    (clamped * i16::MAX as f32) as i16
                })
                .flat_map(|s| s.to_le_bytes())
                .collect();

            for chunk in pcm_bytes.chunks(CHUNK_SIZE) {
                let msg = audio_chunk_message(chunk, false);
                write.send(Message::Text(msg.into())).await?;
            }
        }
        // Audio channel closed — send a final commit chunk to flush any buffered audio,
        // then close the WebSocket so the server sends remaining results.
        let msg = audio_chunk_message(&[], true);
        write.send(Message::Text(msg.into())).await?;
        // Don't send Close here; let the reader finish collecting results first.
        Ok(())
    });

    // Reader task: accumulates committed transcript segments, sends deltas for live preview.
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "ElevenLabs RT streaming: timed out waiting for transcription"
                    ));
                }
            };

            match msg {
                Message::Text(text) => {
                    let resp: serde_json::Value = serde_json::from_str(&text)?;
                    check_error(&resp)?;

                    let msg_type = resp
                        .get("message_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    match msg_type {
                        "committed_transcript" | "committed_transcript_with_timestamps" => {
                            if let Some(t) = resp.get("text").and_then(|v| v.as_str()) {
                                if !final_text.is_empty() && !t.is_empty() {
                                    final_text.push(' ');
                                }
                                final_text.push_str(t);
                            }
                            // Show final text for immediate feedback
                            if let Some(tx) = &delta_tx {
                                let _ = tx.send(final_text.clone());
                            }
                        }
                        "partial_transcript" => {
                            // Show final + partial for live preview
                            if let Some(tx) = &delta_tx {
                                let partial =
                                    resp.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                let mut display = final_text.clone();
                                if !display.is_empty() && !partial.is_empty() {
                                    display.push(' ');
                                }
                                display.push_str(partial);
                                let _ = tx.send(display);
                            }
                        }
                        _ => {}
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        debug!("ElevenLabs RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}

/// Convert WAV bytes to raw PCM i16 little-endian bytes.
/// Assumes the input is a standard WAV with a 44-byte header, 16kHz, mono, 16-bit.
/// If the input is too short or doesn't look like WAV, returns the raw bytes as-is
/// (the server may still handle it depending on the audio_format setting).
fn wav_to_pcm_i16le(wav: &[u8]) -> Result<Vec<u8>> {
    // Standard WAV header is 44 bytes; look for "RIFF" magic
    if wav.len() > 44 && &wav[0..4] == b"RIFF" {
        // Find the "data" sub-chunk
        let mut pos = 12; // skip RIFF header
        while pos + 8 < wav.len() {
            let chunk_id = &wav[pos..pos + 4];
            let chunk_size =
                u32::from_le_bytes([wav[pos + 4], wav[pos + 5], wav[pos + 6], wav[pos + 7]])
                    as usize;
            if chunk_id == b"data" {
                let data_start = pos + 8;
                let data_end = (data_start + chunk_size).min(wav.len());
                return Ok(wav[data_start..data_end].to_vec());
            }
            pos += 8 + chunk_size;
            // Align to even byte boundary
            if pos % 2 != 0 {
                pos += 1;
            }
        }
        // Fallback: skip the standard 44-byte header
        Ok(wav[44..].to_vec())
    } else {
        // Not WAV — assume raw PCM already
        Ok(wav.to_vec())
    }
}
