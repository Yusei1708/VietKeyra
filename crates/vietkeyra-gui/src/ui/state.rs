use serde::{Deserialize, Serialize};

/// Typing method selected in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TypingMethod {
    #[default]
    BasicTelex,
    ExpandedTelex,
    Disabled,
}

impl TypingMethod {
    pub fn all_labels() -> &'static [&'static str] {
        &[
            "Telex cơ bản",
            "Telex mở rộng (w -> ư, [ -> ơ, ] -> ư)",
            "Tắt hoàn toàn (Chỉ Tiếng Anh)",
        ]
    }

    pub fn to_index(self) -> u32 {
        match self {
            TypingMethod::BasicTelex => 0,
            TypingMethod::ExpandedTelex => 1,
            TypingMethod::Disabled => 2,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => TypingMethod::BasicTelex,
            1 => TypingMethod::ExpandedTelex,
            _ => TypingMethod::Disabled,
        }
    }
}

/// Shortcut selection for switching input modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShortcutOption {
    #[default]
    SuperSpace,
    CtrlShift,
    AltShift,
    CapsLock,
}

impl ShortcutOption {
    pub fn all_labels() -> &'static [&'static str] {
        &[
            "Super + Space (Khuyến nghị cho Hyprland)",
            "Ctrl + Shift (Chuẩn phổ biến)",
            "Alt + Shift",
            "Caps Lock",
        ]
    }

    pub fn to_index(self) -> u32 {
        match self {
            ShortcutOption::SuperSpace => 0,
            ShortcutOption::CtrlShift => 1,
            ShortcutOption::AltShift => 2,
            ShortcutOption::CapsLock => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => ShortcutOption::SuperSpace,
            1 => ShortcutOption::CtrlShift,
            2 => ShortcutOption::AltShift,
            _ => ShortcutOption::CapsLock,
        }
    }
}

/// Configuration settings model for the GUI control panel.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuiSettings {
    pub is_vietnamese: bool,
    pub typing_method: TypingMethod,
    pub shortcut: ShortcutOption,
    pub modern_tone: bool,
    pub auto_undo_repeat: bool,
    pub backspace_recovery: bool,
    pub autostart_hyprland: bool,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            is_vietnamese: true,
            typing_method: TypingMethod::BasicTelex,
            shortcut: ShortcutOption::SuperSpace,
            modern_tone: true,
            auto_undo_repeat: true,
            backspace_recovery: true,
            autostart_hyprland: true,
        }
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// State controller holding GUI state and recursion-prevention guards.
#[derive(Clone)]
pub struct GuiStateController {
    pub settings: Arc<Mutex<GuiSettings>>,
    /// Guard flag: when true, widget changes originated from an external IPC
    /// notification and should NOT emit outbound commands back to the socket.
    pub is_updating: Arc<AtomicBool>,
}

impl Default for GuiStateController {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiStateController {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(GuiSettings::default())),
            is_updating: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_settings(&self) -> GuiSettings {
        *self.settings.lock().unwrap()
    }

    pub fn set_mode_from_daemon(&self, mode: &str) -> bool {
        self.is_updating.store(true, Ordering::SeqCst);
        let mut cur = self.settings.lock().unwrap();
        let is_vi = mode.trim().eq_ignore_ascii_case("VI");
        cur.is_vietnamese = is_vi;
        self.is_updating.store(false, Ordering::SeqCst);
        is_vi
    }

    pub fn set_guard(&self, val: bool) {
        self.is_updating.store(val, Ordering::SeqCst);
    }

    pub fn is_guard_active(&self) -> bool {
        self.is_updating.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_settings_default_values() {
        let default_settings = GuiSettings::default();
        assert!(default_settings.is_vietnamese);
        assert_eq!(default_settings.typing_method, TypingMethod::BasicTelex);
        assert_eq!(default_settings.shortcut, ShortcutOption::SuperSpace);
        assert!(default_settings.modern_tone);
        assert!(default_settings.auto_undo_repeat);
        assert!(default_settings.backspace_recovery);
        assert!(default_settings.autostart_hyprland);
    }

    #[test]
    fn test_typing_method_index_mapping() {
        for idx in 0..3 {
            let method = TypingMethod::from_index(idx);
            assert_eq!(method.to_index(), idx);
        }
        assert_eq!(TypingMethod::from_index(99), TypingMethod::Disabled);
    }

    #[test]
    fn test_shortcut_option_index_mapping() {
        for idx in 0..4 {
            let opt = ShortcutOption::from_index(idx);
            assert_eq!(opt.to_index(), idx);
        }
        assert_eq!(ShortcutOption::from_index(99), ShortcutOption::CapsLock);
    }

    #[test]
    fn test_guard_flag_prevents_feedback_loop() {
        let controller = GuiStateController::new();
        assert!(!controller.is_guard_active());

        // Simulate external IPC update
        controller.set_guard(true);
        assert!(controller.is_guard_active());

        // While guard is active, user callbacks check is_guard_active and return early
        assert!(controller.is_guard_active());
        controller.set_guard(false);
        assert!(!controller.is_guard_active());
    }

    #[test]
    fn test_set_mode_from_daemon() {
        let controller = GuiStateController::new();
        assert!(controller.set_mode_from_daemon("VI"));
        assert!(controller.get_settings().is_vietnamese);

        assert!(!controller.set_mode_from_daemon("EN"));
        assert!(!controller.get_settings().is_vietnamese);
        assert!(!controller.is_guard_active());
    }
}
