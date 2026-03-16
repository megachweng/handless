pub mod mistral;
pub mod elevenlabs;
pub mod fireworks;
pub mod groq;
pub mod openai;
pub mod realtime;
pub mod soniox;

pub async fn test_api_key(
    provider_id: &str,
    api_key: &str,
    base_url: &str,
    model: &str,
) -> anyhow::Result<()> {
    match provider_id {
        "mistral" => mistral::test_api_key(api_key, base_url, model).await,
        "openai_stt" => openai::test_api_key(api_key, base_url, model).await,
        "elevenlabs" => elevenlabs::test_api_key(api_key, base_url, model).await,
        "groq" => groq::test_api_key(api_key, base_url, model).await,
        "soniox" => soniox::test_api_key(api_key, base_url, model).await,
        "fireworks" => fireworks::test_api_key(api_key, base_url, model).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider: {}",
            provider_id
        )),
    }
}

pub async fn transcribe(
    provider_id: &str,
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> anyhow::Result<String> {
    match provider_id {
        "mistral" => mistral::transcribe(api_key, base_url, model, audio_wav, options).await,
        "openai_stt" => openai::transcribe(api_key, base_url, model, audio_wav, options).await,
        "elevenlabs" => elevenlabs::transcribe(api_key, base_url, model, audio_wav, options).await,
        "groq" => groq::transcribe(api_key, base_url, model, audio_wav, options).await,
        "soniox" => soniox::transcribe(api_key, base_url, model, audio_wav, options).await,
        "fireworks" => fireworks::transcribe(api_key, base_url, model, audio_wav, options).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider: {}",
            provider_id
        )),
    }
}
