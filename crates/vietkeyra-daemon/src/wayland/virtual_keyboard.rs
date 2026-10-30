use std::os::fd::BorrowedFd;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

/// Wrapper around zwp_virtual_keyboard_v1 for transparent keystroke pass-through.
#[derive(Debug, Clone)]
pub struct VirtualKeyboardHandler {
    vk: ZwpVirtualKeyboardV1,
}

impl VirtualKeyboardHandler {
    pub fn new(vk: ZwpVirtualKeyboardV1) -> Self {
        Self { vk }
    }

    /// Forwards a hardware key event (press or release) to the virtual keyboard.
    pub fn forward_key(&self, time: u32, key: u32, state: u32) {
        self.vk.key(time, key, state);
    }

    /// Synchronizes modifier masks with the virtual keyboard.
    pub fn update_modifiers(&self, depressed: u32, latched: u32, locked: u32, group: u32) {
        self.vk.modifiers(depressed, latched, locked, group);
    }

    /// Forwards compositor keymap file descriptor to the virtual keyboard.
    pub fn set_keymap(&self, format: u32, fd: BorrowedFd<'_>, size: u32) {
        self.vk.keymap(format, fd, size);
    }

    /// Access to inner ZwpVirtualKeyboardV1.
    pub fn raw(&self) -> &ZwpVirtualKeyboardV1 {
        &self.vk
    }
}
