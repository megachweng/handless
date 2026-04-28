#![cfg(feature = "cloud-stt-tests")]
//! Integration tests for the retained Windows cloud STT providers.
//!
//! These tests hit real APIs and require keys. They are gated behind
//! the `cloud-stt-tests` cargo feature so they never run by default.
//!
//! ## Setup
//!
//! Create a `.env` file at the repo root with the keys you want to test:
//!
//! ```text
//! SONIOX_API_KEY=...
//! DOUBAO_ACCESS_KEY=...
//! DOUBAO_APP_KEY=...
//! DOUBAO_RESOURCE_ID=volc.seedasr.sauc.duration
//! ```
//!
//! Place a short WAV file at `src-tauri/tests/fixtures/hello_test.wav`.
//! Tests skip gracefully when keys or the fixture are missing.

use handless_app_lib::cloud_stt;
use handless_app_lib::stt_provider;
use std::sync::Once;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        dotenvy::from_filename("../.env").ok();
        dotenvy::dotenv().ok();
    });
}

fn require_env(name: &str) -> Option<String> {
    init();
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            eprintln!("Skipping: {name} not set");
            None
        }
    }
}

fn require_audio() -> Option<Vec<u8>> {
    let paths = &[
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/hello_test.wav"),
        "tests/fixtures/hello_test.wav",
    ];
    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }
    eprintln!("Skipping: tests/fixtures/hello_test.wav not found");
    None
}

fn assert_transcript_matches_fixture(result: &str) {
    let normalized = result.to_lowercase();
    assert!(
        ["hello", "test"]
            .iter()
            .any(|word| normalized.contains(word)),
        "transcript should contain hello or test, got: \"{result}\""
    );
}

mod soniox {
    use super::*;

    const BASE_URL: &str = "https://api.soniox.com/v1";
    const BATCH_MODEL: &str = "stt-async-v3";
    const REALTIME_MODEL: &str = "stt-rt-v4";

    #[tokio::test]
    async fn batch_default() {
        let Some(key) = require_env("SONIOX_API_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };

        let result = cloud_stt::transcribe("soniox", &key, BASE_URL, BATCH_MODEL, audio, None)
            .await
            .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn batch_with_dictionary() {
        let Some(key) = require_env("SONIOX_API_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };

        let opts = serde_json::json!({ "language_hints": ["en"] });
        let opts = stt_provider::inject_dictionary(
            "soniox",
            Some(opts),
            &["Handless".to_string(), "Tauri".to_string()],
            "A desktop speech-to-text application",
        );
        let result =
            cloud_stt::transcribe("soniox", &key, BASE_URL, BATCH_MODEL, audio, opts.as_ref())
                .await
                .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn realtime_default() {
        let Some(key) = require_env("SONIOX_API_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };

        let result = cloud_stt::realtime::transcribe("soniox", &key, REALTIME_MODEL, audio, None)
            .await
            .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn api_key_validation() {
        let Some(key) = require_env("SONIOX_API_KEY") else {
            return;
        };

        cloud_stt::test_api_key("soniox", &key, BASE_URL, BATCH_MODEL, None)
            .await
            .unwrap();
    }
}

mod doubao {
    use super::*;

    const BASE_URL: &str = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async";
    const MODEL: &str = "bigmodel";
    const DEFAULT_RESOURCE_ID: &str = "volc.seedasr.sauc.duration";

    fn require_options() -> Option<serde_json::Value> {
        let app_key = require_env("DOUBAO_APP_KEY")?;
        let resource_id =
            std::env::var("DOUBAO_RESOURCE_ID").unwrap_or_else(|_| DEFAULT_RESOURCE_ID.to_string());
        Some(serde_json::json!({
            "app_key": app_key,
            "resource_id": resource_id,
            "language": "en",
        }))
    }

    #[tokio::test]
    async fn batch_default() {
        let Some(key) = require_env("DOUBAO_ACCESS_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };
        let Some(opts) = require_options() else {
            return;
        };

        let result = cloud_stt::transcribe("doubao", &key, BASE_URL, MODEL, audio, Some(&opts))
            .await
            .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn batch_with_dictionary() {
        let Some(key) = require_env("DOUBAO_ACCESS_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };
        let Some(opts) = require_options() else {
            return;
        };

        let opts = stt_provider::inject_dictionary(
            "doubao",
            Some(opts),
            &["Handless".to_string(), "Tauri".to_string()],
            "A desktop speech-to-text application",
        );
        let result = cloud_stt::transcribe("doubao", &key, BASE_URL, MODEL, audio, opts.as_ref())
            .await
            .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn realtime_default() {
        let Some(key) = require_env("DOUBAO_ACCESS_KEY") else {
            return;
        };
        let Some(audio) = require_audio() else {
            return;
        };
        let Some(opts) = require_options() else {
            return;
        };

        let result = cloud_stt::realtime::transcribe("doubao", &key, MODEL, audio, Some(&opts))
            .await
            .unwrap();
        assert_transcript_matches_fixture(&result);
    }

    #[tokio::test]
    async fn api_key_validation() {
        let Some(key) = require_env("DOUBAO_ACCESS_KEY") else {
            return;
        };
        let Some(opts) = require_options() else {
            return;
        };

        cloud_stt::test_api_key("doubao", &key, BASE_URL, MODEL, Some(&opts))
            .await
            .unwrap();
    }
}
