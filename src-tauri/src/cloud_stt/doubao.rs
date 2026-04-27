use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::{SinkExt, StreamExt};
use log::debug;
use std::io::{Read, Write};
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

// ─── Default endpoint ───────────────────────────────────────────────

pub(crate) const DEFAULT_WS_URL: &str = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async";
const DEFAULT_RESOURCE_ID: &str = "volc.seedasr.sauc.duration";

// ─── Binary protocol constants ──────────────────────────────────────

const PROTOCOL_VERSION: u8 = 0b0001;
const HEADER_SIZE: u8 = 0b0001; // 1 * 4 = 4 bytes

const MSG_FULL_CLIENT_REQUEST: u8 = 0b0001;
const MSG_AUDIO_ONLY: u8 = 0b0010;
const MSG_FULL_SERVER_RESPONSE: u8 = 0b1001;
const MSG_ERROR: u8 = 0b1111;

const FLAG_NONE: u8 = 0b0000;
const FLAG_LAST_PACKET: u8 = 0b0010;

const SERIAL_NONE: u8 = 0b0000;
const SERIAL_JSON: u8 = 0b0001;

const COMPRESS_GZIP: u8 = 0b0001;

const CHUNK_SIZE: usize = 3200; // 100ms at 16kHz 16-bit mono
const WS_READ_TIMEOUT: Duration = Duration::from_secs(30);
const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// ─── Protocol types ─────────────────────────────────────────────────

pub(crate) enum DoubaoEvent {
    Interim(String),
    Definite(String),
    Final(String),
    Error { code: u32, message: String },
}

// ─── Credential extraction ──────────────────────────────────────────

/// Extract the three Doubao credentials:
/// - `access_token` from the main credential field
/// - `app_key` and `resource_id` from the cloud options
pub(crate) fn extract_credentials<'a>(
    api_key: &'a str,
    options: Option<&'a serde_json::Value>,
) -> Result<(&'a str, &'a str, &'a str)> {
    let opts = options.ok_or_else(|| {
        anyhow!("Doubao requires app_key and resource_id. Please fill in the provider options.")
    })?;
    let app_key = opts
        .get("app_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Missing app_key in Doubao options"))?;
    let resource_id = opts
        .get("resource_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_RESOURCE_ID);
    let access_token = api_key;
    Ok((access_token, app_key, resource_id))
}

/// Map app language codes to Doubao API language codes.
/// Doubao requires region-qualified codes (e.g. `zh-CN`, `en-US`).
fn map_language_code(lang: &str) -> &str {
    match lang {
        "zh-Hans" => "zh-CN",
        "zh-Hant" => "zh-CN",
        "en" => "en-US",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        "es" => "es-MX",
        "fr" => "fr-FR",
        "de" => "de-DE",
        "ru" => "ru-RU",
        "pt" => "pt-BR",
        "it" => "it-IT",
        "ar" => "ar-SA",
        "th" => "th-TH",
        "vi" => "vi-VN",
        "id" => "id-ID",
        "ms" => "ms-MY",
        "yue" => "yue-CN",
        "bn" => "bn-BD",
        "el" => "el-GR",
        "nl" => "nl-NL",
        "tr" => "tr-TR",
        "pl" => "pl-PL",
        "ro" => "ro-RO",
        "ne" => "ne-NP",
        "uk" => "uk-UA",
        other => other,
    }
}

// ─── Binary protocol helpers ────────────────────────────────────────

pub(crate) fn build_header(msg_type: u8, flags: u8, serialization: u8, compression: u8) -> [u8; 4] {
    [
        (PROTOCOL_VERSION << 4) | HEADER_SIZE,
        (msg_type << 4) | flags,
        (serialization << 4) | compression,
        0x00,
    ]
}

pub(crate) fn gzip_compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| anyhow!("gzip compress: {e}"))?;
    encoder.finish().map_err(|e| anyhow!("gzip finish: {e}"))
}

pub(crate) fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder
        .read_to_end(&mut buf)
        .map_err(|e| anyhow!("gzip decompress: {e}"))?;
    Ok(buf)
}

pub(crate) fn build_frame(header: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let payload_len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + 4 + payload.len());
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&payload_len.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

