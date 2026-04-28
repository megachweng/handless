use crate::audio_toolkit::{apply_custom_words, filter_transcription_output};
use crate::settings::get_settings;
use anyhow::Result;
use log::{debug, info};
use std::sync::Arc;
use tauri::AppHandle;

#[derive(Clone)]
pub struct TranscriptionManager {
    app_handle: Arc<AppHandle>,
}

impl TranscriptionManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        Ok(Self {
            app_handle: Arc::new(app_handle.clone()),
        })
    }

    pub async fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
        let st = std::time::Instant::now();
        debug!("Audio vector length: {}", audio.len());

        if audio.is_empty() {
            debug!("Empty audio vector");
            return Ok(String::new());
        }

        let settings = get_settings(&self.app_handle);
        let wav_bytes = crate::audio_toolkit::audio::encode_wav_bytes(&audio)?;
        let api_key = settings
            .stt_api_keys
            .get(&settings.stt_provider_id)
            .cloned()
            .unwrap_or_default();
        let provider = settings
            .stt_provider(&settings.stt_provider_id)
            .ok_or_else(|| anyhow::anyhow!("STT provider not found"))?;
        let model = settings
            .stt_cloud_models
            .get(&settings.stt_provider_id)
            .cloned()
            .unwrap_or_else(|| provider.default_model.clone());
        let cloud_options: Option<serde_json::Value> = settings
            .stt_cloud_options
            .get(&settings.stt_provider_id)
            .and_then(|s| serde_json::from_str(s).ok());
        let cloud_options = crate::stt_provider::inject_dictionary(
            &settings.stt_provider_id,
            cloud_options,
            &settings.dictionary_terms,
            &settings.dictionary_context,
        );

        let realtime_enabled = settings
            .stt_realtime_enabled
            .get(&settings.stt_provider_id)
            .copied()
            .unwrap_or(false);

        let raw_text = if realtime_enabled {
            crate::cloud_stt::realtime::transcribe(
                &settings.stt_provider_id,
                &api_key,
                &model,
                wav_bytes,
                cloud_options.as_ref(),
            )
            .await?
        } else {
            crate::cloud_stt::transcribe(
                &settings.stt_provider_id,
                &api_key,
                &provider.base_url,
                &model,
                wav_bytes,
                cloud_options.as_ref(),
            )
            .await?
        };

        let corrected_result = if !settings.custom_words.is_empty() {
            apply_custom_words(
                &raw_text,
                &settings.custom_words,
                settings.word_correction_threshold,
            )
        } else {
            raw_text
        };
        let filtered_result = filter_transcription_output(&corrected_result);

        info!("Transcription completed in {}ms", st.elapsed().as_millis());
        if filtered_result.is_empty() {
            info!("Transcription result is empty");
        } else {
            info!("Transcription result: {}", filtered_result);
        }

        Ok(filtered_result)
    }
}
