use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAssistantPanel {
    Accessibility,
    Microphone,
}

#[cfg(target_os = "macos")]
impl PermissionAssistantPanel {
    const fn as_ffi(self) -> i32 {
        match self {
            Self::Accessibility => 0,
            Self::Microphone => 1,
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn permission_assistant_present(panel: i32);
    fn permission_assistant_dismiss();
}

pub fn present(panel: PermissionAssistantPanel) {
    #[cfg(target_os = "macos")]
    unsafe {
        permission_assistant_present(panel.as_ffi());
    }
}

pub fn dismiss() {
    #[cfg(target_os = "macos")]
    unsafe {
        permission_assistant_dismiss();
    }
}
