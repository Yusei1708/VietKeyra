use std::sync::RwLock;
use vietkeyra_engine::{InputMode, TonePlacementStyle};

/// Commands sent from the IPC thread to the Wayland IME thread.
#[derive(Debug, Clone)]
pub enum IpcToWayland {
    SetMode(InputMode),
    ToggleMode,
    SetStyle(TonePlacementStyle),
}

/// Events sent from the Wayland IME thread to the IPC thread.
#[derive(Debug, Clone)]
pub enum WaylandToIpc {
    ModeChanged(InputMode),
}

/// Shared daemon state accessed concurrently across threads.
#[derive(Debug)]
pub struct SharedDaemonState {
    mode: RwLock<InputMode>,
    style: RwLock<TonePlacementStyle>,
    version: &'static str,
    engine: &'static str,
}

impl Default for SharedDaemonState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedDaemonState {
    pub fn new() -> Self {
        Self {
            mode: RwLock::new(InputMode::Vietnamese),
            style: RwLock::new(TonePlacementStyle::Modern),
            version: "0.1.0",
            engine: "telex",
        }
    }

    pub fn get_mode(&self) -> InputMode {
        *self.mode.read().unwrap()
    }

    pub fn get_mode_str(&self) -> &'static str {
        match self.get_mode() {
            InputMode::Vietnamese => "VI",
            InputMode::English => "EN",
        }
    }

    pub fn set_mode(&self, mode: InputMode) {
        let mut m = self.mode.write().unwrap();
        *m = mode;
    }

    pub fn toggle_mode(&self) -> InputMode {
        let mut m = self.mode.write().unwrap();
        let new_mode = match *m {
            InputMode::Vietnamese => InputMode::English,
            InputMode::English => InputMode::Vietnamese,
        };
        *m = new_mode;
        new_mode
    }

    pub fn get_style(&self) -> TonePlacementStyle {
        *self.style.read().unwrap()
    }

    pub fn set_style(&self, style: TonePlacementStyle) {
        let mut s = self.style.write().unwrap();
        *s = style;
    }

    pub fn version(&self) -> &'static str {
        self.version
    }

    pub fn engine(&self) -> &'static str {
        self.engine
    }
}
