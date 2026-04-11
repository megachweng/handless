---
name: add-cloud-stt-provider
description: Add a new cloud speech-to-text provider to Handless. Use when integrating a new cloud STT API (e.g., Google Cloud STT, Azure Speech, Rev AI). Covers Rust backend, provider registration, settings defaults, dictionary injection, realtime WebSocket support, and i18n across all 17 locales.
---

# Add Cloud STT Provider

This skill walks through every file and code change needed to add a new cloud speech-to-text provider to Handless. Follow every step exactly — skipping any step will cause compilation errors or runtime failures.

Before starting, research the provider's API documentation to understand:

- Authentication method (usually Bearer token)
- Transcription endpoint(s) and request format
- Response shape (where the transcribed text lives)
- Available parameters (language, model variants, etc.)
- Supported languages (ISO 639-1 codes)
- Whether it supports streaming/realtime via WebSocket
- Whether it's OpenAI-compatible (same endpoint format, multipart form, response shape)

## Step 1: Create the provider module

Create `src-tauri/src/cloud_stt/<provider_name>.rs`.

### OpenAI-compatible shortcut

If the provider uses an OpenAI-compatible API (same `/audio/transcriptions` endpoint, same multipart form fields, same `{ "text": "..." }` response — like Groq), you can skip writing a full module and just re-export:

```rust
/// <Provider>'s STT API is fully OpenAI-compatible.
/// We delegate directly to the OpenAI implementation since `base_url` already differentiates them.
pub use super::openai::{test_api_key, transcribe};
```

