pub mod doubao;
pub mod realtime;
pub mod soniox;

pub(crate) async fn check_response(
    response: reqwest::Response,
    context: &str,
) -> anyhow::Result<reqwest::Response> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("{} ({}): {}", context, status, body));
    }
    Ok(response)
}

/// Generate a minimal silent WAV clip for API key validation.
pub(crate) fn test_silence_wav() -> anyhow::Result<Vec<u8>> {
    crate::audio_toolkit::audio::encode_wav_bytes(&vec![0.0f32; 1600])
}

/// Strip language subtags (e.g. "zh-Hans" -> "zh") for APIs that expect ISO 639-1 codes.
pub(crate) fn strip_lang_subtag(lang: &str) -> &str {
    lang.split('-').next().unwrap_or(lang)
}

pub async fn test_api_key(
    provider_id: &str,
    api_key: &str,
    base_url: &str,
    model: &str,
    options: Option<&serde_json::Value>,
) -> anyhow::Result<()> {
    match provider_id {
        "doubao" => doubao::test_api_key(api_key, base_url, model, options).await,
        "soniox" => soniox::test_api_key(api_key, base_url, model).await,
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
        "doubao" => doubao::transcribe(api_key, base_url, model, audio_wav, options).await,
        "soniox" => soniox::transcribe(api_key, base_url, model, audio_wav, options).await,
        _ => Err(anyhow::anyhow!(
            "Unknown cloud STT provider: {}",
            provider_id
        )),
    }
}