/// Build the full client request frame with audio config, model settings,
/// and optional hotwords/ITN/punc/DDC from cloud options.
pub(crate) fn build_full_client_request(
    model: &str,
    options: Option<&serde_json::Value>,
) -> Result<Vec<u8>> {
    let enable_itn = options
        .and_then(|o| o.get("enable_itn"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let enable_punc = options
        .and_then(|o| o.get("enable_punc"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let enable_ddc = options
        .and_then(|o| o.get("enable_ddc"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let language = options
        .and_then(|o| o.get("language"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let mut request = serde_json::json!({
        "model_name": model,
        "enable_itn": enable_itn,
        "enable_punc": enable_punc,
        "enable_ddc": enable_ddc,
        "result_type": "full",
        "show_utterances": true,
    });

    // Build corpus.context: hotwords + dialog context from dictionary injection
    let hotwords: Vec<serde_json::Value> = options
        .and_then(|o| o.get("hotwords"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .map(|w| w.trim())
                .filter(|w| !w.is_empty())
                .map(|w| serde_json::json!({"word": w}))
                .collect()
        })
        .unwrap_or_default();

    let dialog_context = options
        .and_then(|o| o.get("dialog_context"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    if !hotwords.is_empty() || dialog_context.is_some() {
        let mut context_obj = serde_json::Map::new();

        if !hotwords.is_empty() {
            context_obj.insert("hotwords".to_string(), serde_json::json!(hotwords));
        }
        if let Some(ctx) = dialog_context {
            context_obj.insert("context_type".to_string(), serde_json::json!("dialog_ctx"));
            context_obj.insert(
                "context_data".to_string(),
                serde_json::json!([{"text": ctx}]),
            );
        }

        let context_str =
            serde_json::to_string(&serde_json::Value::Object(context_obj)).unwrap_or_default();
        request["corpus"] = serde_json::json!({ "context": context_str });
        debug!(
            "Doubao ASR: {} hotwords, dialog_ctx={} via corpus.context",
            hotwords.len(),
            dialog_context.is_some()
        );
    }

    let mut audio = serde_json::json!({
        "format": "pcm",
        "codec": "raw",
        "rate": 16000,
        "bits": 16,
        "channel": 1,
    });

    if let Some(lang) = language {
        let lang_code = map_language_code(lang);
        audio["language"] = serde_json::json!(lang_code);
    }

    let payload_json = serde_json::json!({
        "user": { "uid": "handless" },
        "audio": audio,
        "request": request,
    });

    debug!(
        "Doubao ASR full client request payload: {}",
        serde_json::to_string_pretty(&payload_json).unwrap_or_default()
    );

    let json_bytes =
        serde_json::to_vec(&payload_json).map_err(|e| anyhow!("serialize request: {e}"))?;
    let compressed = gzip_compress(&json_bytes)?;
    let header = build_header(
        MSG_FULL_CLIENT_REQUEST,
        FLAG_NONE,
        SERIAL_JSON,
        COMPRESS_GZIP,
    );
    Ok(build_frame(header, &compressed))
}

/// Build an audio-only frame. `is_last` sets the last-packet flag to signal
/// end of audio input.
pub(crate) fn build_audio_frame(data: &[u8], is_last: bool) -> Result<Vec<u8>> {
    let compressed = gzip_compress(data)?;
    let flags = if is_last { FLAG_LAST_PACKET } else { FLAG_NONE };
    let header = build_header(MSG_AUDIO_ONLY, flags, SERIAL_NONE, COMPRESS_GZIP);
    Ok(build_frame(header, &compressed))
}

/// Parse a binary server response frame into a typed event.
pub(crate) fn parse_server_response(data: &[u8]) -> Result<DoubaoEvent> {
    if data.len() < 4 {
        return Err(anyhow!("Doubao: frame too short ({} bytes)", data.len()));
    }

    let msg_type = (data[1] >> 4) & 0x0F;
    let flags = data[1] & 0x0F;
    let serialization = (data[2] >> 4) & 0x0F;
    let compression = data[2] & 0x0F;

    match msg_type {
        MSG_FULL_SERVER_RESPONSE => {
            let has_sequence = (flags & 0b0001) != 0;
            let is_last = (flags & 0b0010) != 0;

            let header_bytes = ((data[0] & 0x0F) as usize) * 4;
            let mut offset = header_bytes;

            if has_sequence {
                if data.len() < offset + 4 {
                    return Err(anyhow!("Doubao: missing sequence number"));
                }
                offset += 4;
            }

            if data.len() < offset + 4 {
                return Err(anyhow!("Doubao: missing payload size"));
            }
            let payload_size = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if data.len() < offset + payload_size {
                return Err(anyhow!("Doubao: incomplete payload"));
            }
            let payload_bytes = &data[offset..offset + payload_size];

            let json_bytes = if compression == COMPRESS_GZIP {
                gzip_decompress(payload_bytes)?
            } else {
                payload_bytes.to_vec()
            };

            let json: serde_json::Value = if serialization == SERIAL_JSON {
                serde_json::from_slice(&json_bytes)
                    .map_err(|e| anyhow!("Doubao: parse JSON: {e}"))?
            } else {
                serde_json::Value::Null
            };

            let text = json
                .get("result")
                .and_then(|r| r.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            let has_definite = json
                .get("result")
                .and_then(|r| r.get("utterances"))
                .and_then(|u| u.as_array())
                .map(|utterances| {
                    utterances
                        .iter()
                        .any(|u| u.get("definite").and_then(|d| d.as_bool()).unwrap_or(false))
                })
                .unwrap_or(false);

            if is_last {
                Ok(DoubaoEvent::Final(text))
            } else if has_definite {
                Ok(DoubaoEvent::Definite(text))
            } else {
                Ok(DoubaoEvent::Interim(text))
            }
        }
        MSG_ERROR => {
            let header_bytes = ((data[0] & 0x0F) as usize) * 4;
            let mut offset = header_bytes;

            let error_code = if data.len() >= offset + 4 {
                let code = u32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;
                code
            } else {
                0
            };

            let error_msg = if data.len() >= offset + 4 {
                let msg_size = u32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]) as usize;
                offset += 4;
                if data.len() >= offset + msg_size {
                    String::from_utf8_lossy(&data[offset..offset + msg_size]).to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            Ok(DoubaoEvent::Error {
                code: error_code,
                message: error_msg,
            })
        }
        _ => Err(anyhow!("Doubao: unknown message type: {msg_type:#06b}")),
    }
}

/// Build a WebSocket request with Volcengine authentication headers.
pub(crate) fn build_ws_request(
    ws_url: &str,
    access_key: &str,
    app_key: &str,
    resource_id: &str,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
    let connect_id = uuid::Uuid::new_v4().to_string();
    let mut request = ws_url
        .into_client_request()
        .map_err(|e| anyhow!("invalid WebSocket URL: {e}"))?;
    let headers = request.headers_mut();
    headers.insert(
        "X-Api-App-Key",
        app_key
            .parse()
            .map_err(|_| anyhow!("invalid app_key header value"))?,
    );
    headers.insert(
        "X-Api-Access-Key",
        access_key
            .parse()
            .map_err(|_| anyhow!("invalid access_key header value"))?,
    );
    headers.insert(
        "X-Api-Resource-Id",
        resource_id
            .parse()
            .map_err(|_| anyhow!("invalid resource_id header value"))?,
    );
    headers.insert(
        "X-Api-Connect-Id",
        connect_id
            .parse()
            .map_err(|_| anyhow!("invalid connect_id header value"))?,
    );
    debug!("Doubao ASR: connect_id={}", connect_id);
    Ok(request)
}

// ─── Public API ─────────────────────────────────────────────────────

/// Test API credentials by connecting and sending a minimal request.
pub async fn test_api_key(
    api_key: &str,
    base_url: &str,
    model: &str,
    options: Option<&serde_json::Value>,
) -> Result<()> {
    let (access_key, app_key, resource_id) = extract_credentials(api_key, options)?;
    let ws_url = if base_url.is_empty() {
        DEFAULT_WS_URL
    } else {
        base_url
    };

    let request = build_ws_request(ws_url, access_key, app_key, resource_id)?;

    let (ws_stream, _) = tokio::time::timeout(WS_CONNECT_TIMEOUT, connect_async(request))
        .await
        .map_err(|_| anyhow!("Doubao: connection timed out"))?
        .map_err(|e| anyhow!("Doubao: connection failed: {e}"))?;

    let (mut write, mut read) = ws_stream.split();

    // Send the full client request
    let full_request = build_full_client_request(model, options)?;
    write
        .send(Message::Binary(full_request.into()))
        .await
        .map_err(|e| anyhow!("Doubao: send config: {e}"))?;

    // Send a minimal silence frame with last-packet flag
    let silence_pcm = vec![0u8; 3200]; // 100ms of silence at 16kHz 16-bit
    let audio_frame = build_audio_frame(&silence_pcm, true)?;
    write
        .send(Message::Binary(audio_frame.into()))
        .await
        .map_err(|e| anyhow!("Doubao: send silence: {e}"))?;

    // Read the first response to check for errors
    let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
    match msg {
        Ok(Some(Ok(Message::Binary(data)))) => {
            if let DoubaoEvent::Error { code, message } = parse_server_response(&data)? {
                return Err(anyhow!("Doubao server error {code}: {message}"));
            }
        }
        Ok(Some(Ok(Message::Close(_)))) => {}
        Ok(Some(Ok(_))) => {} // Ignore Text, Ping, Pong, Frame
        Ok(Some(Err(e))) => {
            return Err(anyhow!("Doubao: WebSocket error: {e}"));
        }
        Ok(None) => {}
        Err(_) => {
            return Err(anyhow!("Doubao: timed out waiting for response"));
        }
    }

    let _ = write.send(Message::Close(None)).await;
    debug!("Doubao ASR: API key validation successful");
    Ok(())
}

/// Transcribe audio using the Doubao WebSocket API (batch mode).
/// Sends all audio at once and waits for the final transcript.
pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String> {
    let (access_key, app_key, resource_id) = extract_credentials(api_key, options)?;
    let ws_url = if base_url.is_empty() {
        DEFAULT_WS_URL
    } else {
        base_url
    };

    debug!(
        "Doubao ASR batch: model={}, audio_size={}",
        model,
        audio_wav.len()
    );

    let request = build_ws_request(ws_url, access_key, app_key, resource_id)?;

    let (ws_stream, _) = tokio::time::timeout(WS_CONNECT_TIMEOUT, connect_async(request))
        .await
        .map_err(|_| anyhow!("Doubao: connection timed out"))?
        .map_err(|e| anyhow!("Doubao: connection failed: {e}"))?;

    let (mut write, mut read) = ws_stream.split();

    // Send full client request with options
    let full_request = build_full_client_request(model, options)?;
    write
        .send(Message::Binary(full_request.into()))
        .await
        .map_err(|e| anyhow!("Doubao: send config: {e}"))?;

    // Extract raw PCM from WAV
    let (pcm_samples, _sample_rate) =
        crate::audio_toolkit::audio::extract_pcm_from_wav(&audio_wav)?;
    let pcm_bytes: Vec<u8> = pcm_samples.iter().flat_map(|s| s.to_le_bytes()).collect();

    // Send audio in chunks
    let total_chunks = pcm_bytes.len().div_ceil(CHUNK_SIZE);
    for (i, chunk) in pcm_bytes.chunks(CHUNK_SIZE).enumerate() {
        let is_last = i == total_chunks - 1;
        let audio_frame = build_audio_frame(chunk, is_last)?;
        write
            .send(Message::Binary(audio_frame.into()))
            .await
            .map_err(|e| anyhow!("Doubao: send audio chunk: {e}"))?;
    }

    // If audio was empty, send an empty last-packet
    if pcm_bytes.is_empty() {
        let audio_frame = build_audio_frame(&[], true)?;
        write
            .send(Message::Binary(audio_frame.into()))
            .await
            .map_err(|e| anyhow!("Doubao: send empty last packet: {e}"))?;
    }

    // Read responses until final
    let mut final_text = String::new();

    loop {
        let msg = tokio::time::timeout(WS_READ_TIMEOUT, read.next()).await;
        let msg = match msg {
            Ok(Some(msg)) => msg?,
            Ok(None) => break,
            Err(_) => {
                let _ = write.send(Message::Close(None)).await;
                return Err(anyhow!("Doubao: timed out waiting for transcription"));
            }
        };

        if let Message::Binary(data) = msg {
            match parse_server_response(&data)? {
                DoubaoEvent::Definite(text) | DoubaoEvent::Interim(text) => {
                    if !text.is_empty() {
                        final_text = text;
                    }
                }
                DoubaoEvent::Final(text) => {
                    if !text.is_empty() {
                        final_text = text;
                    }
                    break;
                }
                DoubaoEvent::Error { code, message } => {
                    let _ = write.send(Message::Close(None)).await;
                    return Err(anyhow!("Doubao server error {code}: {message}"));
                }
            }
        } else if let Message::Close(_) = msg {
            break;
        }
    }

    let _ = write.send(Message::Close(None)).await;
    debug!("Doubao ASR batch result: '{}'", final_text);
    Ok(final_text.trim().to_string())
}