This works because `base_url` is passed as a parameter and distinguishes the provider. Only use this if the API is truly identical — if there are any extra parameters (like Fireworks' `diarize` field), write a dedicated module.

### Full implementation

Every provider must implement exactly two public async functions with these signatures:

```rust
pub async fn test_api_key(api_key: &str, base_url: &str, model: &str) -> Result<()>

pub async fn transcribe(
    api_key: &str,
    base_url: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String>
```

### Shared helpers (use these, don't rewrite)

The following helpers are defined in `src-tauri/src/cloud_stt/mod.rs` — use them instead of writing inline equivalents:

- **`super::test_silence_wav()?`** — generates a minimal silent WAV for API key validation. Use instead of `crate::audio_toolkit::audio::encode_wav_bytes(&vec![0.0f32; 1600])?`
- **`super::check_response(response, "Provider error context").await?`** — checks HTTP status and returns a formatted error with status code and body. Use instead of the inline `if !response.status().is_success() { ... }` pattern
- **`super::TranscriptionResponse`** — shared `{ text: String }` deserialization struct. Use for providers whose response has a top-level `text` field (OpenAI, Fireworks, Cartesia, ElevenLabs, Mistral)
- **`super::strip_lang_subtag(lang)`** — strips language subtags like `"zh-Hans"` → `"zh"`. Use for providers that expect ISO 639-1 codes without subtags

### test_api_key

Validates credentials by sending a minimal request. Pattern:

1. Generate a tiny silent WAV: `super::test_silence_wav()?`
2. Send it to the provider's API with the given `api_key` and `model`
3. Use `super::check_response(response, "Provider test error").await?` to validate the response

### transcribe

Transcribes real audio. Pattern:

1. `audio_wav` is already WAV-encoded bytes — send directly
2. Extract provider-specific options from `options: Option<&serde_json::Value>` using `.get("key").and_then(|v| v.as_str())` etc.
3. Make the API call using `reqwest::Client`
4. Use `super::check_response(...)` for error handling
5. Deserialize the response (use `super::TranscriptionResponse` if applicable) and return the transcribed text
6. Use `debug!()` for logging request params and results

### Reference implementations

- **Simple multipart POST** — `openai.rs`: single POST to `/audio/transcriptions`
- **Multi-step (upload → create → poll)** — `assemblyai.rs`: upload audio, create transcript, poll until done
- **Binary body (not multipart)** — `deepgram.rs`: raw audio body with query params
- **Custom auth header** — `elevenlabs.rs`: uses `xi-api-key` header instead of Bearer
- **Re-export** — `groq.rs`: 3-line file delegating to OpenAI

## Step 2: Register in the dispatch module

Edit `src-tauri/src/cloud_stt/mod.rs`.

1. Add the module declaration at the top:

```rust
pub mod <provider_name>;
```

2. Add match arms in both functions:

```rust
// In test_api_key():
"<provider_id>" => <provider_name>::test_api_key(api_key, base_url, model).await,

// In transcribe():
"<provider_id>" => <provider_name>::transcribe(api_key, base_url, model, audio_wav, options).await,
```

The `provider_id` is a unique string identifier (e.g., `"deepgram"`, `"assemblyai"`). It must be consistent across all files.

## Step 3: Add to the provider registry

Edit `src-tauri/src/stt_provider.rs` — add a new entry to the `cloud_provider_registry()` vec.

```rust
SttProviderInfo {
    id: "<provider_id>".to_string(),
    name: "<Display Name>".to_string(),
    description: "onboarding.cloud.<provider_id>.description".to_string(),
    supported_languages: vec![
        // ISO 639-1 codes the provider supports
        // Use "zh-Hans"/"zh-Hant" for Chinese variants
    ].into_iter().map(String::from).collect(),
    supports_translation: false,        // Can it translate audio to English?
    supports_realtime: false,           // ONLY set true if Step 7 (realtime) is implemented
    is_recommended: false,
    backend: ProviderBackend::Cloud {
        base_url: "<default API base URL>".to_string(),
        default_model: "<default model name>".to_string(),
        console_url: Some("<URL where users get API keys>".to_string()),
    },
    available_options: vec![
        // Provider-specific options rendered in the settings UI.
        // Each option generates a UI control automatically.
        // Use i18n keys for label and description.
        //
        // Option types:
        //   CloudOptionType::Language       — single language dropdown
        //   CloudOptionType::LanguageMulti  — multi-select language picker
        //   CloudOptionType::Text           — freeform text input
        //   CloudOptionType::Number { min, max, step } — numeric slider
        //   CloudOptionType::Boolean        — toggle switch
        //
        // Example:
        // CloudProviderOption {
        //     key: "language".to_string(),
        //     label: "settings.models.cloudProviders.options.language".to_string(),
        //     option_type: CloudOptionType::Language,
        //     description: String::new(),
        // },
    ],
    supports_dictionary_terms: false,   // Set true if provider can use glossary terms
    supports_dictionary_context: false, // Set true if provider can use context text
},
```

### Important notes

- The `key` field in options must match exactly what you read from `options` in the `transcribe()` function
- Reuse existing i18n keys when possible (e.g., `settings.models.cloudProviders.options.language` is already shared)
- **`supports_realtime` must be `false` unless you implement Step 7** — this flag gates UI functionality that routes to WebSocket code. Setting it `true` without a matching realtime client causes runtime errors.

## Step 4: Add settings defaults

Edit `src-tauri/src/settings.rs` — add an entry to `default_stt_providers()`:

```rust
SttProvider {
    id: "<provider_id>".to_string(),
    label: "<Display Name>".to_string(),
    provider_type: SttProviderType::Cloud,
    base_url: "<default API base URL>".to_string(),
    default_model: "<default model name>".to_string(),
},
```

The `base_url` and `default_model` must match the values in `cloud_provider_registry()`.

No other settings changes needed — `default_stt_api_keys()`, `default_stt_cloud_models()`, `default_stt_cloud_options()`, and `ensure_stt_defaults()` all iterate over `default_stt_providers()` automatically.

## Step 5: Add dictionary injection (if supported)

If the provider supports dictionary terms or context, edit the `inject_dictionary()` function in `src-tauri/src/stt_provider.rs`.

Add a new match arm in the `match provider_id { ... }` block. Existing patterns:

- **Prompt-based** (`"openai_stt" | "groq" | "fireworks"`): prepend glossary to a `prompt` text field
- **Keyword array** (`"deepgram"`): merge into a `keyterm` comma-separated field
- **JSON array** (`"assemblyai"`): merge into a `keyterms_prompt` JSON array
- **Comma-separated** (`"mistral"`): merge into a `context_bias` text field
- **Dedicated fields** (`"soniox"`): merge into `context_terms` and `context_description` separately
- **JSON array** (`"elevenlabs"`): merge into a `keyterms` JSON array

If the provider does NOT support dictionary injection, skip this step — the default `_` arm handles unknown providers with a debug log.

## Step 6: Add i18n translations

### 6a. English translations

Edit `src/i18n/locales/en/translation.json`. Add two groups of keys:

**Provider identity** (under `onboarding.cloud`):

```json
"<provider_id>": {
  "name": "<Display Name>",
  "description": "<One-line description of the provider's strengths>"
}
```

**Provider-specific option labels** (under `settings.models.cloudProviders.options`) — only for NEW option keys not already present:

```json
"<optionKey>": "<Label text>",
"<optionKey>Description": "<Help text>"
```

Reuse existing keys when the option is semantically identical to one already defined (e.g., `language`, `prompt`, `temperature`, `enableSpeakerDiarization`).

### 6b. All other locales

Add the **exact same keys** with **English text as placeholder** to all 16 other locale files:

- `src/i18n/locales/ar/translation.json`
- `src/i18n/locales/cs/translation.json`
- `src/i18n/locales/de/translation.json`
- `src/i18n/locales/es/translation.json`
- `src/i18n/locales/fr/translation.json`
- `src/i18n/locales/it/translation.json`
- `src/i18n/locales/ja/translation.json`
- `src/i18n/locales/ko/translation.json`
- `src/i18n/locales/pl/translation.json`
- `src/i18n/locales/pt/translation.json`
- `src/i18n/locales/ru/translation.json`
- `src/i18n/locales/tr/translation.json`
- `src/i18n/locales/uk/translation.json`
- `src/i18n/locales/vi/translation.json`
- `src/i18n/locales/zh/translation.json`
- `src/i18n/locales/zh-TW/translation.json`

### 6c. Verify

Run:

```bash
bun run check:translations
```

This must pass with zero missing/extra keys.

## Step 7: Add realtime support (optional)

Only if the provider supports WebSocket-based streaming transcription. **Skip this entirely if the provider is batch-only.**

### 7a. Create the realtime module

Create `src-tauri/src/cloud_stt/realtime/<provider_name>.rs` with three public functions:

```rust
pub async fn test_api_key(api_key: &str, model: &str) -> Result<()>

pub async fn transcribe(
    api_key: &str,
    model: &str,
    audio_wav: Vec<u8>,
    options: Option<&serde_json::Value>,
) -> Result<String>

pub async fn start_streaming(
    api_key: &str,
    model: &str,
    audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    options: Option<serde_json::Value>,
    delta_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<StreamingHandles>
```

Note: realtime functions do NOT take `base_url` — WebSocket URLs are typically hardcoded constants since they differ from REST base URLs.

### Key implementation details

**WebSocket connection with custom headers:** Use `IntoClientRequest` to build the request — do NOT use raw `Request::builder()` which omits required WebSocket upgrade headers:

```rust
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

fn build_ws_request(api_key: &str, url: &str) -> Result<tungstenite::http::Request<()>> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert("Authorization", format!("Bearer {}", api_key).parse()?);
    Ok(request)
}
```

If a provider requires manually constructed headers (e.g., `Host`, non-standard auth), you may use `Request::builder()` but MUST include `Connection: Upgrade`, `Upgrade: websocket`, `Sec-WebSocket-Version: 13`, and `Sec-WebSocket-Key` (via `tungstenite::handshake::client::generate_key()`). See `realtime/fireworks.rs` or `realtime/mistral.rs` for this pattern.

**Audio format:** Convert f32 frames to i16 LE PCM bytes:

```rust
let bytes: Vec<u8> = frame.iter()
    .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
    .flat_map(|s| s.to_le_bytes())
    .collect();
```

**`start_streaming` pattern:** Spawn two tokio tasks returning `StreamingHandles`:

- **Sender task**: reads f32 audio from `audio_rx`, converts to PCM, sends via WebSocket
- **Reader task**: reads WebSocket messages, accumulates final text, sends live preview via `delta_tx`

**Delta preview:** Send `final_text + interim_text` via `delta_tx` on every message for live UI updates.

### Reference implementations (by protocol type)

- **Binary PCM frames** — `realtime/deepgram.rs`, `realtime/assemblyai.rs`: send raw binary, `CloseStream`/`Terminate` JSON to end
- **Base64 JSON messages** — `realtime/elevenlabs.rs`, `realtime/mistral.rs`, `realtime/openai.rs`: audio encoded as base64 in JSON text frames
- **Segment-based responses** — `realtime/fireworks.rs`: tracks segments by ID in a BTreeMap

### 7b. Register in realtime dispatch

Edit `src-tauri/src/cloud_stt/realtime/mod.rs`:

- Add `mod <provider_name>;`
- Add match arms in both `test_api_key()` and `transcribe()`

### 7c. Register in session dispatch

Edit `src-tauri/src/cloud_stt/realtime/session.rs`:

- Add a match arm in `RealtimeStreamingSession::start()` using the existing macro:

```rust
"<provider_id>" => start_provider!(<provider_name>),
```

### 7d. Set the flag

Set `supports_realtime: true` in the `cloud_provider_registry()` entry (Step 3).

## Step 8: Verify

Run these checks:

```bash
# Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# Clippy
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Rust formatting
cd src-tauri && cargo fmt --check && cd ..

# Translation keys
bun run check:translations

# TypeScript (bindings regenerate during cargo check)
npx tsc --noEmit

# Lint
bun run lint
```

All must pass. The specta bindings (`src/bindings.ts`) auto-regenerate during `cargo check` — no manual changes needed.

## What you do NOT need to change

The following are fully generic and require zero modifications:

- **Frontend components** — `CloudProviderConfigCard.tsx` renders all providers dynamically from the registry
- **Tauri commands** — `test_stt_api_key`, `get_all_stt_providers`, `change_stt_cloud_options_setting` are provider-agnostic
- **TypeScript bindings** — auto-generated by specta from Rust types
- **Model store** — fetches providers via `getAllSttProviders()` command
- **Transcription manager** — dispatches to the correct provider via the `cloud_stt::transcribe()` function
- **Settings persistence** — `ensure_stt_defaults()` automatically picks up new providers from `default_stt_providers()`

## File checklist

| #   | File                                                                                     | Action                                                |
| --- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| 1   | `src-tauri/src/cloud_stt/<provider>.rs`                                                  | Create (or 3-line re-export for OpenAI-compatible)    |
| 2   | `src-tauri/src/cloud_stt/mod.rs`                                                         | Edit (module + match arms)                            |
| 3   | `src-tauri/src/stt_provider.rs`                                                          | Edit (registry entry + optional dictionary injection) |
| 4   | `src-tauri/src/settings.rs`                                                              | Edit (default_stt_providers)                          |
| 5   | `src/i18n/locales/en/translation.json`                                                   | Edit (provider + option keys)                         |
| 6   | `src/i18n/locales/{ar,cs,de,es,fr,it,ja,ko,pl,pt,ru,tr,uk,vi,zh,zh-TW}/translation.json` | Edit (same keys, English placeholder)                 |
| 7   | `src-tauri/src/cloud_stt/realtime/<provider>.rs`                                         | Create (only if realtime)                             |
| 8   | `src-tauri/src/cloud_stt/realtime/mod.rs`                                                | Edit (only if realtime)                               |
| 9   | `src-tauri/src/cloud_stt/realtime/session.rs`                                            | Edit (only if realtime — add match arm)               |
