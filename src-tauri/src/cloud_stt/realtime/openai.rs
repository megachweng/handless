use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

use super::StreamingHandles;

/// OpenAI Realtime API WebSocket endpoint for transcription sessions.
const OPENAI_WS_URL: &str = "wss://api.openai.com/v1/realtime";

/// Chunk size in bytes for sending PCM audio (3840 = 80ms at 24kHz mono s16le).
const CHUNK_SIZE: usize = 3840;

/// Timeout for reading messages from the WebSocket.
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// OpenAI Realtime API requires 24kHz mono PCM16 audio.
const OPENAI_SAMPLE_RATE: u32 = 24000;

/// Build the WebSocket URL with query parameters for a transcription session.
fn build_ws_url() -> String {
    format!("{}?intent=transcription", OPENAI_WS_URL)
}

/// Build a WebSocket request with the required headers for the OpenAI Realtime API.
fn build_ws_request(api_key: &str) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
    let url = build_ws_url();
    let mut request = url.into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {}", api_key).parse()?);
    request
        .headers_mut()
        .insert("OpenAI-Beta", "realtime=v1".parse()?);
    Ok(request)
}

/// Build the `transcription_session.update` event to configure the session.
fn build_session_update(
    model: &str,
    input_format: &str,
    options: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut transcription = serde_json::json!({
        "model": model,
    });

    if let Some(opts) = options {
        // Language: accept ISO-639-1 code from options
        if let Some(lang) = opts.get("language").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                transcription["language"] = serde_json::json!(lang);
            }
        }
        // Also support language_hints array (take the first)
        if transcription.get("language").is_none() {
            if let Some(hints) = opts.get("language_hints").and_then(|v| v.as_array()) {
                if let Some(first) = hints.first().and_then(|v| v.as_str()) {
                    let code = first.split('-').next().unwrap_or(first);
                    if !code.is_empty() {
                        transcription["language"] = serde_json::json!(code);
                    }
                }
            }
        }
        // Prompt for style guidance
        if let Some(prompt) = opts.get("prompt").and_then(|v| v.as_str()) {
            if !prompt.is_empty() {
                transcription["prompt"] = serde_json::json!(prompt);
            }
        }
    }

    serde_json::json!({
        "type": "transcription_session.update",
        "session": {
            "input_audio_format": input_format,
            "input_audio_transcription": transcription,
            // Disable turn detection so we control when audio is committed.
            "turn_detection": null,
        }
    })
}

/// Build an `input_audio_buffer.append` event with base64-encoded audio.
fn audio_append_message(pcm_bytes: &[u8]) -> String {
    let b64 = BASE64.encode(pcm_bytes);
    serde_json::json!({
        "type": "input_audio_buffer.append",
        "audio": b64,
    })
    .to_string()
}

/// Check a server event for errors. Returns `Err` if the message is an error event.
fn check_error(resp: &serde_json::Value) -> Result<()> {
    let event_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

    if event_type == "error" {
        let err = resp.get("error").unwrap_or(resp);
        let code = err
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let message = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(anyhow::anyhow!("OpenAI RT error ({}): {}", code, message));
    }

    Ok(())
}

/// Convert WAV bytes to raw PCM i16 little-endian bytes.
/// Strips the WAV header so we send only raw PCM data.
fn wav_to_pcm_i16le(wav: &[u8]) -> Vec<u8> {
    if wav.len() > 44 && &wav[0..4] == b"RIFF" {
        // Find the "data" sub-chunk
        let mut pos = 12;
        while pos + 8 < wav.len() {
            let chunk_id = &wav[pos..pos + 4];
            let chunk_size =
                u32::from_le_bytes([wav[pos + 4], wav[pos + 5], wav[pos + 6], wav[pos + 7]])
                    as usize;
            if chunk_id == b"data" {
                let data_start = pos + 8;
                let data_end = (data_start + chunk_size).min(wav.len());
                return wav[data_start..data_end].to_vec();
            }
            pos += 8 + chunk_size;
            if pos % 2 != 0 {
                pos += 1;
            }
        }
        // Fallback: skip the standard 44-byte header
        wav[44..].to_vec()
    } else {
        wav.to_vec()
    }
}

