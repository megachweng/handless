mod assemblyai;
mod deepgram;
mod doubao;
mod elevenlabs;
mod fireworks;
mod mistral;
mod openai;
mod session;
mod soniox;

pub use session::{RealtimeStreamingSession, SessionConfig, StreamingHandles};

pub async fn test_api_key(
    provider_id: &str,
    api_key: &str,
    model: &str,
    options: Option<&serde_json::Value>,
) -> anyhow::Result<()> {
    match provider_id {
        "assemblyai" => assemblyai::test_api_key(api_key, model).await,
        "deepgram" => deepgram::test_api_key(api_key, model).await,
        "doubao" => doubao::test_api_key(api_key, model, options).await,
        "elevenlabs" => elevenlabs::test_api_key(api_key, model).await,
        "fireworks" => fireworks::test_api_key(api_key, model).await,
        "mistral" => mistral::test_api_key(api_key, model).await,
        "openai_stt" => openai::test_api_key(api_key, model).await,
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
        "assemblyai" => assemblyai::transcribe(api_key, model, audio_wav, options).await,
        "deepgram" => deepgram::transcribe(api_key, model, audio_wav, options).await,
        "doubao" => doubao::transcribe(api_key, model, audio_wav, options).await,
        "elevenlabs" => elevenlabs::transcribe(api_key, model, audio_wav, options).await,
        "fireworks" => fireworks::transcribe(api_key, model, audio_wav, options).await,
        "mistral" => mistral::transcribe(api_key, model, audio_wav, options).await,
        "openai_stt" => openai::transcribe(api_key, model, audio_wav, options).await,
        "soniox" => soniox::transcribe(api_key, model, audio_wav, options).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider for realtime: {}",
            provider_id
        )),
    }
}
