use super::virtual_keyboard::VirtualKeyboardHandler;
use crate::state::{IpcToWayland, SharedDaemonState, WaylandToIpc};
use std::os::fd::AsFd;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use vietkeyra_engine::{KeyAction, VietKeyraEngine};
use wayland_client::protocol::wl_keyboard::KeyState;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::{self, WlSeat};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_method_keyboard_grab_v2::{
    self, ZwpInputMethodKeyboardGrabV2,
};
use wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_method_manager_v2::{
    self, ZwpInputMethodManagerV2,
};
use wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_method_v2::{
    self, ZwpInputMethodV2,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::{
    self, ZwpVirtualKeyboardManagerV1,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::{
    self, ZwpVirtualKeyboardV1,
};
use xkbcommon::xkb;

/// Maximum safe length for surrounding text window before truncation (Wayland message limit is 4096 bytes).
pub const MAX_SURROUNDING_BYTES: usize = 4000;

/// State struct holding Wayland proxies, XKB state, and Telex engine.
pub struct WaylandState {
    pub seat: Option<WlSeat>,
    pub im_mgr: Option<ZwpInputMethodManagerV2>,
    pub vk_mgr: Option<ZwpVirtualKeyboardManagerV1>,
    pub im: Option<ZwpInputMethodV2>,
    pub grab: Option<ZwpInputMethodKeyboardGrabV2>,
    pub vk: Option<VirtualKeyboardHandler>,
    pub engine: VietKeyraEngine,
    pub xkb_context: xkb::Context,
    pub xkb_keymap: Option<xkb::Keymap>,
    pub xkb_state: Option<xkb::State>,
    pub active: bool,
    pub serial: u32,
    pub surrounding_text: String,
    pub surrounding_cursor: u32,
    pub surrounding_anchor: u32,
    pub has_surrounding_text: bool,
    pub unavailable: bool,
    pub rx_from_ipc: Arc<std::sync::Mutex<Receiver<IpcToWayland>>>,
    pub tx_to_ipc: Sender<WaylandToIpc>,
    pub shared_state: Arc<SharedDaemonState>,
}

impl WaylandState {
    pub fn new(
        shared_state: Arc<SharedDaemonState>,
        rx_from_ipc: Arc<std::sync::Mutex<Receiver<IpcToWayland>>>,
        tx_to_ipc: Sender<WaylandToIpc>,
    ) -> Self {
        let style = shared_state.get_style();
        let engine = VietKeyraEngine::new(style);
        let xkb_context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

        Self {
            seat: None,
            im_mgr: None,
            vk_mgr: None,
            im: None,
            grab: None,
            vk: None,
            engine,
            xkb_context,
            xkb_keymap: None,
            xkb_state: None,
            active: false,
            serial: 0,
            surrounding_text: String::new(),
            surrounding_cursor: 0,
            surrounding_anchor: 0,
            has_surrounding_text: false,
            unavailable: false,
            rx_from_ipc,
            tx_to_ipc,
            shared_state,
        }
    }

    /// Synchronizes pending state from IPC thread (mode or style changes).
    pub fn drain_ipc_events(&mut self) {
        if let Ok(rx) = self.rx_from_ipc.lock() {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    IpcToWayland::SetMode(mode) => {
                        self.engine.set_mode(mode);
                    }
                    IpcToWayland::ToggleMode => {
                        let new_mode = self.engine.toggle_mode();
                        let _ = self.tx_to_ipc.send(WaylandToIpc::ModeChanged(new_mode));
                    }
                    IpcToWayland::SetStyle(style) => {
                        self.engine.set_style(style);
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Dispatch Implementations
// -----------------------------------------------------------------------------

impl Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_seat" => {
                    let bind_version = version.min(7);
                    state.seat = Some(registry.bind(name, bind_version, qh, ()));
                }
                "zwp_input_method_manager_v2" => {
                    state.im_mgr = Some(registry.bind(name, 1, qh, ()));
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    state.vk_mgr = Some(registry.bind(name, 1, qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpInputMethodManagerV2,
        _: zwp_input_method_manager_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardManagerV1,
        _: zwp_virtual_keyboard_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardV1,
        _: zwp_virtual_keyboard_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                state.active = true;
                state.engine.reset_word();
                state.has_surrounding_text = false;
                state.surrounding_text.clear();
            }

            zwp_input_method_v2::Event::Deactivate => {
                // Focus shifted away: immediately flush word buffer to prevent state leaking
                state.active = false;
                state.engine.reset_word();
                state.has_surrounding_text = false;
                state.surrounding_text.clear();
            }

            zwp_input_method_v2::Event::SurroundingText {
                text,
                cursor,
                anchor,
            } => {
                state.has_surrounding_text = true;
                state.surrounding_cursor = cursor;
                state.surrounding_anchor = anchor;

                // Safely handle surrounding text truncation if buffer exceeds 4000 bytes
                if text.len() > MAX_SURROUNDING_BYTES {
                    // Safe UTF-8 character boundary alignment
                    let target_start = text.len().saturating_sub(1000);
                    let mut start = target_start;
                    while start > 0 && !text.is_char_boundary(start) {
                        start -= 1;
                    }
                    state.surrounding_text = text[start..].to_string();
                } else {
                    state.surrounding_text = text;
                }
            }

            zwp_input_method_v2::Event::Done => {
                state.serial = state.serial.wrapping_add(1);
            }

            zwp_input_method_v2::Event::Unavailable => {
                // Seat exclusivity: compositor indicates another IME (e.g. fcitx5) occupies the seat.
                state.unavailable = true;
                eprintln!(
                    "VietKeyra: zwp_input_method_v2::Unavailable - Ghế nhập liệu hiện đang bị chiếm bởi bộ gõ khác (ví dụ: fcitx5). Daemon tiếp tục chạy chế độ chờ."
                );
            }

            zwp_input_method_v2::Event::TextChangeCause { .. } => {}
            zwp_input_method_v2::Event::ContentType { .. } => {}
            _ => {}
        }
    }
}

impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodKeyboardGrabV2,
        event: zwp_input_method_keyboard_grab_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { format, fd, size } => {
                // Forward keymap to virtual keyboard
                if let Some(ref vk) = state.vk {
                    vk.set_keymap(format.into(), fd.as_fd(), size);
                }

                // Map the anonymous file descriptor into memory for libxkbcommon
                let mmap = unsafe { memmap2::MmapOptions::new().len(size as usize).map(&fd) };
                if let Ok(mmap) = mmap {
                    // Wayland compositor keymap buffer usually includes a trailing null byte. Strip it.
                    let slice = if mmap.last() == Some(&0) {
                        &mmap[..mmap.len() - 1]
                    } else {
                        &mmap[..]
                    };
                    if let Ok(keymap_str) = std::str::from_utf8(slice) {
                        let clean_str = keymap_str.trim_matches('\0');
                        if let Some(keymap) = xkb::Keymap::new_from_string(
                            &state.xkb_context,
                            clean_str.to_string(),
                            xkb::KEYMAP_FORMAT_TEXT_V1,
                            xkb::KEYMAP_COMPILE_NO_FLAGS,
                        ) {
                            state.xkb_state = Some(xkb::State::new(&keymap));
                            state.xkb_keymap = Some(keymap);
                        }
                    }
                }
            }

            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                serial: _,
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                // Update xkbcommon modifier tracking
                if let Some(ref mut xkb_st) = state.xkb_state {
                    xkb_st.update_mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
                }

                // Synchronize modifier mask with virtual keyboard
                if let Some(ref vk) = state.vk {
                    vk.update_modifiers(mods_depressed, mods_latched, mods_locked, group);
                }
            }

            zwp_input_method_keyboard_grab_v2::Event::RepeatInfo { .. } => {}

            zwp_input_method_keyboard_grab_v2::Event::Key {
                serial: _,
                time,
                key,
                state: key_state_val,
            } => {
                // Synchronize inter-thread IPC mode changes before handling key
                state.drain_ipc_events();

                let is_press = matches!(key_state_val, WEnum::Value(KeyState::Pressed));

                if !is_press {
                    // Forward key release unconditionally so physical keys never stick
                    if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 0);
                    }
                    return;
                }

                // If not active on an editable input field, forward key press transparently
                if !state.active {
                    if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 1);
                    }
                    return;
                }

                // Inspect modifiers: Control, Alt, Super/Logo shortcuts bypass Telex
                let (ctrl, alt, logo) = if let Some(ref xkb_st) = state.xkb_state {
                    (
                        xkb_st.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE),
                        xkb_st.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE),
                        xkb_st.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE),
                    )
                } else {
                    (false, false, false)
                };

                if ctrl || alt || logo {
                    // Shortcut key chord bypasses Telex diacritics
                    state.engine.reset_word();
                    if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 1);
                    }
                    return;
                }

                // English mode bypasses Telex diacritics
                if state.engine.get_mode().is_english() {
                    if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 1);
                    }
                    return;
                }

                // Translate hardware scancode (evdev scancode) to XKB keysym and UTF-8 char
                // Note: Wayland key events report evdev key code; XKB expects (key + 8)
                let xkb_kc = xkb::Keycode::new(key + 8);
                let (sym_raw, utf8_str) = if let Some(ref xkb_st) = state.xkb_state {
                    (
                        xkb_st.key_get_one_sym(xkb_kc).raw(),
                        xkb_st.key_get_utf8(xkb_kc),
                    )
                } else {
                    (xkb::keysyms::KEY_NoSymbol, String::new())
                };

                // Handle BackSpace
                if sym_raw == xkb::keysyms::KEY_BackSpace {
                    match state.engine.handle_backspace() {
                        KeyAction::Replace {
                            delete_bytes, text, ..
                        } => {
                            if let Some(ref im) = state.im {
                                if delete_bytes > 0 {
                                    im.delete_surrounding_text(delete_bytes as u32, 0);
                                }
                                if !text.is_empty() {
                                    im.commit_string(text);
                                }
                                im.commit(state.serial);
                            }
                            return;
                        }
                        KeyAction::PassThrough | KeyAction::Commit { .. } => {
                            if let Some(ref vk) = state.vk {
                                vk.forward_key(time, key, 1);
                            }
                            return;
                        }
                    }
                }

                // Handle Navigation and Control keys (Return, Tab, Escape, Arrows, Home, End, Delete)
                if matches!(
                    sym_raw,
                    xkb::keysyms::KEY_Return
                        | xkb::keysyms::KEY_KP_Enter
                        | xkb::keysyms::KEY_Tab
                        | xkb::keysyms::KEY_Escape
                        | xkb::keysyms::KEY_Left
                        | xkb::keysyms::KEY_Right
                        | xkb::keysyms::KEY_Up
                        | xkb::keysyms::KEY_Down
                        | xkb::keysyms::KEY_Home
                        | xkb::keysyms::KEY_End
                        | xkb::keysyms::KEY_Page_Up
                        | xkb::keysyms::KEY_Page_Down
                        | xkb::keysyms::KEY_Delete
                        | xkb::keysyms::KEY_Insert
                ) {
                    state.engine.reset_word();
                    if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 1);
                    }
                    return;
                }

                // Handle Space: commits word and inserts space
                if sym_raw == xkb::keysyms::KEY_space {
                    state.engine.reset_word();
                    if let Some(ref im) = state.im {
                        im.commit_string(" ".to_string());
                        im.commit(state.serial);
                    } else if let Some(ref vk) = state.vk {
                        vk.forward_key(time, key, 1);
                    }
                    return;
                }

                // Handle printable characters
                if let Some(ch) = utf8_str.chars().next() {
                    if ch.is_alphabetic() {
                        let action = state.engine.process_char(ch);
                        match action {
                            KeyAction::Replace {
                                delete_bytes, text, ..
                            } => {
                                if let Some(ref im) = state.im {
                                    if delete_bytes > 0 {
                                        // Strict UTF-8 byte offset deletion
                                        im.delete_surrounding_text(delete_bytes as u32, 0);
                                    }
                                    im.commit_string(text);
                                    im.commit(state.serial);
                                } else if let Some(ref vk) = state.vk {
                                    vk.forward_key(time, key, 1);
                                }
                            }
                            KeyAction::Commit { text } => {
                                if let Some(ref im) = state.im {
                                    im.commit_string(text);
                                    im.commit(state.serial);
                                } else if let Some(ref vk) = state.vk {
                                    vk.forward_key(time, key, 1);
                                }
                            }
                            KeyAction::PassThrough => {
                                if let Some(ref vk) = state.vk {
                                    vk.forward_key(time, key, 1);
                                }
                            }
                        }
                        return;
                    } else if !ch.is_control() {
                        // Punctuation, digits, symbols: commit character and reset word buffer
                        state.engine.reset_word();
                        if let Some(ref im) = state.im {
                            im.commit_string(ch.to_string());
                            im.commit(state.serial);
                        } else if let Some(ref vk) = state.vk {
                            vk.forward_key(time, key, 1);
                        }
                        return;
                    }
                }

                // Fallback for function keys or unhandled keys: pass through to virtual keyboard
                if let Some(ref vk) = state.vk {
                    vk.forward_key(time, key, 1);
                }
            }

            _ => {}
        }
    }
}
