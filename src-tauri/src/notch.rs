use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int};

extern "C" {
    fn notch_indicator_init();
    fn notch_indicator_update_state(state: c_int);
    fn notch_indicator_update_audio_level(level: c_float);
    fn notch_indicator_update_streaming_text(text: *const c_char);
    fn notch_indicator_destroy();
}

/// Notch indicator states matching the Swift-side constants.
#[repr(i32)]
pub enum NotchState {
    Hidden = 0,
    Recording = 1,
    Transcribing = 2,
}

/// Create the native notch indicator panel. Call once during app setup.
pub fn init() {
    unsafe { notch_indicator_init() }
}

/// Transition the notch indicator to the given state.
pub fn update_state(state: NotchState) {
    unsafe { notch_indicator_update_state(state as c_int) }
}

/// Forward the current microphone audio level (0.0..1.0) so the recording
/// dot and waveform animate in real time. Called ~30 times/second.
pub fn update_audio_level(level: f32) {
    unsafe { notch_indicator_update_audio_level(level) }
}

/// Append streaming transcription text to the notch indicator.
/// The notch expands to show the live transcript.
pub fn update_streaming_text(text: &str) {
    if let Ok(cstr) = CString::new(text) {
        unsafe { notch_indicator_update_streaming_text(cstr.as_ptr()) }
    }
}

/// Tear down the panel. Optional safety net for app quit.
#[allow(dead_code)]
pub fn destroy() {
    unsafe { notch_indicator_destroy() }
}
