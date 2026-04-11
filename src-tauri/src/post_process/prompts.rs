use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct LLMPrompt {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

pub const BUILTIN_PROMPT_PREFIX: &str = "default_";
pub const BUILTIN_PROMPT_CORRECT: &str = "default_correct";
const BUILTIN_PROMPT_IMPROVE: &str = "default_improve";
const BUILTIN_PROMPT_RESTRUCTURE: &str = "default_restructure";
const LEGACY_BUILTIN_PROMPT_IMPROVE_TRANSCRIPTIONS: &str = "default_improve_transcriptions";

pub fn is_builtin_prompt(id: &str) -> bool {
    id.starts_with(BUILTIN_PROMPT_PREFIX)
}

pub fn default_prompts() -> Vec<LLMPrompt> {
    vec![
        LLMPrompt {
            id: BUILTIN_PROMPT_CORRECT.to_string(),
            name: "Mild - Correct Transcript".to_string(),
            prompt: include_str!("prompts/correct.txt").to_string(),
        },
        LLMPrompt {
            id: BUILTIN_PROMPT_IMPROVE.to_string(),
            name: "Medium - Improve Fluency".to_string(),
            prompt: include_str!("prompts/fluent.txt").to_string(),
        },
        LLMPrompt {
            id: BUILTIN_PROMPT_RESTRUCTURE.to_string(),
            name: "Aggressive - Restructure & Format".to_string(),
            prompt: include_str!("prompts/restructure.txt").to_string(),
        },
    ]
}

pub fn default_selected_prompt_id() -> Option<String> {
    Some(BUILTIN_PROMPT_CORRECT.to_string())
}

pub fn normalized_prompts(stored_prompts: &[LLMPrompt]) -> Vec<LLMPrompt> {
    let mut prompts = default_prompts();

    for prompt in stored_prompts {
        if is_builtin_prompt(&prompt.id) {
            continue;
        }

        prompts.push(prompt.clone());
    }

    prompts
}

fn remap_prompt_reference(prompt_id: &str) -> Option<String> {
    match prompt_id {
        LEGACY_BUILTIN_PROMPT_IMPROVE_TRANSCRIPTIONS => Some(BUILTIN_PROMPT_CORRECT.to_string()),
        _ => None,
    }
}

pub fn ensure_prompt_defaults(settings: &mut AppSettings) -> bool {
    let normalized_prompts = normalized_prompts(&settings.post_process_prompts);
    let mut changed = settings.post_process_prompts != normalized_prompts;

    if changed {
        settings.post_process_prompts = normalized_prompts;
    }

    if let Some(selected_prompt_id) = settings.post_process_selected_prompt_id.clone() {
        if let Some(remapped_prompt_id) = remap_prompt_reference(&selected_prompt_id) {
            if settings.post_process_selected_prompt_id != Some(remapped_prompt_id.clone()) {
                settings.post_process_selected_prompt_id = Some(remapped_prompt_id);
                changed = true;
            }
        }
    }

    for binding in settings.bindings.values_mut() {
        let Some(prompt_id) = binding.post_process_prompt_id.clone() else {
            continue;
        };

        if let Some(remapped_prompt_id) = remap_prompt_reference(&prompt_id) {
            if binding.post_process_prompt_id != Some(remapped_prompt_id.clone()) {
                binding.post_process_prompt_id = Some(remapped_prompt_id);
                changed = true;
            }
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prompts_keep_stable_builtin_ids() {
        let ids = default_prompts()
            .into_iter()
            .map(|prompt| prompt.id)
            .collect::<Vec<_>>();

        assert!(ids.iter().any(|id| id == "default_correct"));
        assert!(ids.iter().any(|id| id == "default_improve"));
        assert!(ids.iter().any(|id| id == "default_restructure"));
    }

    #[test]
    fn ensure_prompt_defaults_remaps_legacy_builtin_prompt_references() {
        let mut settings = crate::settings::get_default_settings();
        settings.post_process_selected_prompt_id =
            Some(LEGACY_BUILTIN_PROMPT_IMPROVE_TRANSCRIPTIONS.to_string());
        settings
            .bindings
            .get_mut("transcribe_with_post_process")
            .expect("binding should exist")
            .post_process_prompt_id =
            Some(LEGACY_BUILTIN_PROMPT_IMPROVE_TRANSCRIPTIONS.to_string());

        assert!(ensure_prompt_defaults(&mut settings));
        assert_eq!(
            settings.post_process_selected_prompt_id,
            Some(BUILTIN_PROMPT_CORRECT.to_string())
        );
        assert_eq!(
            settings
                .bindings
                .get("transcribe_with_post_process")
                .and_then(|binding| binding.post_process_prompt_id.clone()),
            Some(BUILTIN_PROMPT_CORRECT.to_string())
        );
    }
}
