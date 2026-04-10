use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::StreamingHandles;
use crate::cloud_stt::doubao::{
    build_audio_frame, build_full_client_request, build_ws_request, extract_credentials,
    parse_server_response, DoubaoEvent, DEFAULT_WS_URL,
};

const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_SIZE: usize = 3200;

/// Test API key by delegating to the batch module.
pub async fn test_api_key(
    api_key: &str,
    model: &str,
    options: Option<&serde_json::Value>,
) -> Result<()> {
    crate::cloud_stt::doubao::test_api_key(api_key, DEFAULT_WS_URL, model, options).await
}

/// One-shot WebSocket transcription by delegating to the batch module.
pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    crate::cloud_stt::doubao::transcribe(api_key, DEFAULT_WS_URL, model, audio_wav, options).await
}

/// Start a streaming WebSocket session. Returns handles for sender and reader tasks.
/// The caller drops the `audio_rx` sender side to signal end-of-audio,
/// then awaits the handles.
pub async fn start_streaming(
    api_key: &str,
    model: &str,
    mut audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    options: Option<serde_json::Value>,
    delta_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<StreamingHandles> {
    let (access_key, app_key, resource_id) =
        extract_credentials(api_key, options.as_ref())?;

    let request = build_ws_request(DEFAULT_WS_URL, access_key, app_key, resource_id)?;

    let (ws_stream, _) = tokio::time::timeout(WS_CONNECT_TIMEOUT, connect_async(request))
        .await
        .map_err(|_| anyhow::anyhow!("Doubao RT: connection timed out"))?
        .map_err(|e| anyhow::anyhow!("Doubao RT: connection failed: {e}"))?;

    let (mut write, mut read) = ws_stream.split();

    // Send the full client request with options before spawning tasks
    let full_request = build_full_client_request(model, options.as_ref())?;
    write
        .send(Message::Binary(full_request.into()))
        .await
        .map_err(|e| anyhow::anyhow!("Doubao RT: send config: {e}"))?;

    debug!("Doubao RT: streaming session started");

    // Sender task: reads audio frames from the channel, converts to PCM bytes, sends
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
                let audio_frame = build_audio_frame(chunk, false)?;
                write
                    .send(Message::Binary(audio_frame.into()))
                    .await
                    .map_err(|e| anyhow::anyhow!("Doubao RT: send audio: {e}"))?;
            }
        }
        // Audio channel closed — send last-packet signal
        let last_frame = build_audio_frame(&[], true)?;
        write
            .send(Message::Binary(last_frame.into()))
            .await
            .map_err(|e| anyhow::anyhow!("Doubao RT: send finish: {e}"))?;
        debug!("Doubao RT: sender finished, last packet sent");
        Ok(())
    });

    // Reader task: accumulates definite transcript segments, sends deltas
    let reader_handle = tokio::spawn(async move {
        let mut final_text = String::new();

        loop {
            let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
            let msg = match msg {
                Ok(Some(msg)) => msg?,
                Ok(None) => break,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Doubao RT: timed out waiting for transcription"
                    ));
                }
            };

            if let Message::Binary(data) = msg {
                match parse_server_response(&data)? {
                    DoubaoEvent::Definite(text) => {
                        if !text.is_empty() {
                            final_text = text.clone();
                        }
                        if let Some(tx) = &delta_tx {
                            let _ = tx.send(text);
                        }
                    }
                    DoubaoEvent::Interim(text) => {
                        // Show interim as preview without committing to final_text
                        if let Some(tx) = &delta_tx {
                            let display = if final_text.is_empty() {
                                text
                            } else if text.is_empty() {
                                final_text.clone()
                            } else {
                                // The server typically sends the full accumulated text
                                // in each interim, so just use it as-is
                                text
                            };
                            let _ = tx.send(display);
                        }
                    }
                    DoubaoEvent::Final(text) => {
                        if !text.is_empty() {
                            final_text = text;
                        }
                        break;
                    }
                    DoubaoEvent::Error { code, message } => {
                        return Err(anyhow::anyhow!(
                            "Doubao server error {code}: {message}"
                        ));
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }

        debug!("Doubao RT streaming result: '{}'", final_text);
        Ok(final_text.trim().to_string())
    });

    Ok(StreamingHandles {
        sender_handle,
        reader_handle,
    })
}
