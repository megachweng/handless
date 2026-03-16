use crate::managers::history::{DailySpeakingStats, HistoryEntry, HistoryManager, HistoryPage};
use crate::settings::RecordingRetentionPeriod;
use std::sync::Arc;
use tauri::{AppHandle, State};

fn parse_retention_period(period: &str) -> Result<RecordingRetentionPeriod, String> {
    match period {
        "never" => Ok(RecordingRetentionPeriod::Never),
        "preserve_limit" => Ok(RecordingRetentionPeriod::PreserveLimit),
        "days3" => Ok(RecordingRetentionPeriod::Days3),
        "weeks2" => Ok(RecordingRetentionPeriod::Weeks2),
        "months3" => Ok(RecordingRetentionPeriod::Months3),
        _ => Err(format!("Invalid retention period: {}", period)),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_history_entries(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<Vec<HistoryEntry>, String> {
    history_manager
        .get_history_entries()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_history_entries_page(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: i64,
    cursor: Option<i64>,
) -> Result<HistoryPage, String> {
    history_manager
        .get_history_entries_page(limit, cursor)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_history_entry_saved(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .toggle_saved_status(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_audio_file_path(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    file_name: String,
) -> Result<String, String> {
    let path = history_manager.get_audio_file_path(&file_name);
    path.to_str()
        .ok_or_else(|| "Invalid file path".to_string())
        .map(|s| s.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .delete_entry(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn update_history_limit(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: usize,
) -> Result<(), String> {
    let mut settings = crate::settings::get_settings(&app);
    settings.history_limit = limit;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_recording_retention_period(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    period: String,
) -> Result<(), String> {
    let retention_period = parse_retention_period(&period)?;

    let mut settings = crate::settings::get_settings(&app);
    settings.recording_retention_period = retention_period;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn preview_retention_cleanup(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    period: String,
    limit: Option<usize>,
) -> Result<usize, String> {
    let retention_period = parse_retention_period(&period)?;
    if retention_period == RecordingRetentionPeriod::Never {
        return Ok(0);
    }

    match retention_period {
        RecordingRetentionPeriod::PreserveLimit => {
            let count_limit = limit.unwrap_or_else(|| crate::settings::get_history_limit(&app));
            history_manager
                .count_entries_affected_by_count(count_limit)
                .map_err(|e| e.to_string())
        }
        _ => history_manager
            .count_entries_affected_by_time(retention_period)
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_speaking_stats(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    from_timestamp: i64,
    to_timestamp: i64,
) -> Result<Vec<DailySpeakingStats>, String> {
    history_manager
        .get_speaking_stats(from_timestamp, to_timestamp)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn clear_speaking_stats(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<(), String> {
    history_manager
        .clear_speaking_stats()
        .map_err(|e| e.to_string())
}