/// Resample PCM i16 LE audio from `from_rate` to `to_rate` using linear interpolation.
fn resample_i16le(pcm: &[u8], from_rate: u32, to_rate: u32) -> Vec<u8> {
    if from_rate == to_rate {
        return pcm.to_vec();
    }

    let samples: Vec<i16> = pcm
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();

    if samples.is_empty() {
        return Vec::new();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((samples.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(out_len * 2);

    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let sample = if idx + 1 < samples.len() {
            let s0 = samples[idx] as f64;
            let s1 = samples[idx + 1] as f64;
            (s0 + frac * (s1 - s0)) as i16
        } else {
            samples[samples.len() - 1]
        };

        output.extend_from_slice(&sample.to_le_bytes());
    }

    output
}

/// Test API key by opening a WebSocket connection and checking for auth errors.
/// A successful connection + session configuration acceptance validates the key and model.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let request = build_ws_request(api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("OpenAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Send session update to configure transcription mode
    let session_update = build_session_update(model, "pcm16", None);
    write
        .send(Message::Text(session_update.to_string().into()))
        .await?;

    // Read the first response to check for auth/model errors
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(msg)) => {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                check_error(&resp)?;
            }
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow::anyhow!("OpenAI RT: timed out waiting for response"));
        }
    }

    let _ = write.send(Message::Close(None)).await;
    Ok(())
}

/// Transcribe a complete WAV audio buffer via the Realtime WebSocket.
/// Opens a connection, streams the audio, and collects the final transcript.
pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    debug!("OpenAI RT: model={}, audio_size={}", model, audio_wav.len());

    let request = build_ws_request(api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("OpenAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session.created before configuring
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
                "OpenAI RT: connection closed before session.created"
            ));
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "OpenAI RT: timed out waiting for session.created"
            ));
        }
    }

    // Configure the transcription session
    let session_update = build_session_update(model, "pcm16", options);
    write
        .send(Message::Text(session_update.to_string().into()))
        .await?;

    // Extract raw PCM from WAV and resample from 16kHz to 24kHz
    let pcm_16k = wav_to_pcm_i16le(&audio_wav);
    let pcm_24k = resample_i16le(&pcm_16k, 16000, OPENAI_SAMPLE_RATE);

    // Send audio in chunks as base64-encoded input_audio_buffer.append events
    for chunk in pcm_24k.chunks(CHUNK_SIZE) {
        let msg = audio_append_message(chunk);
        write.send(Message::Text(msg.into())).await?;
    }

    // Commit the audio buffer to trigger transcription
    write
        .send(Message::Text(
            serde_json::json!({"type": "input_audio_buffer.commit"})
                .to_string()
                .into(),
        ))
        .await?;

    // Collect the transcript from delta and completed events
    let mut final_text = String::new();
    let mut got_completed = false;

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow::anyhow!(
                    "OpenAI RT: timed out waiting for transcription"
                ));
            }
        };

        if let Message::Text(text) = msg {
            let resp: serde_json::Value = serde_json::from_str(&text)?;
            check_error(&resp)?;

            let event_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match event_type {
                "conversation.item.input_audio_transcription.completed" => {
                    // Use the completed transcript as the authoritative result
                    if let Some(transcript) = resp.get("transcript").and_then(|v| v.as_str()) {
                        final_text = transcript.to_string();
                    }
                    got_completed = true;
                    break;
                }
                "conversation.item.input_audio_transcription.delta" => {
                    // Accumulate deltas as fallback
                    if let Some(delta) = resp.get("delta").and_then(|v| v.as_str()) {
                        final_text.push_str(delta);
                    }
                }
                _ => {}
            }
        }
    }

    if !got_completed {
        debug!("OpenAI RT: did not receive completed event, using accumulated deltas");
    }

    let _ = write.send(Message::Close(None)).await;
    debug!("OpenAI RT result: '{}'", final_text);
    Ok(final_text.trim().to_string())
}

