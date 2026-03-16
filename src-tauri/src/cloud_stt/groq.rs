/// Groq's STT API is fully OpenAI-compatible (same endpoint, auth, request/response format).
/// We delegate directly to the OpenAI implementation since `base_url` already differentiates them.
pub use super::openai::{test_api_key, transcribe};
