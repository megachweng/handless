use crate::settings::SoundTheme;
use crate::settings::{self, AppSettings};
use cpal::traits::{DeviceTrait, HostTrait};
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamBuilder, Sink};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub enum SoundType {
    Start,
    Stop,
}

#[derive(Clone)]
struct SoundBuffer {
    channels: u16,
    sample_rate: u32,
    samples: Vec<f32>,
    duration: Duration,
}

struct FeedbackPlayer {
    device_key: Option<String>,
    stream: OutputStream,
    sounds: HashMap<PathBuf, SoundBuffer>,
}

impl FeedbackPlayer {
    fn new(device_key: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            stream: create_output_stream(device_key.as_deref())?,
            device_key,
            sounds: HashMap::new(),
        })
    }

    fn sound_buffer(&mut self, path: &Path) -> Result<SoundBuffer, Box<dyn std::error::Error>> {
        if let Some(buffer) = self.sounds.get(path) {
            return Ok(buffer.clone());
        }

        let buffer = load_sound_buffer(path)?;
        self.sounds.insert(path.to_path_buf(), buffer.clone());
        Ok(buffer)
    }
}

struct PlayRequest {
    path: PathBuf,
    selected_device: Option<String>,
    volume: f32,
    completion: Option<Sender<Result<Duration, String>>>,
}

static FEEDBACK_AUDIO_TX: Lazy<Sender<PlayRequest>> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel::<PlayRequest>();

    thread::spawn(move || {
        let mut player: Option<FeedbackPlayer> = None;

        while let Ok(request) = rx.recv() {
            let result = play_request(
                &mut player,
                request.path,
                request.selected_device,
                request.volume,
            )
            .map_err(|err| err.to_string());

            if let Some(completion) = request.completion {
                let _ = completion.send(result);
            } else if let Err(err) = result {
                error!("Failed to play feedback sound: {err}");
            }
        }
    });

    tx
});

fn resolve_sound_path(
    app: &AppHandle,
    settings: &AppSettings,
    sound_type: SoundType,
) -> Option<PathBuf> {
    let sound_file = get_sound_path(settings, sound_type);
    let base_dir = get_sound_base_dir(settings);
    app.path().resolve(&sound_file, base_dir).ok()
}

fn get_sound_path(settings: &AppSettings, sound_type: SoundType) -> String {
    match (settings.sound_theme, sound_type) {
        (SoundTheme::Custom, SoundType::Start) => "custom_start.wav".to_string(),
        (SoundTheme::Custom, SoundType::Stop) => "custom_stop.wav".to_string(),
        (_, SoundType::Start) => settings.sound_theme.to_start_path(),
        (_, SoundType::Stop) => settings.sound_theme.to_stop_path(),
    }
}

fn get_sound_base_dir(settings: &AppSettings) -> tauri::path::BaseDirectory {
    match settings.sound_theme {
        SoundTheme::Custom => tauri::path::BaseDirectory::AppData,
        _ => tauri::path::BaseDirectory::Resource,
    }
}

pub fn play_feedback_sound(app: &AppHandle, sound_type: SoundType) {
    let settings = settings::get_settings(app);
    if let Some(path) = resolve_sound_path(app, &settings, sound_type) {
        play_sound_async(app, path);
    }
}

pub fn play_feedback_sound_blocking(app: &AppHandle, sound_type: SoundType) {
    let settings = settings::get_settings(app);
    if let Some(path) = resolve_sound_path(app, &settings, sound_type) {
        play_sound_blocking(app, &path);
    }
}

pub fn play_test_sound(app: &AppHandle, sound_type: SoundType) {
    let settings = settings::get_settings(app);
    if let Some(path) = resolve_sound_path(app, &settings, sound_type) {
        play_sound_blocking(app, &path);
    }
}

fn play_sound_async(app: &AppHandle, path: PathBuf) {
    if let Err(err) = play_sound_at_path(app, path.as_path(), false) {
        error!("Failed to play sound '{}': {}", path.display(), err);
    }
}

fn play_sound_blocking(app: &AppHandle, path: &Path) {
    if let Err(e) = play_sound_at_path(app, path, true) {
        error!("Failed to play sound '{}': {}", path.display(), e);
    }
}

fn play_sound_at_path(
    app: &AppHandle,
    path: &Path,
    blocking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = settings::get_settings(app);
    let volume = settings.audio_feedback_volume;
    let selected_device = normalize_device_name(settings.selected_output_device.clone());
    play_cached_audio(path, selected_device, volume, blocking)
}

fn play_cached_audio(
    path: &Path,
    selected_device: Option<String>,
    volume: f32,
    blocking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if blocking {
        let (tx, rx) = mpsc::channel();
        FEEDBACK_AUDIO_TX.send(PlayRequest {
            path: path.to_path_buf(),
            selected_device,
            volume,
            completion: Some(tx),
        })?;
        let duration = rx.recv()?.map_err(std::io::Error::other)?;
        thread::sleep(duration);
    } else {
        FEEDBACK_AUDIO_TX.send(PlayRequest {
            path: path.to_path_buf(),
            selected_device,
            volume,
            completion: None,
        })?;
    }

    Ok(())
}

fn normalize_device_name(device_name: Option<String>) -> Option<String> {
    match device_name.as_deref() {
        Some("default") | Some("Default") | None => None,
        Some(name) => Some(name.to_string()),
    }
}

fn create_output_stream(
    selected_device: Option<&str>,
) -> Result<OutputStream, Box<dyn std::error::Error>> {
    let stream_builder = if let Some(device_name) = selected_device {
        let host = crate::audio_toolkit::get_cpal_host();
        let devices = host.output_devices()?;

        let mut found_device = None;
        for device in devices {
            if device.name()? == device_name {
                found_device = Some(device);
                break;
            }
        }

        match found_device {
            Some(device) => OutputStreamBuilder::from_device(device)?,
            None => {
                warn!("Device '{}' not found, using default device", device_name);
                OutputStreamBuilder::from_default_device()?
            }
        }
    } else {
        debug!("Using default device");
        OutputStreamBuilder::from_default_device()?
    };

    Ok(stream_builder.open_stream()?)
}

fn play_request(
    player: &mut Option<FeedbackPlayer>,
    path: PathBuf,
    selected_device: Option<String>,
    volume: f32,
) -> Result<Duration, Box<dyn std::error::Error>> {
    // Reopen the stream whenever Handless is following the system default
    // output so runtime device switches are picked up on the next cue.
    let recreate_player = if selected_device.is_none() {
        true
    } else {
        player
            .as_ref()
            .is_none_or(|current| current.device_key != selected_device)
    };
    if recreate_player {
        *player = Some(FeedbackPlayer::new(selected_device.clone())?);
    }

    let player = player.as_mut().expect("feedback player initialized");
    let buffer = player.sound_buffer(&path)?;
    let sink = Sink::connect_new(player.stream.mixer());
    sink.set_volume(volume);
    sink.append(SamplesBuffer::new(
        buffer.channels,
        buffer.sample_rate,
        buffer.samples,
    ));
    sink.detach();
    Ok(buffer.duration)
}

fn load_sound_buffer(path: &Path) -> Result<SoundBuffer, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let max_value = ((1_i64 << (spec.bits_per_sample - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / max_value))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    Ok(SoundBuffer {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        duration: Duration::from_secs_f64(
            samples.len() as f64 / (spec.sample_rate as f64 * spec.channels as f64),
        ),
        samples,
    })
}
