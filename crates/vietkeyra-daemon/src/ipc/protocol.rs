use serde::{Deserialize, Serialize};

/// Maximum allowed payload size for a single command line (64 KB).
pub const MAX_PAYLOAD_BYTES: usize = 65536;

/// Incoming IPC commands from clients (CLI, GUI, Waybar).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcRequest {
    GetStatus,
    Toggle,
    SetMode { mode: String },
    SetEngine { engine: String },
    ReloadConfig,
    Subscribe,
    Quit,
}

/// Outgoing IPC responses from daemon to clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IpcResponse {
    Ok {
        status: String,
        mode: String,
        engine: String,
        version: String,
    },
    Error {
        status: String,
        message: String,
    },
}

impl IpcResponse {
    pub fn ok(mode: &str, engine: &str, version: &str) -> Self {
        Self::Ok {
            status: "ok".to_string(),
            mode: mode.to_string(),
            engine: engine.to_string(),
            version: version.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self::Error {
            status: "error".to_string(),
            message: message.to_string(),
        }
    }

    pub fn to_json_line(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"status\":\"error\",\"message\":\"Serialization error\"}".to_string()
        });
        s.push('\n');
        s
    }
}

/// Broadcast event emitted to subscribers on mode changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcEvent {
    pub event: String,
    pub mode: String,
    pub engine: String,
}

impl IpcEvent {
    pub fn mode_changed(mode: &str, engine: &str) -> Self {
        Self {
            event: "mode_changed".to_string(),
            mode: mode.to_string(),
            engine: engine.to_string(),
        }
    }

    pub fn to_json_line(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"event\":\"mode_changed\",\"mode\":\"VI\",\"engine\":\"telex\"}".to_string()
        });
        s.push('\n');
        s
    }
}

/// Parses an incoming line into an `IpcRequest`.
pub fn parse_request(line: &str) -> Result<IpcRequest, IpcResponse> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(IpcResponse::error("Empty command"));
    }

    if trimmed.len() > MAX_PAYLOAD_BYTES {
        return Err(IpcResponse::error("Oversized payload rejected"));
    }

    // Check if JSON has valid syntax
    let v: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(val) => val,
        Err(_) => return Err(IpcResponse::error("Invalid JSON")),
    };

    let cmd_str = match v.get("cmd").and_then(|c| c.as_str()) {
        Some(c) => c,
        None => return Err(IpcResponse::error("Missing 'cmd' field in request")),
    };

    match serde_json::from_str::<IpcRequest>(trimmed) {
        Ok(req) => Ok(req),
        Err(_) => Err(IpcResponse::error(&format!("Unknown command: {cmd_str}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_requests() {
        assert_eq!(
            parse_request("{\"cmd\": \"get_status\"}").unwrap(),
            IpcRequest::GetStatus
        );
        assert_eq!(
            parse_request("{\"cmd\": \"toggle\"}").unwrap(),
            IpcRequest::Toggle
        );
        assert_eq!(
            parse_request("{\"cmd\": \"set_mode\", \"mode\": \"EN\"}").unwrap(),
            IpcRequest::SetMode {
                mode: "EN".to_string()
            }
        );
        assert_eq!(
            parse_request("{\"cmd\": \"set_engine\", \"engine\": \"telex\"}").unwrap(),
            IpcRequest::SetEngine {
                engine: "telex".to_string()
            }
        );
        assert_eq!(
            parse_request("{\"cmd\": \"reload_config\"}").unwrap(),
            IpcRequest::ReloadConfig
        );
        assert_eq!(
            parse_request("{\"cmd\": \"subscribe\"}").unwrap(),
            IpcRequest::Subscribe
        );
        assert_eq!(
            parse_request("{\"cmd\": \"quit\"}").unwrap(),
            IpcRequest::Quit
        );
    }

    #[test]
    fn test_parse_invalid_requests() {
        let err1 = parse_request("{invalid_json").unwrap_err();
        assert_eq!(err1, IpcResponse::error("Invalid JSON"));

        let err2 = parse_request("{\"cmd\": \"destroy_all\"}").unwrap_err();
        assert_eq!(err2, IpcResponse::error("Unknown command: destroy_all"));
    }

    #[test]
    fn test_response_and_event_serialization() {
        let resp = IpcResponse::ok("VI", "telex", "0.1.0");
        let json = resp.to_json_line();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"mode\":\"VI\""));
        assert!(json.ends_with('\n'));

        let ev = IpcEvent::mode_changed("EN", "telex");
        let ev_json = ev.to_json_line();
        assert!(ev_json.contains("\"event\":\"mode_changed\""));
        assert!(ev_json.contains("\"mode\":\"EN\""));
        assert!(ev_json.ends_with('\n'));
    }
}
