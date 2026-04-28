use tauri::{AppHandle, Emitter};

/// Start watching for audio device changes and emit "audio-devices-changed"
/// on the app event bus whenever the set of input devices changes.
/// Uses a lightweight polling thread so the Windows build has no platform
/// audio notification dependency.
pub fn start(app: &AppHandle) {
    use crate::audio_toolkit::audio::list_input_devices;
    use std::collections::HashSet;

    let app = app.clone();
    std::thread::spawn(move || {
        let mut prev: HashSet<String> = list_input_devices()
            .unwrap_or_default()
            .into_iter()
            .map(|info| info.name)
            .collect();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));

            let current: HashSet<String> = list_input_devices()
                .unwrap_or_default()
                .into_iter()
                .map(|info| info.name)
                .collect();

            if current != prev {
                prev = current;
                let _ = app.emit("audio-devices-changed", ());
            }
        }
    });
}
