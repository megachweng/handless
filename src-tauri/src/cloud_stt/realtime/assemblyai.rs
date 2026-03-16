use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const WS_URL: &str = "wss://streaming.assemblyai.com/v3/ws";
const CHUNK_SIZE: usize = 3840; // ~120ms at 16kHz 16-bit mono
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Build the WebSocket URL with query parameters.
fn build_ws_url(api_key: &str, model: &str, options: Option<&serde_json::Value>) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    let mut params = vec![
        format!("token={}", utf8_percent_encode(api_key, NON_ALPHANUMERIC)),
        "sample_rate=16000".to_string(),
    ];

    if !model.is_empty() {
        params.push(format!(
            "model={}",
            utf8_percent_encode(model, NON_ALPHANUMERIC)
        ));
    }

    if let Some(opts) = options {
        if let Some(lang) = opts.get("language_code").and_then(|v| v.as_str()) {
            if !lang.is_empty() {
                params.push(format!(
                    "language={}",
                    utf8_percent_encode(lang, NON_ALPHANUMERIC)
                ));
            }
        }
        if let Some(terms) = opts.get("keyterms_prompt").and_then(|v| v.as_array()) {
            for term in terms {
                if let Some(t) = term.as_str() {
                    if !t.is_empty() {
                        params.push(format!(
                            "keyterms={}",
                            utf8_percent_encode(t, NON_ALPHANUMERIC)
                        ));
                    }
                }
            }
        }
    }

    format!("{}?{}", WS_URL, params.join("&"))
}

/// Wait for the "Begin" message from AssemblyAI after WebSocket connection.
async fn wait_for_begin(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Result<()> {
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(Ok(Message::Text(text)))) => {
            let resp: serde_json::Value = serde_json::from_str(&text)?;
            let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if msg_type != "Begin" {
                return Err(anyhow::anyhow!(
                    "AssemblyAI RT: expected Begin, got: {}",
                    text
                ));
            }
            Ok(())
        }
        Ok(Some(Ok(Message::Close(frame)))) => {
            let reason = frame
                .map(|f| format!("code={}, reason={}", f.code, f.reason))
                .unwrap_or_else(|| "unknown".to_string());
            Err(anyhow::anyhow!("AssemblyAI RT auth error: {}", reason))
        }
        Ok(Some(Err(e))) => Err(anyhow::anyhow!("AssemblyAI RT error: {}", e)),
        Ok(None) => Err(anyhow::anyhow!("AssemblyAI RT: connection closed")),
        Err(_) => Err(anyhow::anyhow!(
            "AssemblyAI RT: timed out waiting for response"
        )),
        _ => Err(anyhow::anyhow!("AssemblyAI RT: unexpected message type")),
    }
}

/// Test API key by opening a WebSocket and waiting for the Begin message.
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()> {
    let url = build_ws_url(api_key, model, None);

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("AssemblyAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for Begin message
    wait_for_begin(&mut read).await?;

    // Send terminate and close
    let _ = write
        .send(Message::Text(
            serde_json::json!({"type": "Terminate"}).to_string().into(),
        ))
        .await;
    let _ = write.send(Message::Close(None)).await;
    Ok(())
}

/// Transcribe audio by sending it all via WebSocket and collecting the final transcript.
pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    debug!(
        "AssemblyAI RT: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    let url = build_ws_url(api_key, model, options);

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("AssemblyAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for Begin
    wait_for_begin(&mut read).await?;

    // Strip WAV header — find the "data" chunk
    let pcm_data = wav_to_pcm(&audio_wav)?;

    // Send audio as binary chunks
    for chunk in pcm_data.chunks(CHUNK_SIZE) {
        write.send(Message::Binary(chunk.to_vec().into())).await?;
    }

    // Signal end of audio
    write
        .send(Message::Text(
            serde_json::json!({"type": "Terminate"}).to_string().into(),
        ))
        .await?;

    // Collect final turns
    let mut final_text = String::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => break,
        };

        if let Message::Text(text) = msg {
            let resp: serde_json::Value = serde_json::from_str(&text)?;
            let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match msg_type {
                "Turn" => {
                    let end_of_turn = resp
                        .get("end_of_turn")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if end_of_turn {
                        if let Some(transcript) = resp.get("transcript").and_then(|v| v.as_str()) {
                            if !final_text.is_empty() && !transcript.is_empty() {
                                final_text.push(' ');
                            }
                            final_text.push_str(transcript);
                        }
                    }
                }
                "Termination" => break,
                _ => {}
            }
        }
    }

    debug!("AssemblyAI RT result: '{}'", final_text);
    Ok(final_text.trim().to_string())
}

use super::StreamingHandles;

/// Start a streaming WebSocket session. Returns handles for sender and reader tasks.
pub async fn start_streaming(
    api_key: &str,
    model: &str,
    mut audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    options: Option<serde_json::Value>,
    delta_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<StreamingHandles> {
    let url = build_ws_url(api_key, model, options.as_ref());

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow::anyhow!("AssemblyAI RT connection failed: {}", e))?;
    let (mut write, mut read) = ws_stream.split();

    // Wait for Begin
    wait_for_begin(&mut read).await?;

    // Sender task: reads f32 audio frames, converts to i16 LE bytes, sends as binary
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
        // Signal end of audio
        write
            .send(Message::Text(
                serde_json::json!({"type": "Terminate"}).to_string().into(),
            ))
            .await?;
        Ok(())
    });

    // Reader task: accumulates final turn transcripts
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "AssemblyAI RT streaming: timed out waiting for response"
                    ));
                }
            };

            if let Message::Text(text) = msg {
                let resp: serde_json::Value = serde_json::from_str(&text)?;
                let msg_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match msg_type {
                    "Turn" => {
                        let end_of_turn = resp
                            .get("end_of_turn")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let transcript = resp
                            .get("transcript")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if end_of_turn {
                            if !final_text.is_empty() && !transcript.is_empty() {
                                final_text.push(' ');
                            }
                            final_text.push_str(transcript);
                        }

                        // Send live preview
                        if let Some(tx) = &delta_tx {
                            let display = if end_of_turn {
                                final_text.clone()
                            } else {
                                format!("{} {}", final_text, transcript)
                            };
                            let _ = tx.send(display.trim().to_string());
                        }
                    }
                    "Termination" => break,
                    _ => {}
                }
            }
        }

        debug!("AssemblyAI RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}

/// Extract raw PCM data from WAV bytes by finding the "data" sub-chunk.
fn wav_to_pcm(wav: &[u8]) -> Result<&[u8]> {
    if wav.len() < 44 {
        return Err(anyhow::anyhow!("WAV data too short"));
    }
    // Simple approach: find "data" marker and skip 8 bytes (marker + size)
    for i in 0..wav.len().saturating_sub(4) {
        if &wav[i..i + 4] == b"data" {
            let data_start = i + 8; // skip "data" + 4-byte size
            if data_start <= wav.len() {
                return Ok(&wav[data_start..]);
            }
        }
    }
    // Fallback: skip standard 44-byte header
    Ok(&wav[44..])
}
