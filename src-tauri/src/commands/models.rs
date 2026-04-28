use crate::settings::{get_settings, write_settings};
use crate::stt_provider::{cloud_provider_registry, SttProviderInfo};
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
pub async fn get_all_stt_providers() -> Result<Vec<SttProviderInfo>, String> {
    Ok(cloud_provider_registry())
}

#[tauri::command]
#[specta::specta]
pub async fn test_stt_api_key(
    app_handle: AppHandle,
    provider_id: String,
    api_key: String,
    model: String,
    realtime: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app_handle);
    let provider = settings
        .stt_provider(&provider_id)
        .ok_or_else(|| format!("STT provider '{}' not found", provider_id))?;

    let base_url = provider.base_url.clone();
    let cloud_opts: Option<serde_json::Value> = settings
        .stt_cloud_options
        .get(&provider_id)
        .and_then(|s| serde_json::from_str(s).ok());

    if realtime {
        crate::cloud_stt::realtime::test_api_key(
            &provider_id,
            &api_key,
            &model,
            cloud_opts.as_ref(),
        )
        .await
        .map_err(|e| e.to_string())?;
    } else {
        crate::cloud_stt::test_api_key(
            &provider_id,
            &api_key,
            &base_url,
            &model,
            cloud_opts.as_ref(),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    settings.stt_verified_providers.insert(provider_id);
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn change_stt_cloud_options_setting(
    app_handle: AppHandle,
    provider_id: String,
    options: String,
) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(&options)
        .map_err(|e| format!("Invalid JSON options: {}", e))?;
    let mut settings = get_settings(&app_handle);
    settings.stt_cloud_options.insert(provider_id, options);
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_dictionary_terms(
    app_handle: AppHandle,
    terms: Vec<String>,
) -> Result<(), String> {
    let mut settings = get_settings(&app_handle);
    settings.dictionary_terms = terms;
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_dictionary_context(
    app_handle: AppHandle,
    context: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app_handle);
    settings.dictionary_context = context;
    write_settings(&app_handle, settings);
    Ok(())
}