/// Start a streaming WebSocket session. Returns handles for the sender and reader tasks.
/// The caller should drop `audio_rx` (by dropping the sender side) to signal end-of-audio,
/// then await the handles.
///
/// Audio frames arrive as f32 samples at 16kHz from the recorder. They are converted to
/// i16 PCM, resampled to 24kHz (required by OpenAI), base64-encoded, and sent as
/// `input_audio_buffer.append` events.
pub async fn start_streaming(
    api_key: &str,
    model: &str,
    mut audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    options: Option<serde_json::Value>,
    delta_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<StreamingHandles> {
    let request = build_ws_request(api_key)?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow::anyhow!("OpenAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for session.created before configuring
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
                "OpenAI RT: connection closed before session.created"
            ));
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "OpenAI RT: timed out waiting for session.created"
            ));
        }
    }

    // Configure the transcription session with turn_detection disabled
    // so we control when audio is committed.
    let session_update = build_session_update(model, "pcm16", options.as_ref());
    write
        .send(Message::Text(session_update.to_string().into()))
        .await?;

    // Sender task: reads audio frames from the channel, converts to i16 PCM,
    // resamples to 24kHz, base64-encodes, and sends as input_audio_buffer.append events.
    let sender_handle = tokio::spawn(async move {
        while let Some(frame) = audio_rx.recv().await {
            // Convert f32 [-1.0, 1.0] to i16 LE bytes at 16kHz
            let pcm_16k: Vec<u8> = frame
                .iter()
                .map(|&s| {
                    let clamped = s.clamp(-1.0, 1.0);
                    (clamped * i16::MAX as f32) as i16
                })
                .flat_map(|s| s.to_le_bytes())
                .collect();

            // Resample from 16kHz to 24kHz
            let pcm_24k = resample_i16le(&pcm_16k, 16000, OPENAI_SAMPLE_RATE);

            for chunk in pcm_24k.chunks(CHUNK_SIZE) {
                let msg = audio_append_message(chunk);
                write.send(Message::Text(msg.into())).await?;
            }
        }

        // Audio channel closed — commit the audio buffer to trigger final transcription
        write
            .send(Message::Text(
                serde_json::json!({"type": "input_audio_buffer.commit"})
                    .to_string()
                    .into(),
            ))
            .await?;

        // Don't send Close here; let the reader finish collecting results first.
        Ok(())
    });

    // Reader task: accumulates transcript from delta and completed events
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();
        let mut pending_delta = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "OpenAI RT streaming: timed out waiting for transcription"
                    ));
                }
            };

            match msg {
                Message::Text(text) => {
                    let resp: serde_json::Value = serde_json::from_str(&text)?;
                    check_error(&resp)?;

                    let event_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

                    match event_type {
                        "conversation.item.input_audio_transcription.delta" => {
                            if let Some(delta) = resp.get("delta").and_then(|v| v.as_str()) {
                                pending_delta.push_str(delta);
                            }
                            // Show final + pending delta for immediate feedback
                            if let Some(tx) = &delta_tx {
                                let display = format!("{}{}", final_text, pending_delta);
                                let _ = tx.send(display);
                            }
                        }
                        "conversation.item.input_audio_transcription.completed" => {
                            // Use the completed transcript as the authoritative segment
                            if let Some(transcript) =
                                resp.get("transcript").and_then(|v| v.as_str())
                            {
                                if !final_text.is_empty() && !transcript.is_empty() {
                                    final_text.push(' ');
                                }
                                final_text.push_str(transcript);
                            } else {
                                // Fallback to accumulated deltas
                                if !final_text.is_empty() && !pending_delta.is_empty() {
                                    final_text.push(' ');
                                }
                                final_text.push_str(&pending_delta);
                            }
                            pending_delta.clear();

                            if let Some(tx) = &delta_tx {
                                let _ = tx.send(final_text.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        // If we have remaining deltas that were never completed, include them
        if !pending_delta.is_empty() {
            if !final_text.is_empty() {
                final_text.push(' ');
            }
            final_text.push_str(&pending_delta);
        }

        debug!("OpenAI RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}
