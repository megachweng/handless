use crate::settings::ActivationMode;
use crate::TranscriptionCoordinator;
use log::warn;
use std::time::Instant;
use tauri::{AppHandle, Manager};

pub fn send_transcription_input(app: &AppHandle, binding_id: &str, source: &str) {
    if let Some(coordinator) = app.try_state::<TranscriptionCoordinator>() {
        coordinator.send_input(
            binding_id,
            source,
            true,
            ActivationMode::Toggle,
            Instant::now(),
        );
    } else {
        warn!("TranscriptionCoordinator not initialized");
    }
}