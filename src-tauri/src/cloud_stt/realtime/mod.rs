mod session;
mod soniox;

pub use session::{RealtimeStreamingSession, SessionConfig, StreamingHandles};

pub async fn test_api_key(provider_id: &str, api_key: &str, model: &str) -> anyhow::Result<()> {
    match provider_id {
        "soniox" => soniox::test_api_key(api_key, model).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider for realtime: {}",
            provider_id
        )),
    }
}

pub async fn transcribe(
    provider_id: &str,
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> anyhow::Result<String> {
    match provider_id {
        "soniox" => soniox::transcribe(api_key, model, audio_wav, options).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider for realtime: {}",
            provider_id
        )),
    }
}
