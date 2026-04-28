use crate::input;
use crate::settings;
use crate::settings::{ActivationMode, OverlayPosition};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};

static OVERLAY_SHOW_GENERATION: AtomicU64 = AtomicU64::new(0);

const OVERLAY_WIDTH: f64 = 320.0;
const OVERLAY_HEIGHT: f64 = 120.0;
const OVERLAY_TOP_OFFSET: f64 = 4.0;
const OVERLAY_BOTTOM_OFFSET: f64 = 40.0;

#[cfg(target_os = "windows")]
fn force_overlay_topmost(overlay_window: &tauri::webview::WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    let overlay_clone = overlay_window.clone();
    let _ = overlay_clone.clone().run_on_main_thread(move || {
        if let Ok(hwnd) = overlay_clone.hwnd() {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn force_overlay_topmost(_overlay_window: &tauri::webview::WebviewWindow) {}

#[cfg(target_os = "windows")]
fn show_overlay_window(overlay_window: &tauri::webview::WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    };

    let overlay_clone = overlay_window.clone();
    let _ = overlay_clone.clone().run_on_main_thread(move || {
        if let Ok(hwnd) = overlay_clone.hwnd() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn show_overlay_window(overlay_window: &tauri::webview::WebviewWindow) {
    let _ = overlay_window.show();
    force_overlay_topmost(overlay_window);
}

struct LogicalBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Serialize)]
struct ShowOverlayPayload<'a> {
    state: &'a str,
    position: &'a str,
    activation_mode: &'a str,
}

fn activation_mode_payload(mode: ActivationMode) -> &'static str {
    match mode {
        ActivationMode::Toggle => "toggle",
        ActivationMode::Hold => "hold",
        ActivationMode::HoldOrToggle => "hold_or_toggle",
    }
}

fn overlay_position_payload(position: OverlayPosition) -> &'static str {
    match position {
        OverlayPosition::Top => "top",
        OverlayPosition::Bottom | OverlayPosition::None => "bottom",
    }
}

fn logical_bounds(monitor: &tauri::Monitor) -> LogicalBounds {
    let scale = monitor.scale_factor();
    LogicalBounds {
        x: monitor.position().x as f64 / scale,
        y: monitor.position().y as f64 / scale,
        width: monitor.size().width as f64 / scale,
        height: monitor.size().height as f64 / scale,
    }
}

fn get_monitor_with_cursor(app_handle: &AppHandle) -> Option<tauri::Monitor> {
    if let Some(mouse_location) = input::get_cursor_position(app_handle) {
        if let Ok(monitors) = app_handle.available_monitors() {
            for monitor in monitors {
                let b = logical_bounds(&monitor);
                let (mx, my) = (mouse_location.0 as f64, mouse_location.1 as f64);

                if mx >= b.x && mx < b.x + b.width && my >= b.y && my < b.y + b.height {
                    return Some(monitor);
                }
            }
        }
    }

    app_handle.primary_monitor().ok().flatten()
}

fn calculate_overlay_position(app_handle: &AppHandle) -> Option<(f64, f64)> {
    let monitor = get_monitor_with_cursor(app_handle)?;
    let b = logical_bounds(&monitor);
    let settings = settings::get_settings(app_handle);

    let x = b.x + (b.width - OVERLAY_WIDTH) / 2.0;
    let y = match settings.overlay_position {
        OverlayPosition::Top => b.y + OVERLAY_TOP_OFFSET,
        OverlayPosition::Bottom | OverlayPosition::None => {
            b.y + b.height - OVERLAY_HEIGHT - OVERLAY_BOTTOM_OFFSET
        }
    };

    Some((x, y))
}

pub fn create_recording_overlay(app_handle: &AppHandle) {
    let position = calculate_overlay_position(app_handle);
    if position.is_none() {
        log::debug!("Failed to determine overlay position, not creating overlay window");
        return;
    }
    let (x, y) = position.unwrap();

    let builder = WebviewWindowBuilder::new(
        app_handle,
        "recording_overlay",
        tauri::WebviewUrl::App("src/overlay/index.html".into()),
    )
    .title("Recording")
    .resizable(false)
    .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
    .position(x, y)
    .shadow(false)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible(false);

    #[cfg(target_os = "windows")]
    let builder = builder.transparent(true).focused(false).focusable(false);

    match builder.build() {
        Ok(window) => {
            force_overlay_topmost(&window);
            log::debug!("Recording overlay created");
        }
        Err(error) => {
            log::error!("Failed to create recording overlay: {}", error);
        }
    }
}

fn show_overlay_state(app_handle: &AppHandle, state: &str) {
    let settings = settings::get_settings(app_handle);
    if !settings.overlay_enabled || settings.overlay_position == OverlayPosition::None {
        return;
    }

    let generation = OVERLAY_SHOW_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    if app_handle.get_webview_window("recording_overlay").is_none() {
        create_recording_overlay(app_handle);
    }

    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
        }
        let _ = window.set_size(tauri::LogicalSize::new(OVERLAY_WIDTH, OVERLAY_HEIGHT));
        #[cfg(target_os = "windows")]
        let _ = window.set_focusable(false);
        show_overlay_window(&window);
        let _ = window.emit(
            "show-overlay",
            ShowOverlayPayload {
                state,
                position: overlay_position_payload(settings.overlay_position),
                activation_mode: activation_mode_payload(settings.activation_mode),
            },
        );
    }

    if state == "processing" {
        let app = app_handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(20));
            if OVERLAY_SHOW_GENERATION.load(Ordering::SeqCst) == generation {
                hide_recording_overlay(&app);
            }
        });
    }
}

pub fn show_recording_overlay(app_handle: &AppHandle) {
    show_overlay_state(app_handle, "recording");
}

pub fn show_transcribing_overlay(app_handle: &AppHandle) {
    show_overlay_state(app_handle, "transcribing");
}

pub fn show_processing_overlay(app_handle: &AppHandle) {
    show_overlay_state(app_handle, "processing");
}

pub fn update_overlay_position(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
            force_overlay_topmost(&window);
        }
    }
}

pub fn hide_recording_overlay(app_handle: &AppHandle) {
    OVERLAY_SHOW_GENERATION.fetch_add(1, Ordering::SeqCst);
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        let _ = window.emit("hide-overlay", ());
        let _ = window.hide();
    }
}

pub fn emit_levels(app_handle: &AppHandle, levels: &Vec<f32>) {
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        let _ = window.emit("mic-level", levels);
    }
}

pub fn update_overlay_activation_mode(app_handle: &AppHandle, mode: ActivationMode) {
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        let _ = window.emit("update-activation-mode", activation_mode_payload(mode));
    }
}

pub fn resize_overlay_window(app_handle: &AppHandle, width: f64, height: f64) {
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
        update_overlay_position(app_handle);
        force_overlay_topmost(&window);
    }
}

pub fn emit_streaming_text(app_handle: &AppHandle, text: &str) {
    if let Some(window) = app_handle.get_webview_window("recording_overlay") {
        let _ = window.emit("streaming-text", text);
    }
}
