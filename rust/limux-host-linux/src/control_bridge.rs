//! Bridge the limux control socket onto the GTK host state.

use std::io::{self, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use gtk::glib;
use gtk4 as gtk;

use crate::control_registry::{RouteClass, WAVE1_MUTATION_KILL_SWITCH_ENV};
use crate::layout_state::PaneFlagColor;
use limux_control::auth::{self, SocketControlMode};
use limux_control::request_io::{self, read_request_frame};
use limux_control::socket_path::{bind_listener, resolve_socket_path, SocketMode};
use limux_protocol::{
    parse_v1_command_envelope, restricted_method_allowlist, validate_restricted_method,
    validate_terminal_text_payload, V2Request, V2Response,
};
use serde_json::{json, Map, Value};

const PARSE_ERROR_CODE: i64 = -32700;
const INVALID_PARAMS_CODE: i64 = -32602;
const UNKNOWN_METHOD_CODE: i64 = -32601;
const INTERNAL_ERROR_CODE: i64 = -32603;
const NOT_FOUND_CODE: i64 = -32004;
const CONFLICT_CODE: i64 = -32009;
const DEFAULT_CONTROL_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const PANE_CREATE_CONTROL_COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const CURSOR_PANE_CREATE_EMPTY_PARAMS: &[&str] = &[
    "workspace_id",
    "id",
    "name",
    "index",
    "surface_id",
    "pane_id",
    "direction",
];

type BridgeResult = Result<Value, BridgeError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MethodSurface {
    Unrestricted,
    CursorRestricted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceTarget {
    Active,
    Handle(String),
    Name(String),
    Index(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaneCreateDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaneCreateType {
    Terminal,
    Browser,
}

/// Parser-level contract for the live-GTK `pane.create` route.
///
/// Request fields accepted by the bridge:
/// - `workspace_id`/`id`, `name`, or `index` target the workspace. Raw
///   handles and `workspace:<id>` refs are accepted and preserved for the GTK
///   layer to resolve.
/// - `surface_id` and `pane_id` identify the source pane. Raw handles and
///   `surface:<id>`/`pane:<id>` refs are accepted. Later GTK work resolves
///   precedence as explicit surface, explicit pane, then safe workspace-local
///   fallback.
/// - `direction` is one of `left|right|up|down`, defaulting to `right`.
/// - `type` is one of `terminal|browser`, defaulting to `terminal`.
/// - `command` is a terminal-only host extension: the host injects it into the
///   newly-created surface after creation. The standalone core dispatcher may
///   accept the field for compatibility but does not launch a process.
///
/// This delivery only implements live-GTK terminal panes. Browser pane support
/// remains a follow-up, so `type=browser` and `url` fail at parse time before
/// any GTK work is scheduled. Responses must keep the existing core/CLI field
/// names: `pane_id`, `pane_ref`, `surface_id`, and `surface_ref`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePaneRequest {
    pub target: WorkspaceTarget,
    pub source_pane_id: Option<String>,
    pub source_surface_id: Option<String>,
    pub direction: PaneCreateDirection,
    pub pane_type: PaneCreateType,
    pub command: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaneActionKind {
    SetFlagColor(PaneFlagColor),
    ClearFlagColor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneActionRequest {
    pub target: WorkspaceTarget,
    pub pane_id: Option<String>,
    pub action: PaneActionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneFocusRequest {
    pub target: WorkspaceTarget,
    pub pane_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneResizeRequest {
    pub target: WorkspaceTarget,
    pub pane_id: String,
    pub direction: PaneResizeDirection,
    pub amount: f64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceSplitRequest {
    pub target: WorkspaceTarget,
    pub source_surface_id: Option<String>,
    pub direction: PaneCreateDirection,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceFocusRequest {
    pub target: WorkspaceTarget,
    pub surface_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceCloseRequest {
    pub target: WorkspaceTarget,
    pub surface_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneResizeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub enum ControlCommand {
    Identify {
        caller: Option<Value>,
        reply: mpsc::Sender<BridgeResult>,
    },
    PresentWindow {
        reply: mpsc::Sender<BridgeResult>,
    },
    CurrentWorkspace {
        reply: mpsc::Sender<BridgeResult>,
    },
    ListWorkspaces {
        reply: mpsc::Sender<BridgeResult>,
    },
    ListPanes {
        target: WorkspaceTarget,
        reply: mpsc::Sender<BridgeResult>,
    },
    ListPaneSurfaces {
        target: WorkspaceTarget,
        pane_id: Option<String>,
        reply: mpsc::Sender<BridgeResult>,
    },
    CreatePane {
        request: CreatePaneRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    PaneAction {
        request: PaneActionRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    FocusPane {
        request: PaneFocusRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    ResizePane {
        request: PaneResizeRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    SplitSurface {
        request: SurfaceSplitRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    FocusSurface {
        request: SurfaceFocusRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    CloseSurface {
        request: SurfaceCloseRequest,
        reply: mpsc::Sender<BridgeResult>,
    },
    ListSurfaces {
        target: WorkspaceTarget,
        reply: mpsc::Sender<BridgeResult>,
    },
    CurrentSurface {
        target: WorkspaceTarget,
        reply: mpsc::Sender<BridgeResult>,
    },
    SurfaceHealth {
        target: WorkspaceTarget,
        surface_hint: Option<String>,
        reply: mpsc::Sender<BridgeResult>,
    },
    ReadSurfaceText {
        target: WorkspaceTarget,
        surface_hint: Option<String>,
        scrollback: bool,
        lines: Option<usize>,
        reply: mpsc::Sender<BridgeResult>,
    },
    CreateWorkspace {
        name: Option<String>,
        cwd: Option<String>,
        command: Option<String>,
        reply: mpsc::Sender<BridgeResult>,
    },
    SelectWorkspace {
        target: WorkspaceTarget,
        reply: mpsc::Sender<BridgeResult>,
    },
    RenameWorkspace {
        target: WorkspaceTarget,
        title: String,
        reply: mpsc::Sender<BridgeResult>,
    },
    CloseWorkspace {
        target: WorkspaceTarget,
        reply: mpsc::Sender<BridgeResult>,
    },
    SendText {
        target: WorkspaceTarget,
        surface_hint: Option<String>,
        text: String,
        reply: mpsc::Sender<BridgeResult>,
    },
    SendKey {
        target: WorkspaceTarget,
        surface_hint: Option<String>,
        key: String,
        reply: mpsc::Sender<BridgeResult>,
    },
    /// Post a desktop-style notification into the sidebar + toast overlay.
    /// `target` chooses the workspace to flag as unread; if not provided,
    /// the currently-active workspace is used.
    CreateNotification {
        target: WorkspaceTarget,
        title: String,
        subtitle: String,
        body: String,
        reply: mpsc::Sender<BridgeResult>,
    },
    ListNotifications {
        unread_only: bool,
        reply: mpsc::Sender<BridgeResult>,
    },
    FallthroughRead {
        method: String,
        params: Value,
        reply: mpsc::Sender<BridgeResult>,
    },
}

impl ControlCommand {
    pub fn respond(self, result: BridgeResult) {
        match self {
            Self::Identify { reply, .. }
            | Self::PresentWindow { reply }
            | Self::CurrentWorkspace { reply }
            | Self::ListWorkspaces { reply }
            | Self::ListPanes { reply, .. }
            | Self::ListPaneSurfaces { reply, .. }
            | Self::CreatePane { reply, .. }
            | Self::PaneAction { reply, .. }
            | Self::FocusPane { reply, .. }
            | Self::ResizePane { reply, .. }
            | Self::SplitSurface { reply, .. }
            | Self::FocusSurface { reply, .. }
            | Self::CloseSurface { reply, .. }
            | Self::ListSurfaces { reply, .. }
            | Self::CurrentSurface { reply, .. }
            | Self::SurfaceHealth { reply, .. }
            | Self::ReadSurfaceText { reply, .. }
            | Self::CreateWorkspace { reply, .. }
            | Self::SelectWorkspace { reply, .. }
            | Self::RenameWorkspace { reply, .. }
            | Self::CloseWorkspace { reply, .. }
            | Self::SendText { reply, .. }
            | Self::SendKey { reply, .. }
            | Self::CreateNotification { reply, .. }
            | Self::ListNotifications { reply, .. }
            | Self::FallthroughRead { reply, .. } => {
                let _ = reply.send(result);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl BridgeError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub(crate) fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS_CODE, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(NOT_FOUND_CODE, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(CONFLICT_CODE, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR_CODE, message)
    }
}

fn parse_request(input: &str) -> Result<V2Request, BridgeError> {
    if let Ok(request) = serde_json::from_str::<V2Request>(input) {
        return Ok(request);
    }

    match parse_v1_command_envelope(input) {
        Ok(v1) => Ok(v1.into_v2_request(None)),
        Err(error) => Err(BridgeError::new(
            PARSE_ERROR_CODE,
            format!("invalid request payload: {error}"),
        )
        .with_data(json!({ "raw": input }))),
    }
}

fn cursor_restricted_socket_path(runtime_path: &Path) -> PathBuf {
    let file_name = runtime_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("limux.sock");
    if file_name.ends_with(".cursor.sock") || file_name.ends_with(".cursor") {
        return runtime_path.to_path_buf();
    }
    let cursor_file_name = file_name
        .strip_suffix(".sock")
        .map(|stem| format!("{stem}.cursor.sock"))
        .unwrap_or_else(|| format!("{file_name}.cursor"));
    runtime_path.with_file_name(cursor_file_name)
}

fn is_restricted_system_method(method: &str) -> bool {
    matches!(
        method,
        "system.ping" | "system.identify" | "system.capabilities"
    )
}

fn restricted_unknown_method(method: &str) -> BridgeError {
    BridgeError::new(
        UNKNOWN_METHOD_CODE,
        format!("restricted Limux method is not allowlisted: {method}"),
    )
}

fn ensure_only_params(
    method: &str,
    params: &Map<String, Value>,
    allowed: &[&str],
) -> Result<(), BridgeError> {
    for key in params.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(BridgeError::invalid_params(format!(
                "{method} unexpected parameter: {key}"
            )));
        }
    }
    Ok(())
}

fn validate_cursor_restricted_request(
    method: &str,
    params: &Map<String, Value>,
) -> Result<(), BridgeError> {
    if !is_restricted_system_method(method) {
        validate_restricted_method(method).map_err(|_| restricted_unknown_method(method))?;
    }

    match method {
        "system.ping" | "system.capabilities" => ensure_only_params(method, params, &[]),
        "system.identify" => ensure_only_params(method, params, &["caller"]),
        "workspace.list" | "window.present" => ensure_only_params(method, params, &[]),
        "workspace.select" => {
            ensure_only_params(method, params, &["workspace_id", "id", "name", "index"])
        }
        "cursor.pane_create_empty" => {
            ensure_only_params(method, params, CURSOR_PANE_CREATE_EMPTY_PARAMS)
        }
        "surface.read_text" => ensure_only_params(
            method,
            params,
            &["workspace_id", "name", "index", "surface_id"],
        ),
        "cursor.workspace_open_folder" => {
            ensure_only_params(method, params, &["path", "folder", "cwd", "name", "title"])
        }
        _ => Err(restricted_unknown_method(method)),
    }
}

fn params_object(params: &Value) -> Result<&Map<String, Value>, BridgeError> {
    params
        .as_object()
        .ok_or_else(|| BridgeError::invalid_params("params must be a JSON object"))
}

fn optional_string(params: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        params
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn validate_terminal_text_param(label: &str, text: &str) -> Result<(), BridgeError> {
    validate_terminal_text_payload(label, text)
        .map_err(|error| BridgeError::invalid_params(error.to_string()))
}

fn optional_terminal_text(
    params: &Map<String, Value>,
    keys: &[&str],
    label: &str,
) -> Result<Option<String>, BridgeError> {
    let value = optional_string(params, keys);
    if let Some(value) = value.as_ref() {
        validate_terminal_text_param(label, value)?;
    }
    Ok(value)
}

fn optional_handle(
    params: &Map<String, Value>,
    keys: &[&str],
) -> Result<Option<String>, BridgeError> {
    for key in keys {
        let Some(value) = params.get(*key) else {
            continue;
        };
        match value {
            Value::Null => {}
            Value::String(raw) => {
                let handle = raw.trim();
                if !handle.is_empty() {
                    return Ok(Some(handle.to_string()));
                }
            }
            Value::Number(number) => {
                let id = number.as_u64().ok_or_else(|| {
                    BridgeError::invalid_params(format!(
                        "{key} must be a non-negative integer or ref handle"
                    ))
                })?;
                return Ok(Some(id.to_string()));
            }
            _ => {
                return Err(BridgeError::invalid_params(format!(
                    "{key} must be a non-negative integer or ref handle"
                )));
            }
        }
    }
    Ok(None)
}

fn optional_ref_handle(
    params: &Map<String, Value>,
    keys: &[&str],
    prefix: &str,
) -> Result<Option<String>, BridgeError> {
    optional_handle(params, keys).map(|handle| {
        handle.map(|handle| {
            handle
                .strip_prefix(prefix)
                .unwrap_or(handle.as_str())
                .to_string()
        })
    })
}

fn optional_surface_ref_handle(
    params: &Map<String, Value>,
    keys: &[&str],
    method: &str,
) -> Result<Option<String>, BridgeError> {
    for key in keys {
        let Some(value) = params.get(*key) else {
            continue;
        };
        match value {
            Value::Null => return Ok(None),
            Value::String(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Err(BridgeError::invalid_params(format!(
                        "{method} {key} must not be empty"
                    )));
                }
                let handle = trimmed.strip_prefix("surface:").unwrap_or(trimmed).trim();
                if handle.is_empty() {
                    return Err(BridgeError::invalid_params(format!(
                        "{method} {key} must not be empty"
                    )));
                }
                return Ok(Some(handle.to_string()));
            }
            Value::Number(number) => {
                let id = number.as_u64().ok_or_else(|| {
                    BridgeError::invalid_params(format!(
                        "{method} {key} must be a non-negative integer or surface ref"
                    ))
                })?;
                return Ok(Some(id.to_string()));
            }
            _ => {
                return Err(BridgeError::invalid_params(format!(
                    "{method} {key} must be a non-negative integer or surface ref"
                )));
            }
        }
    }
    Ok(None)
}

fn optional_index(params: &Map<String, Value>, key: &str) -> Result<Option<usize>, BridgeError> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };

    if let Some(index) = value.as_u64() {
        return Ok(Some(index as usize));
    }

    Err(BridgeError::invalid_params(format!(
        "{key} must be a non-negative integer"
    )))
}

fn optional_bool(params: &Map<String, Value>, key: &str) -> Result<Option<bool>, BridgeError> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };
    match value {
        Value::Null => Ok(None),
        Value::Bool(value) => Ok(Some(*value)),
        _ => Err(BridgeError::invalid_params(format!(
            "{key} must be a boolean"
        ))),
    }
}

fn looks_like_workspace_handle(raw: &str) -> bool {
    let raw = raw.trim();
    if raw.starts_with("workspace:") {
        return true;
    }
    let value = raw;
    uuid::Uuid::parse_str(value).is_ok() || value.chars().all(|ch| ch.is_ascii_digit())
}

fn parse_optional_workspace_target(
    params: &Map<String, Value>,
    allow_name: bool,
) -> Result<WorkspaceTarget, BridgeError> {
    if let Some(handle) = optional_handle(params, &["workspace_id", "id"])? {
        if allow_name && !looks_like_workspace_handle(&handle) {
            return Ok(WorkspaceTarget::Name(handle));
        }
        return Ok(WorkspaceTarget::Handle(handle));
    }
    if allow_name {
        if let Some(name) = optional_string(params, &["name"]) {
            return Ok(WorkspaceTarget::Name(name));
        }
    }
    if let Some(index) = optional_index(params, "index")? {
        return Ok(WorkspaceTarget::Index(index));
    }
    Ok(WorkspaceTarget::Active)
}

fn parse_optional_workspace_target_without_id(
    params: &Map<String, Value>,
    allow_name: bool,
) -> Result<WorkspaceTarget, BridgeError> {
    if let Some(handle) = optional_handle(params, &["workspace_id"])? {
        if allow_name && !looks_like_workspace_handle(&handle) {
            return Ok(WorkspaceTarget::Name(handle));
        }
        return Ok(WorkspaceTarget::Handle(handle));
    }
    if allow_name {
        if let Some(name) = optional_string(params, &["name"]) {
            return Ok(WorkspaceTarget::Name(name));
        }
    }
    if let Some(index) = optional_index(params, "index")? {
        return Ok(WorkspaceTarget::Index(index));
    }
    Ok(WorkspaceTarget::Active)
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_create_pane_request(
    params: &Map<String, Value>,
) -> Result<CreatePaneRequest, BridgeError> {
    let direction = match optional_string(params, &["direction"])
        .unwrap_or_else(|| "right".to_string())
        .as_str()
    {
        "left" => PaneCreateDirection::Left,
        "right" => PaneCreateDirection::Right,
        "up" => PaneCreateDirection::Up,
        "down" => PaneCreateDirection::Down,
        _ => {
            return Err(BridgeError::invalid_params(
                "pane.create direction must be one of left|right|up|down",
            ));
        }
    };

    let pane_type = match optional_string(params, &["type"])
        .unwrap_or_else(|| "terminal".to_string())
        .as_str()
    {
        "terminal" => PaneCreateType::Terminal,
        "browser" => PaneCreateType::Browser,
        _ => {
            return Err(BridgeError::invalid_params(
                "pane.create type must be one of terminal|browser",
            ));
        }
    };

    if matches!(pane_type, PaneCreateType::Browser) {
        return Err(BridgeError::invalid_params(
            "pane.create live GTK bridge supports type=terminal only",
        ));
    }
    if optional_string(params, &["url"]).is_some() {
        return Err(BridgeError::invalid_params(
            "pane.create url is only supported for browser panes",
        ));
    }

    Ok(CreatePaneRequest {
        target: parse_optional_workspace_target(params, true)?,
        source_pane_id: optional_ref_handle(params, &["pane_id"], "pane:")?,
        source_surface_id: optional_ref_handle(params, &["surface_id"], "surface:")?,
        direction,
        pane_type,
        command: optional_terminal_text(params, &["command"], "pane.create command")?,
    })
}

fn parse_cursor_pane_create_empty_request(
    params: &Map<String, Value>,
) -> Result<CreatePaneRequest, BridgeError> {
    ensure_only_params(
        "cursor.pane_create_empty",
        params,
        CURSOR_PANE_CREATE_EMPTY_PARAMS,
    )?;

    let direction = match optional_string(params, &["direction"])
        .unwrap_or_else(|| "right".to_string())
        .as_str()
    {
        "left" => PaneCreateDirection::Left,
        "right" => PaneCreateDirection::Right,
        "up" => PaneCreateDirection::Up,
        "down" => PaneCreateDirection::Down,
        _ => {
            return Err(BridgeError::invalid_params(
                "cursor.pane_create_empty direction must be one of left|right|up|down",
            ));
        }
    };

    Ok(CreatePaneRequest {
        target: parse_optional_workspace_target(params, true)?,
        source_pane_id: optional_ref_handle(params, &["pane_id"], "pane:")?,
        source_surface_id: optional_ref_handle(params, &["surface_id"], "surface:")?,
        direction,
        pane_type: PaneCreateType::Terminal,
        command: None,
    })
}

fn parse_pane_focus_request(params: &Map<String, Value>) -> Result<PaneFocusRequest, BridgeError> {
    let pane_id = optional_ref_handle(params, &["pane_id", "pane", "id"], "pane:")?
        .ok_or_else(|| BridgeError::invalid_params("pane.focus requires pane_id/id"))?;
    Ok(PaneFocusRequest {
        target: parse_optional_workspace_target_without_id(params, true)?,
        pane_id,
    })
}

fn optional_f64(params: &Map<String, Value>, key: &str) -> Result<Option<f64>, BridgeError> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_f64()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(Some)
            .ok_or_else(|| {
                BridgeError::invalid_params(format!("{key} must be a non-negative number"))
            }),
        _ => Err(BridgeError::invalid_params(format!(
            "{key} must be a non-negative number"
        ))),
    }
}

fn parse_pane_resize_request(
    params: &Map<String, Value>,
) -> Result<PaneResizeRequest, BridgeError> {
    let pane_id = optional_ref_handle(params, &["pane_id", "pane", "id"], "pane:")?
        .ok_or_else(|| BridgeError::invalid_params("pane.resize requires pane_id/id"))?;
    let direction = match optional_string(params, &["direction"])
        .unwrap_or_else(|| "right".to_string())
        .as_str()
    {
        "left" => PaneResizeDirection::Left,
        "right" => PaneResizeDirection::Right,
        "up" => PaneResizeDirection::Up,
        "down" => PaneResizeDirection::Down,
        _ => {
            return Err(BridgeError::invalid_params(
                "pane.resize direction must be one of left|right|up|down",
            ));
        }
    };
    Ok(PaneResizeRequest {
        target: parse_optional_workspace_target_without_id(params, true)?,
        pane_id,
        direction,
        amount: optional_f64(params, "amount")?.unwrap_or(1.0),
    })
}

fn parse_surface_split_request(
    params: &Map<String, Value>,
) -> Result<SurfaceSplitRequest, BridgeError> {
    let direction = match optional_string(params, &["direction"])
        .unwrap_or_else(|| "right".to_string())
        .as_str()
    {
        "left" => PaneCreateDirection::Left,
        "right" => PaneCreateDirection::Right,
        "up" => PaneCreateDirection::Up,
        "down" => PaneCreateDirection::Down,
        _ => {
            return Err(BridgeError::invalid_params(
                "surface.split direction must be one of left|right|up|down",
            ));
        }
    };

    Ok(SurfaceSplitRequest {
        target: parse_optional_workspace_target_without_id(params, true)?,
        source_surface_id: optional_surface_ref_handle(
            params,
            &["surface_id", "surface", "id"],
            "surface.split",
        )?,
        direction,
        title: optional_string(params, &["title"]),
    })
}

fn parse_surface_focus_request(
    params: &Map<String, Value>,
) -> Result<SurfaceFocusRequest, BridgeError> {
    let surface_id =
        optional_surface_ref_handle(params, &["surface_id", "surface", "id"], "surface.focus")?
            .ok_or_else(|| {
                BridgeError::invalid_params("surface.focus requires surface_id/surface/id")
            })?;
    Ok(SurfaceFocusRequest {
        target: parse_optional_workspace_target_without_id(params, true)?,
        surface_id,
    })
}

fn parse_surface_close_request(
    params: &Map<String, Value>,
) -> Result<SurfaceCloseRequest, BridgeError> {
    Ok(SurfaceCloseRequest {
        target: parse_optional_workspace_target_without_id(params, true)?,
        surface_id: optional_surface_ref_handle(
            params,
            &["surface_id", "surface", "id"],
            "surface.close",
        )?,
    })
}

fn parse_pane_action_request(
    params: &Map<String, Value>,
) -> Result<PaneActionRequest, BridgeError> {
    let action = optional_string(params, &["action"])
        .ok_or_else(|| BridgeError::invalid_params("pane.action requires action"))?;
    let action = match action.as_str() {
        "set_flag_color" | "set-flag-color" => {
            let color = optional_string(params, &["color", "flag_color"]).ok_or_else(|| {
                BridgeError::invalid_params("pane.action set_flag_color requires color")
            })?;
            let color = PaneFlagColor::from_name(&color).ok_or_else(|| {
                BridgeError::invalid_params(format!(
                    "pane.action color must be one of {}",
                    PaneFlagColor::allowed_names()
                ))
            })?;
            PaneActionKind::SetFlagColor(color)
        }
        "clear_flag_color" | "clear-flag-color" => PaneActionKind::ClearFlagColor,
        _ => {
            return Err(BridgeError::invalid_params(
                "pane.action action must be set_flag_color or clear_flag_color",
            ));
        }
    };

    Ok(PaneActionRequest {
        target: parse_optional_workspace_target(params, true)?,
        pane_id: optional_ref_handle(params, &["pane_id", "pane"], "pane:")?,
        action,
    })
}

fn parse_cursor_workspace_open_folder(
    params: &Map<String, Value>,
) -> Result<(Option<String>, String), BridgeError> {
    let Some(raw_path) = optional_string(params, &["path", "folder", "cwd"]) else {
        return Err(BridgeError::invalid_params(
            "cursor.workspace_open_folder requires path/folder/cwd",
        ));
    };
    let path = Path::new(&raw_path);
    if !path.is_absolute() {
        return Err(BridgeError::invalid_params(
            "cursor.workspace_open_folder path must be absolute",
        ));
    }
    let canonical = path.canonicalize().map_err(|error| {
        BridgeError::invalid_params(format!(
            "cursor.workspace_open_folder path is not accessible: {error}"
        ))
    })?;
    if !canonical.is_dir() {
        return Err(BridgeError::invalid_params(
            "cursor.workspace_open_folder path must be an existing directory",
        ));
    }
    Ok((
        optional_string(params, &["name", "title"]),
        canonical.to_string_lossy().to_string(),
    ))
}

fn parse_required_workspace_target(
    params: &Map<String, Value>,
    allow_name: bool,
    method: &str,
) -> Result<WorkspaceTarget, BridgeError> {
    let target = parse_optional_workspace_target(params, allow_name)?;
    if matches!(target, WorkspaceTarget::Active) {
        Err(BridgeError::invalid_params(format!(
            "{method} requires workspace_id/id, name, or index"
        )))
    } else {
        Ok(target)
    }
}

fn handle_method(
    id: Option<Value>,
    method: &str,
    params: Value,
    dispatch: &dyn Fn(ControlCommand),
    surface: MethodSurface,
) -> V2Response {
    let params = match params_object(&params) {
        Ok(params) => params,
        Err(error) => return error_response(id, error),
    };
    if surface == MethodSurface::CursorRestricted {
        if let Err(error) = validate_cursor_restricted_request(method, params) {
            return error_response(id, error);
        }
    }

    let queued = match method {
        "system.ping" | "ping" => return V2Response::success(id, json!({ "pong": true })),
        "system.capabilities" => {
            if surface == MethodSurface::CursorRestricted {
                return V2Response::success(
                    id,
                    json!({
                        "commands": restricted_method_allowlist(),
                        "methods": restricted_method_allowlist(),
                        "surface": "cursor-restricted"
                    }),
                );
            }
            let methods = crate::control_registry::capability_methods();
            return V2Response::success(
                id,
                json!({ "commands": methods.clone(), "methods": methods }),
            );
        }
        "system.identify" => {
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::Identify {
                    caller: params.get("caller").cloned(),
                    reply,
                },
                rx,
            )
        }
        "window.present" => {
            let (reply, rx) = mpsc::channel();
            (ControlCommand::PresentWindow { reply }, rx)
        }
        "workspace.current" => {
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CurrentWorkspace { reply }, rx)
        }
        "workspace.list" | "list-workspaces" => {
            let (reply, rx) = mpsc::channel();
            (ControlCommand::ListWorkspaces { reply }, rx)
        }
        "pane.list" | "list-panes" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::ListPanes { target, reply }, rx)
        }
        "pane.surfaces" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::ListPaneSurfaces {
                    target,
                    pane_id: optional_string(params, &["pane_id", "id"]),
                    reply,
                },
                rx,
            )
        }
        "pane.create" | "new-pane" => {
            let request = match parse_create_pane_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CreatePane { request, reply }, rx)
        }
        "pane.action" => {
            let request = match parse_pane_action_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::PaneAction { request, reply }, rx)
        }
        "pane.focus" => {
            if crate::control_registry::wave1_mutations_disabled() {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!(
                            "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                        ),
                    ),
                );
            }
            let request = match parse_pane_focus_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::FocusPane { request, reply }, rx)
        }
        "pane.resize" | "resize-pane" => {
            if crate::control_registry::wave1_mutations_disabled() {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!(
                            "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                        ),
                    ),
                );
            }
            let request = match parse_pane_resize_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::ResizePane { request, reply }, rx)
        }
        "surface.split" => {
            if crate::control_registry::wave1_mutations_disabled() {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!(
                            "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                        ),
                    ),
                );
            }
            let request = match parse_surface_split_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::SplitSurface { request, reply }, rx)
        }
        "surface.focus" => {
            if crate::control_registry::wave1_mutations_disabled() {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!(
                            "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                        ),
                    ),
                );
            }
            let request = match parse_surface_focus_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::FocusSurface { request, reply }, rx)
        }
        "surface.close" => {
            if crate::control_registry::wave1_mutations_disabled() {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!(
                            "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                        ),
                    ),
                );
            }
            let request = match parse_surface_close_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CloseSurface { request, reply }, rx)
        }
        "cursor.pane_create_empty" => {
            let request = match parse_cursor_pane_create_empty_request(params) {
                Ok(request) => request,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CreatePane { request, reply }, rx)
        }
        "surface.list" | "list-panels" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::ListSurfaces { target, reply }, rx)
        }
        "surface.current" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CurrentSurface { target, reply }, rx)
        }
        "surface.health" | "surface-health" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let surface_hint = match optional_ref_handle(params, &["surface_id", "id"], "surface:")
            {
                Ok(surface_hint) => surface_hint,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::SurfaceHealth {
                    target,
                    surface_hint,
                    reply,
                },
                rx,
            )
        }
        "surface.read_text" | "read-screen" | "capture-pane" => {
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let surface_hint = match optional_ref_handle(params, &["surface_id", "id"], "surface:")
            {
                Ok(surface_hint) => surface_hint,
                Err(error) => return error_response(id, error),
            };
            let scrollback = match optional_bool(params, "scrollback") {
                Ok(scrollback) => scrollback.unwrap_or(false),
                Err(error) => return error_response(id, error),
            };
            let lines = match optional_index(params, "lines") {
                Ok(Some(0)) => {
                    return error_response(
                        id,
                        BridgeError::invalid_params("lines must be greater than 0"),
                    );
                }
                Ok(lines) => lines,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::ReadSurfaceText {
                    target,
                    surface_hint,
                    scrollback,
                    lines,
                    reply,
                },
                rx,
            )
        }
        "workspace.create" | "new-workspace" => {
            let command =
                match optional_terminal_text(params, &["command"], "workspace.create command") {
                    Ok(command) => command,
                    Err(error) => return error_response(id, error),
                };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::CreateWorkspace {
                    name: optional_string(params, &["name", "title"]),
                    cwd: optional_string(params, &["cwd"]),
                    command,
                    reply,
                },
                rx,
            )
        }
        "workspace.select" | "workspace.activate" | "activate-workspace" => {
            let target = match parse_required_workspace_target(params, true, method) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::SelectWorkspace { target, reply }, rx)
        }
        "workspace.rename" | "rename-workspace" => {
            let Some(title) = optional_string(params, &["title", "name"]) else {
                return error_response(
                    id,
                    BridgeError::invalid_params("workspace.rename requires title/name"),
                );
            };
            let target = match parse_optional_workspace_target(params, false) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::RenameWorkspace {
                    target,
                    title,
                    reply,
                },
                rx,
            )
        }
        "workspace.close" | "close-workspace" => {
            let target = match parse_optional_workspace_target(params, false) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::CloseWorkspace { target, reply }, rx)
        }
        "cursor.workspace_open_folder" => {
            let (name, cwd) = match parse_cursor_workspace_open_folder(params) {
                Ok(parsed) => parsed,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::CreateWorkspace {
                    name,
                    cwd: Some(cwd),
                    command: None,
                    reply,
                },
                rx,
            )
        }
        "surface.send_text" | "send-text" | "send" => {
            let Some(text) = optional_string(params, &["text"]) else {
                return error_response(
                    id,
                    BridgeError::invalid_params("surface.send_text requires text"),
                );
            };
            if let Err(error) = validate_terminal_text_param("surface.send_text text", &text) {
                return error_response(id, error);
            }
            // allow_name = true: lets agent-team peers address each other by
            // workspace name (e.g. `--workspace codex`) instead of UUID.
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::SendText {
                    target,
                    surface_hint: optional_string(params, &["surface_id"]),
                    text,
                    reply,
                },
                rx,
            )
        }
        "surface.send_key" | "send-key" => {
            let Some(key) = optional_string(params, &["key"]) else {
                return error_response(
                    id,
                    BridgeError::invalid_params("surface.send_key requires key"),
                );
            };
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::SendKey {
                    target,
                    surface_hint: optional_string(params, &["surface_id"]),
                    key,
                    reply,
                },
                rx,
            )
        }
        "notification.create" | "notify" => {
            // Title is required; subtitle and body are optional. This mirrors
            // cmux notify's shape (title/subtitle/body) and maps onto the
            // existing sidebar unread pipeline.
            let Some(title) = optional_string(params, &["title"]) else {
                return error_response(
                    id,
                    BridgeError::invalid_params("notification.create requires title"),
                );
            };
            let subtitle = optional_string(params, &["subtitle"]).unwrap_or_default();
            let body = optional_string(params, &["body", "message"]).unwrap_or_default();
            // allow_name = true: lets agent hooks target a peer by name.
            let target = match parse_optional_workspace_target(params, true) {
                Ok(target) => target,
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (
                ControlCommand::CreateNotification {
                    target,
                    title,
                    subtitle,
                    body,
                    reply,
                },
                rx,
            )
        }
        "notification.list" => {
            let unread_only = match optional_bool(params, "unread_only") {
                Ok(value) => value.unwrap_or(false),
                Err(error) => return error_response(id, error),
            };
            let (reply, rx) = mpsc::channel();
            (ControlCommand::ListNotifications { unread_only, reply }, rx)
        }
        _ => match crate::control_registry::route_class(method) {
            Some(RouteClass::CoreRead) => {
                debug_assert!(crate::control_registry::is_read_only_fallthrough(method));
                let (reply, rx) = mpsc::channel();
                return dispatch_queued(
                    id,
                    method,
                    ControlCommand::FallthroughRead {
                        method: method.to_string(),
                        params: Value::Object(params.clone()),
                        reply,
                    },
                    rx,
                    dispatch,
                );
            }
            Some(RouteClass::Wave1Mutation) => {
                let message = if crate::control_registry::wave1_mutations_disabled() {
                    format!(
                        "Wave-1 mutation route disabled by {WAVE1_MUTATION_KILL_SWITCH_ENV}: {method}"
                    )
                } else {
                    format!(
                        "Wave-1 mutation route classified but not wired to live GTK state yet: {method}"
                    )
                };
                return error_response(id, BridgeError::new(UNKNOWN_METHOD_CODE, message));
            }
            Some(RouteClass::Deferred) => {
                return error_response(
                    id,
                    BridgeError::new(
                        UNKNOWN_METHOD_CODE,
                        format!("PRD-E method deferred in Wave 1: {method}"),
                    ),
                );
            }
            Some(RouteClass::BridgeNative) | None => {
                return error_response(
                    id,
                    BridgeError::new(UNKNOWN_METHOD_CODE, format!("unknown method: {method}")),
                );
            }
        },
    };

    let (command, reply_rx) = queued;

    dispatch_queued(id, method, command, reply_rx, dispatch)
}

fn dispatch_queued(
    id: Option<Value>,
    method: &str,
    command: ControlCommand,
    reply_rx: mpsc::Receiver<BridgeResult>,
    dispatch: &dyn Fn(ControlCommand),
) -> V2Response {
    let timeout = control_command_timeout(method);
    dispatch_queued_with_timeout(id, method, command, reply_rx, dispatch, timeout)
}

fn dispatch_queued_with_timeout(
    id: Option<Value>,
    method: &str,
    command: ControlCommand,
    reply_rx: mpsc::Receiver<BridgeResult>,
    dispatch: &dyn Fn(ControlCommand),
    timeout: Duration,
) -> V2Response {
    dispatch(command);
    match reply_rx.recv_timeout(timeout) {
        Ok(Ok(result)) => V2Response::success(id, result),
        Ok(Err(error)) => error_response(id, error),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            error_response(id, control_command_timeout_error(method, timeout))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => error_response(
            id,
            BridgeError::internal(format!(
                "control command {method} reply channel disconnected"
            )),
        ),
    }
}

fn control_command_timeout(method: &str) -> Duration {
    match method {
        "pane.create" | "new-pane" => PANE_CREATE_CONTROL_COMMAND_TIMEOUT,
        _ => DEFAULT_CONTROL_COMMAND_TIMEOUT,
    }
}

fn control_command_timeout_error(method: &str, timeout: Duration) -> BridgeError {
    BridgeError::internal(format!(
        "control command {method} timed out after {} ms; outcome is unknown because the queued command may still complete; inspect current state before retrying",
        timeout.as_millis()
    ))
    .with_data(json!({
        "method": method,
        "outcome_unknown": true,
        "retry_safe": false,
        "timeout_ms": timeout.as_millis()
    }))
}

fn error_response(id: Option<Value>, error: BridgeError) -> V2Response {
    V2Response::error(id, error.code, error.message, error.data)
}

pub(crate) fn bridge_result_from_v2_response(response: V2Response) -> BridgeResult {
    if response.ok {
        return Ok(response.result.unwrap_or(Value::Null));
    }

    let Some(error) = response.error else {
        return Err(BridgeError::internal(
            "fallthrough response failed without an error payload",
        ));
    };
    let mut bridge_error = BridgeError::new(error.code, error.message);
    if let Some(data) = error.data {
        bridge_error = bridge_error.with_data(data);
    }
    Err(bridge_error)
}

fn dispatch_request_for_surface(
    input: &str,
    dispatch: &dyn Fn(ControlCommand),
    surface: MethodSurface,
) -> V2Response {
    match parse_request(input) {
        Ok(request) => handle_method(
            request.id,
            &request.method,
            request.params,
            dispatch,
            surface,
        ),
        Err(error) => error_response(None, error),
    }
}

#[cfg(test)]
fn dispatch_request(input: &str, dispatch: &dyn Fn(ControlCommand)) -> V2Response {
    dispatch_request_for_surface(input, dispatch, MethodSurface::Unrestricted)
}

fn handle_client(
    stream: UnixStream,
    dispatch: &(dyn Fn(ControlCommand) + Send + Sync + 'static),
    surface: MethodSurface,
) -> io::Result<()> {
    stream.set_read_timeout(Some(request_io::CLIENT_IDLE_TIMEOUT))?;
    let reader_stream = stream.try_clone()?;
    reader_stream.set_read_timeout(Some(request_io::CLIENT_IDLE_TIMEOUT))?;
    let mut reader = io::BufReader::new(reader_stream);
    let mut writer = stream;
    let mut line_buf = Vec::with_capacity(4096);

    loop {
        if !read_request_frame(&mut reader, &mut line_buf)? {
            return Ok(());
        }

        let input = std::str::from_utf8(&line_buf)
            .map(|line| line.trim_end_matches(['\n', '\r']))
            .unwrap_or("");
        if input.is_empty() {
            continue;
        }

        let response = dispatch_request_for_surface(input, dispatch, surface);
        let mut payload = serde_json::to_string(&response)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        payload.push('\n');
        writer.write_all(payload.as_bytes())?;
        writer.flush()?;
    }
}

struct ConnectionSlot {
    active_connections: Arc<AtomicUsize>,
}

impl ConnectionSlot {
    fn try_acquire(active_connections: Arc<AtomicUsize>) -> Option<Self> {
        active_connections
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < request_io::MAX_CONNECTIONS).then_some(current + 1)
            })
            .ok()?;
        Some(Self { active_connections })
    }
}

impl Drop for ConnectionSlot {
    fn drop(&mut self) {
        self.active_connections.fetch_sub(1, Ordering::AcqRel);
    }
}

fn spawn_control_listener(
    name: &'static str,
    path: PathBuf,
    surface: MethodSurface,
    control_mode: SocketControlMode,
    dispatch: Arc<dyn Fn(ControlCommand) + Send + Sync + 'static>,
) -> io::Result<()> {
    let listener = bind_listener(
        &path,
        SocketMode::Runtime,
        control_mode.requires_owner_only_socket(),
    )?;

    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            run_control_listener(listener, path, surface, control_mode, dispatch);
        })
        .map_err(|error| io::Error::other(format!("failed to spawn {name} thread: {error}")))?;
    Ok(())
}

fn run_control_listener(
    listener: UnixListener,
    path: PathBuf,
    surface: MethodSurface,
    control_mode: SocketControlMode,
    dispatch: Arc<dyn Fn(ControlCommand) + Send + Sync + 'static>,
) {
    eprintln!("limux: control socket at {}", path.display());
    let active_connections = Arc::new(AtomicUsize::new(0));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let Some(slot) = ConnectionSlot::try_acquire(active_connections.clone()) else {
                    eprintln!("limux: rejecting control client, too many active connections");
                    continue;
                };
                let peer = match auth::authorize_peer(&stream, control_mode) {
                    Ok(peer) => peer,
                    Err(error) => {
                        eprintln!("limux: rejected control client: {error}");
                        continue;
                    }
                };
                let dispatch = dispatch.clone();
                std::thread::Builder::new()
                    .name("limux-ctrl-conn".into())
                    .spawn(move || {
                        let _slot = slot;
                        if let Err(error) = handle_client(stream, dispatch.as_ref(), surface) {
                            eprintln!(
                                "limux: control connection error for pid={} uid={}: {error}",
                                peer.pid, peer.uid
                            );
                        }
                    })
                    .ok();
            }
            Err(error) => {
                eprintln!("limux: control accept error: {error}");
            }
        }
    }
}

/// Start the control socket server in a background thread and dispatch each
/// command onto the GTK main context.
pub fn start(dispatch: fn(ControlCommand)) {
    let context = glib::MainContext::default();
    let dispatch = std::sync::Arc::new(move |command: ControlCommand| {
        context.invoke(move || dispatch(command));
    });

    let path = resolve_socket_path(None, SocketMode::Runtime);
    let cursor_path = cursor_restricted_socket_path(&path);
    let control_mode = SocketControlMode::from_env();
    if path == cursor_path {
        eprintln!(
            "limux: runtime socket path already targets Cursor restricted surface; binding restricted listener only"
        );
        if let Err(error) = spawn_control_listener(
            "limux-cursor-control",
            cursor_path,
            MethodSurface::CursorRestricted,
            control_mode,
            dispatch,
        ) {
            eprintln!("limux: control socket bind failed: {error}");
        }
        return;
    }
    if let Err(error) = spawn_control_listener(
        "limux-control",
        path.clone(),
        MethodSurface::Unrestricted,
        control_mode,
        dispatch.clone(),
    ) {
        eprintln!(
            "limux: control socket bind failed ({}): {error}; not starting Cursor restricted socket",
            path.display()
        );
        return;
    }
    if let Err(error) = spawn_control_listener(
        "limux-cursor-control",
        cursor_path.clone(),
        MethodSurface::CursorRestricted,
        control_mode,
        dispatch,
    ) {
        eprintln!(
            "limux: cursor control socket bind failed ({}): {error}",
            cursor_path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static WAVE1_ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvVarGuard {
        fn clear(key: &'static str) -> Self {
            let lock = WAVE1_ENV_TEST_LOCK.lock().expect("env test lock poisoned");
            let previous = std::env::var_os(key);
            std::env::remove_var(key);
            Self {
                key,
                previous,
                _lock: lock,
            }
        }

        fn set(key: &'static str, value: &str) -> Self {
            let lock = WAVE1_ENV_TEST_LOCK.lock().expect("env test lock poisoned");
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self {
                key,
                previous,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn parses_v2_request_directly() {
        let request = parse_request(r#"{"id":"1","method":"system.ping","params":{}}"#)
            .expect("v2 request should parse");
        assert_eq!(request.id, Some(Value::String("1".to_string())));
        assert_eq!(request.method, "system.ping");
    }

    #[test]
    fn parses_v1_request_envelope() {
        let request = parse_request(r#"{"command":"workspace.create","args":{"cwd":"/tmp"}}"#)
            .expect("v1 request should parse");
        assert_eq!(request.method, "workspace.create");
        assert_eq!(request.params["cwd"], "/tmp");
    }

    #[test]
    fn workspace_target_prefers_handle_over_index() {
        let params = json!({
            "workspace_id": "workspace:abc",
            "index": 2
        });
        let target =
            parse_optional_workspace_target(params.as_object().expect("object params"), true)
                .expect("target should parse");
        assert_eq!(target, WorkspaceTarget::Handle("workspace:abc".to_string()));
    }

    #[test]
    fn workspace_target_treats_cli_workspace_id_as_name_when_allowed() {
        let params = json!({
            "workspace_id": "claude"
        });
        let target =
            parse_optional_workspace_target(params.as_object().expect("object params"), true)
                .expect("target should parse");
        assert_eq!(target, WorkspaceTarget::Name("claude".to_string()));
    }

    #[test]
    fn workspace_target_preserves_raw_uuid_workspace_ids_when_names_are_allowed() {
        let workspace_id = "2b8b5ca4-0200-4433-9f7c-d5c9f725be50";
        let params = json!({
            "workspace_id": workspace_id
        });
        let target =
            parse_optional_workspace_target(params.as_object().expect("object params"), true)
                .expect("target should parse");
        assert_eq!(target, WorkspaceTarget::Handle(workspace_id.to_string()));
    }

    #[test]
    fn workspace_select_requires_explicit_target() {
        let params = Map::new();
        let error = parse_required_workspace_target(&params, true, "workspace.select")
            .expect_err("workspace.select should require a target");
        assert_eq!(error.code, INVALID_PARAMS_CODE);
    }

    #[test]
    fn pane_create_contract_accepts_raw_and_ref_targets() {
        let params = json!({
            "workspace_id": 7,
            "surface_id": "surface:11",
            "pane_id": "pane:12",
            "direction": "left",
            "type": "terminal",
            "command": "claude"
        });
        let request = parse_create_pane_request(params.as_object().expect("object params"))
            .expect("pane.create request should parse");

        assert_eq!(request.target, WorkspaceTarget::Handle("7".to_string()));
        assert_eq!(request.source_surface_id, Some("11".to_string()));
        assert_eq!(request.source_pane_id, Some("12".to_string()));
        assert_eq!(request.direction, PaneCreateDirection::Left);
        assert_eq!(request.pane_type, PaneCreateType::Terminal);
        assert_eq!(request.command, Some("claude".to_string()));
    }

    #[test]
    fn pane_create_contract_rejects_invalid_direction_and_type() {
        let bad_direction = json!({ "direction": "diagonal" });
        let error = parse_create_pane_request(bad_direction.as_object().expect("object params"))
            .expect_err("invalid direction should fail");
        assert_eq!(error.code, INVALID_PARAMS_CODE);

        let bad_type = json!({ "type": "webview" });
        let error = parse_create_pane_request(bad_type.as_object().expect("object params"))
            .expect_err("invalid type should fail");
        assert_eq!(error.code, INVALID_PARAMS_CODE);
    }

    #[test]
    fn pane_create_contract_rejects_deferred_browser_fields() {
        let browser = json!({ "type": "browser" });
        let error = parse_create_pane_request(browser.as_object().expect("object params"))
            .expect_err("browser panes are deferred");
        assert_eq!(error.code, INVALID_PARAMS_CODE);

        let url = json!({ "url": "https://example.com" });
        let error = parse_create_pane_request(url.as_object().expect("object params"))
            .expect_err("url is browser-only");
        assert_eq!(error.code, INVALID_PARAMS_CODE);
    }

    #[test]
    fn pane_create_contract_rejects_disallowed_terminal_control_command() {
        let bad = json!({ "command": "codex\u{1b}[31m" });
        let error = parse_create_pane_request(bad.as_object().expect("object params"))
            .expect_err("terminal control command should fail");

        assert_eq!(error.code, INVALID_PARAMS_CODE);
        assert!(error.message.contains("pane.create command"));
        assert!(error.message.contains("U+001B"));
    }

    #[test]
    fn pane_create_route_queues_create_pane_command() {
        let response = dispatch_request(
            r#"{"id":1,"method":"pane.create","params":{"name":"claude","surface_id":"surface:4:tab","direction":"down","command":"codex"}}"#,
            &|command| match command {
                ControlCommand::CreatePane { request, reply } => {
                    assert_eq!(request.target, WorkspaceTarget::Name("claude".to_string()));
                    assert_eq!(request.source_surface_id, Some("4:tab".to_string()));
                    assert_eq!(request.direction, PaneCreateDirection::Down);
                    assert_eq!(request.command, Some("codex".to_string()));
                    let _ = reply.send(Ok(json!({
                        "pane_id": "9",
                        "pane_ref": "pane:9",
                        "surface_id": "9:tab",
                        "surface_ref": "surface:9:tab"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("pane.create should return a result");
        assert_eq!(result["pane_ref"], "pane:9");
        assert_eq!(result["surface_ref"], "surface:9:tab");
    }

    #[test]
    fn pane_create_timeout_exceeds_terminal_readiness_window() {
        let readiness_window = Duration::from_millis(
            crate::window::PANE_CREATE_COMMAND_READY_INTERVAL_MS
                * u64::from(crate::window::PANE_CREATE_COMMAND_READY_ATTEMPTS)
                + crate::window::PANE_CREATE_COMMAND_SUBMIT_DELAY_MS,
        );

        assert!(
            control_command_timeout("pane.create") >= readiness_window + Duration::from_secs(5),
            "pane.create needs scheduling margin beyond its terminal readiness window"
        );
        assert_eq!(
            control_command_timeout("workspace.list"),
            Duration::from_secs(5)
        );
        assert_eq!(
            control_command_timeout("new-pane"),
            PANE_CREATE_CONTROL_COMMAND_TIMEOUT
        );
    }

    #[test]
    fn control_timeout_reports_unknown_outcome_and_unsafe_retry() {
        let error = control_command_timeout_error("pane.create", Duration::from_secs(15));

        assert_eq!(error.code, INTERNAL_ERROR_CODE);
        assert!(error
            .message
            .contains("inspect current state before retrying"));
        assert_eq!(
            error.data,
            Some(json!({
                "method": "pane.create",
                "outcome_unknown": true,
                "retry_safe": false,
                "timeout_ms": 15_000
            }))
        );
    }

    #[test]
    fn queued_dispatch_timeout_serializes_unsafe_retry_metadata() {
        let (reply, reply_rx) = mpsc::channel();
        let held_command = std::cell::RefCell::new(None);
        let response = dispatch_queued_with_timeout(
            Some(json!(1)),
            "pane.create",
            ControlCommand::PresentWindow { reply },
            reply_rx,
            &|command| {
                held_command.replace(Some(command));
            },
            Duration::from_millis(1),
        );

        let error = response.error.expect("timeout response error");
        assert_eq!(error.code, INTERNAL_ERROR_CODE);
        assert_eq!(error.data.as_ref().unwrap()["outcome_unknown"], true);
        assert_eq!(error.data.as_ref().unwrap()["retry_safe"], false);
        assert_eq!(error.data.as_ref().unwrap()["method"], "pane.create");
    }

    #[test]
    fn queued_dispatch_disconnection_is_not_reported_as_unknown_outcome() {
        let (reply, reply_rx) = mpsc::channel();
        let response = dispatch_queued_with_timeout(
            Some(json!(2)),
            "workspace.list",
            ControlCommand::PresentWindow { reply },
            reply_rx,
            &drop,
            Duration::from_millis(1),
        );

        let error = response.error.expect("disconnection response error");
        assert!(error.message.contains("reply channel disconnected"));
        assert_eq!(error.data, None);
    }

    #[test]
    fn pane_create_route_rejects_invalid_params_before_dispatch() {
        let response = dispatch_request(
            r#"{"id":1,"method":"new-pane","params":{"direction":"diagonal"}}"#,
            &|command| panic!("invalid pane.create should not dispatch: {command:?}"),
        );

        assert_eq!(response.result, None);
        assert_eq!(
            response.error.as_ref().map(|error| error.code),
            Some(INVALID_PARAMS_CODE)
        );
    }

    #[test]
    fn pane_focus_route_queues_focus_pane_command() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"pane.focus","params":{"workspace_id":"workspace:dev","pane_id":"pane:7"}}"#,
            &|command| match command {
                ControlCommand::FocusPane { request, reply } => {
                    assert_eq!(
                        request.target,
                        WorkspaceTarget::Handle("workspace:dev".to_string())
                    );
                    assert_eq!(request.pane_id, "7");
                    let _ = reply.send(Ok(json!({
                        "ok": true,
                        "workspace_id": "dev",
                        "workspace_ref": "workspace:dev",
                        "pane_id": "7",
                        "pane_ref": "pane:7"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("pane.focus should return a result");
        assert_eq!(result["ok"], true);
        assert_eq!(result["pane_ref"], "pane:7");
    }

    #[test]
    fn pane_focus_treats_id_as_pane_id_not_workspace_id() {
        let params = json!({ "id": "pane:12" });
        let request = parse_pane_focus_request(params.as_object().expect("object params"))
            .expect("pane.focus should parse id as pane id");

        assert_eq!(request.target, WorkspaceTarget::Active);
        assert_eq!(request.pane_id, "12");
    }

    #[test]
    fn pane_resize_route_queues_resize_pane_command() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"pane.resize","params":{"workspace_id":"workspace:dev","pane_id":"pane:7","direction":"left","amount":24}}"#,
            &|command| match command {
                ControlCommand::ResizePane { request, reply } => {
                    assert_eq!(
                        request.target,
                        WorkspaceTarget::Handle("workspace:dev".to_string())
                    );
                    assert_eq!(request.pane_id, "7");
                    assert_eq!(request.direction, PaneResizeDirection::Left);
                    assert_eq!(request.amount, 24.0);
                    let _ = reply.send(Ok(json!({
                        "ok": true,
                        "pane_id": "7",
                        "pane_ref": "pane:7",
                        "direction": "left",
                        "amount": 24.0,
                        "frame": { "width": 320, "height": 200 }
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("pane.resize should return a result");
        assert_eq!(result["pane_ref"], "pane:7");
        assert_eq!(result["frame"]["width"], 320);
    }

    #[test]
    fn resize_pane_alias_uses_same_parser_and_treats_id_as_pane_id() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"resize-pane","params":{"id":"pane:9","direction":"down"}}"#,
            &|command| match command {
                ControlCommand::ResizePane { request, reply } => {
                    assert_eq!(request.target, WorkspaceTarget::Active);
                    assert_eq!(request.pane_id, "9");
                    assert_eq!(request.direction, PaneResizeDirection::Down);
                    assert_eq!(request.amount, 1.0);
                    let _ = reply.send(Ok(json!({
                        "ok": true,
                        "pane_id": "9",
                        "pane_ref": "pane:9",
                        "direction": "down",
                        "amount": 1.0,
                        "frame": { "width": 100, "height": 101 }
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        assert_eq!(response.result.expect("result")["pane_id"], "9");
    }

    #[test]
    fn surface_split_route_queues_split_surface_command() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.split","params":{"workspace_id":"workspace:dev","surface_id":"surface:4:tab","direction":"up","title":"Review"}}"#,
            &|command| match command {
                ControlCommand::SplitSurface { request, reply } => {
                    assert_eq!(
                        request.target,
                        WorkspaceTarget::Handle("workspace:dev".to_string())
                    );
                    assert_eq!(request.source_surface_id, Some("4:tab".to_string()));
                    assert_eq!(request.direction, PaneCreateDirection::Up);
                    assert_eq!(request.title, Some("Review".to_string()));
                    let _ = reply.send(Ok(json!({
                        "surface_id": "9:tab",
                        "surface_ref": "surface:9:tab",
                        "pane_id": "9",
                        "pane_ref": "pane:9"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response
            .result
            .expect("surface.split should return a result");
        assert_eq!(result["surface_ref"], "surface:9:tab");
    }

    #[test]
    fn surface_mutation_routes_reject_empty_surface_handles_before_dispatch() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        for (method, params) in [
            ("surface.split", json!({ "surface_id": "surface:" })),
            ("surface.focus", json!({ "surface": "   " })),
            ("surface.close", json!({ "id": "surface:   " })),
        ] {
            let request = json!({
                "id": 1,
                "method": method,
                "params": params
            })
            .to_string();
            let response = dispatch_request(&request, &|command| {
                panic!("{method} with empty surface handle should not dispatch: {command:?}")
            });

            assert_eq!(response.result, None, "{method} should fail");
            let error = response.error.expect("error");
            assert_eq!(error.code, INVALID_PARAMS_CODE, "{method} code");
            assert!(error.message.contains(method), "{method} message");
            assert!(
                error.message.contains("must not be empty"),
                "{method} should reject empty handles"
            );
        }
    }

    #[test]
    fn surface_focus_treats_id_as_surface_id_not_workspace_id() {
        let params = json!({ "id": "surface:7:tab" });
        let request = parse_surface_focus_request(params.as_object().expect("object params"))
            .expect("surface.focus should parse id as surface id");

        assert_eq!(request.target, WorkspaceTarget::Active);
        assert_eq!(request.surface_id, "7:tab");
    }

    #[test]
    fn surface_focus_route_queues_focus_surface_command() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.focus","params":{"name":"agents","surface_id":"surface:7:tab"}}"#,
            &|command| match command {
                ControlCommand::FocusSurface { request, reply } => {
                    assert_eq!(request.target, WorkspaceTarget::Name("agents".to_string()));
                    assert_eq!(request.surface_id, "7:tab");
                    let _ = reply.send(Ok(json!({
                        "ok": true,
                        "surface_id": "7:tab",
                        "surface_ref": "surface:7:tab"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response
            .result
            .expect("surface.focus should return a result");
        assert_eq!(result["surface_ref"], "surface:7:tab");
    }

    #[test]
    fn surface_close_route_queues_close_surface_command() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.close","params":{"index":2,"surface_id":"surface:7:tab"}}"#,
            &|command| match command {
                ControlCommand::CloseSurface { request, reply } => {
                    assert_eq!(request.target, WorkspaceTarget::Index(2));
                    assert_eq!(request.surface_id, Some("7:tab".to_string()));
                    let _ = reply.send(Ok(json!({
                        "ok": true,
                        "surface_id": "7:tab",
                        "surface_ref": "surface:7:tab"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response
            .result
            .expect("surface.close should return a result");
        assert_eq!(result["surface_ref"], "surface:7:tab");
    }

    #[test]
    fn wired_wave1_mutation_routes_are_hidden_and_blocked_by_kill_switch() {
        let _env_guard =
            EnvVarGuard::set(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV, "1");

        let capabilities = dispatch_request(
            r#"{"id":1,"method":"system.capabilities","params":{}}"#,
            &|command| panic!("system.capabilities should not dispatch: {command:?}"),
        );
        let methods = capabilities.result.expect("capabilities result")["methods"]
            .as_array()
            .expect("methods array")
            .clone();
        for method in [
            "pane.focus",
            "pane.resize",
            "resize-pane",
            "surface.split",
            "surface.focus",
            "surface.close",
        ] {
            assert!(
                !methods.iter().any(|entry| entry == method),
                "{method} should be hidden when Wave-1 mutations are disabled"
            );
        }

        let response = dispatch_request(
            r#"{"id":1,"method":"surface.close","params":{"surface_id":"surface:7:tab"}}"#,
            &|command| panic!("disabled surface.close should not dispatch: {command:?}"),
        );

        assert_eq!(response.result, None);
        let error = response.error.expect("disabled route should return error");
        assert_eq!(error.code, UNKNOWN_METHOD_CODE);
        assert!(error
            .message
            .contains(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV));
    }

    #[test]
    fn cursor_pane_create_empty_rejects_command_payload_fields_before_dispatch() {
        for field in ["command", "text", "key", "paste", "shell", "pty", "raw_pty"] {
            let mut params = Map::new();
            params.insert(field.to_string(), json!("unsafe"));
            let request = json!({
                "id": 1,
                "method": "cursor.pane_create_empty",
                "params": Value::Object(params)
            })
            .to_string();

            let response = dispatch_request(&request, &|command| {
                panic!("cursor.pane_create_empty with {field} should not dispatch: {command:?}")
            });

            assert_eq!(response.result, None);
            assert_eq!(
                response.error.as_ref().map(|error| error.code),
                Some(INVALID_PARAMS_CODE),
                "{field} should be rejected"
            );
            let message = &response.error.as_ref().expect("error").message;
            assert!(message.contains("cursor.pane_create_empty"));
            assert!(message.contains(field));
        }
    }

    #[test]
    fn surface_health_route_accepts_surface_refs() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.health","params":{"workspace_id":"codex","surface_id":"surface:4:tab"}}"#,
            &|command| match command {
                ControlCommand::SurfaceHealth {
                    target,
                    surface_hint,
                    reply,
                } => {
                    assert_eq!(target, WorkspaceTarget::Name("codex".to_string()));
                    assert_eq!(surface_hint, Some("4:tab".to_string()));
                    let _ = reply.send(Ok(json!({ "surfaces": [] })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        assert!(response.result.is_some());
    }

    #[test]
    fn surface_send_text_route_rejects_disallowed_terminal_control_before_dispatch() {
        let request = json!({
            "id": 1,
            "method": "surface.send_text",
            "params": { "surface_id": "surface:9:tab", "text": "hello\u{1b}[31m" }
        })
        .to_string();

        let response = dispatch_request(&request, &|command| {
            panic!("invalid surface.send_text should not dispatch: {command:?}")
        });

        assert_eq!(response.result, None);
        assert_eq!(
            response.error.as_ref().map(|error| error.code),
            Some(INVALID_PARAMS_CODE)
        );
        assert!(response
            .error
            .as_ref()
            .expect("error")
            .message
            .contains("surface.send_text text"));
    }

    #[test]
    fn read_text_route_accepts_capture_alias_and_surface_refs() {
        let response = dispatch_request(
            r#"{"id":1,"method":"capture-pane","params":{"surface_id":"surface:9:tab"}}"#,
            &|command| match command {
                ControlCommand::ReadSurfaceText {
                    target,
                    surface_hint,
                    reply,
                    ..
                } => {
                    assert_eq!(target, WorkspaceTarget::Active);
                    assert_eq!(surface_hint, Some("9:tab".to_string()));
                    let _ = reply.send(Ok(json!({ "text": "ready" })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        assert_eq!(response.result.expect("result")["text"], "ready");
    }

    #[test]
    fn read_text_route_forwards_scrollback_and_line_limit() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.read_text","params":{"surface_id":"surface:9:tab","scrollback":true,"lines":120}}"#,
            &|command| match command {
                ControlCommand::ReadSurfaceText {
                    target,
                    surface_hint,
                    scrollback,
                    lines,
                    reply,
                } => {
                    assert_eq!(target, WorkspaceTarget::Active);
                    assert_eq!(surface_hint, Some("9:tab".to_string()));
                    assert!(scrollback);
                    assert_eq!(lines, Some(120));
                    let _ = reply.send(Ok(json!({ "text": "ready" })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        assert_eq!(response.result.expect("result")["text"], "ready");
    }

    #[test]
    fn surface_read_text_method_stays_on_live_bridge_route() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.read_text","params":{"surface_id":"surface:9:tab"}}"#,
            &|command| match command {
                ControlCommand::ReadSurfaceText {
                    target,
                    surface_hint,
                    reply,
                    ..
                } => {
                    assert_eq!(target, WorkspaceTarget::Active);
                    assert_eq!(surface_hint, Some("9:tab".to_string()));
                    let _ = reply.send(Ok(json!({ "text": "live viewport text" })));
                }
                other => panic!("surface.read_text must stay live-bridge native: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        assert_eq!(
            response.result.expect("result")["text"],
            "live viewport text"
        );
    }

    #[test]
    fn surface_current_routes_to_live_bridge_surface_payload() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.current","params":{"workspace_id":"workspace:dev"}}"#,
            &|command| match command {
                ControlCommand::CurrentSurface { target, reply } => {
                    assert_eq!(target, WorkspaceTarget::Handle("workspace:dev".to_string()));
                    let _ = reply.send(Ok(json!({
                        "surface_id": "9:tab",
                        "surface_ref": "surface:9:tab",
                        "pane_id": "9",
                        "pane_ref": "pane:9",
                        "surface": {
                            "id": "9:tab",
                            "pane_id": "9",
                            "title": "terminal",
                            "text": "",
                            "panel_type": "terminal",
                            "developer_tools_visible": false,
                            "pinned": false,
                            "unread": false,
                            "flash_count": 0,
                            "refresh_count": 0
                        }
                    })));
                }
                other => panic!("surface.current must use live GTK ids: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["surface_id"], "9:tab");
        assert_eq!(result["surface_ref"], "surface:9:tab");
        assert_eq!(result["surface"]["id"], "9:tab");
    }

    #[test]
    fn notification_list_routes_to_live_bridge_notifications() {
        let response = dispatch_request(
            r#"{"id":1,"method":"notification.list","params":{"unread_only":true}}"#,
            &|command| match command {
                ControlCommand::ListNotifications { unread_only, reply } => {
                    assert!(unread_only);
                    let _ = reply.send(Ok(json!({
                        "notifications": [
                            {
                                "workspace_id": "dev",
                                "workspace_ref": "workspace:dev",
                                "unread": true
                            }
                        ]
                    })));
                }
                other => {
                    panic!("notification.list must use live GTK notification state: {other:?}")
                }
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["notifications"][0]["workspace_id"], "dev");
    }

    #[test]
    fn window_list_routes_to_read_only_core_fallthrough() {
        let response = dispatch_request(
            r#"{"id":1,"method":"window.list","params":{}}"#,
            &|command| match command {
                ControlCommand::FallthroughRead {
                    method,
                    params,
                    reply,
                } => {
                    assert_eq!(method, "window.list");
                    assert_eq!(params, json!({}));
                    let _ = reply.send(Ok(json!({
                        "windows": [
                            {
                                "id": 7,
                                "title": "dev",
                                "workspace_id": 3,
                                "current": true,
                                "pane_count": 1
                            }
                        ]
                    })));
                }
                other => panic!("window.list should use read-only fallthrough: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["windows"][0]["title"], "dev");
    }

    #[test]
    fn window_current_routes_to_read_only_core_fallthrough() {
        let response = dispatch_request(
            r#"{"id":1,"method":"window.current","params":{}}"#,
            &|command| match command {
                ControlCommand::FallthroughRead {
                    method,
                    params,
                    reply,
                } => {
                    assert_eq!(method, "window.current");
                    assert_eq!(params, json!({}));
                    let _ = reply.send(Ok(json!({
                        "window_id": "00007",
                        "window_ref": "window:00007",
                        "window": {
                            "id": 7,
                            "title": "dev",
                            "pane_count": 1,
                            "current_pane_id": 9
                        }
                    })));
                }
                other => panic!("window.current should use read-only fallthrough: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["window"]["title"], "dev");
    }

    #[test]
    fn workspace_create_route_rejects_disallowed_terminal_control_command_before_dispatch() {
        let request = json!({
            "id": 1,
            "method": "workspace.create",
            "params": { "command": "claude\u{7}" }
        })
        .to_string();

        let response = dispatch_request(&request, &|command| {
            panic!("invalid workspace.create should not dispatch: {command:?}")
        });

        assert_eq!(response.result, None);
        assert_eq!(
            response.error.as_ref().map(|error| error.code),
            Some(INVALID_PARAMS_CODE)
        );
        assert!(response
            .error
            .as_ref()
            .expect("error")
            .message
            .contains("workspace.create command"));
    }

    fn dispatch_restricted_request(input: &str, dispatch: &dyn Fn(ControlCommand)) -> V2Response {
        dispatch_request_for_surface(input, dispatch, MethodSurface::CursorRestricted)
    }

    #[test]
    fn unrestricted_capabilities_include_read_only_fallthrough_methods() {
        let response = dispatch_request(
            r#"{"id":1,"method":"system.capabilities","params":{}}"#,
            &|command| panic!("system.capabilities should not dispatch: {command:?}"),
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("capabilities result");
        let methods = result["methods"].as_array().expect("methods array");
        assert!(methods.iter().any(|method| method == "window.list"));
        assert!(methods.iter().any(|method| method == "window.current"));
    }

    #[test]
    fn unrestricted_capabilities_do_not_advertise_unwired_wave1_or_deferred_routes() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"system.capabilities","params":{}}"#,
            &|command| panic!("system.capabilities should not dispatch: {command:?}"),
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("capabilities result");
        let methods = result["methods"].as_array().expect("methods array");
        for method in [
            "pane.focus",
            "pane.resize",
            "resize-pane",
            "surface.split",
            "surface.focus",
            "surface.close",
        ] {
            assert!(
                methods.iter().any(|entry| entry == method),
                "{method} should be advertised after the live GTK route is wired"
            );
        }
        let method = "notification.clear";
        assert!(
            !methods.iter().any(|entry| entry == method),
            "{method} must stay out of capabilities until wired to live GTK state"
        );
        for method in ["pane.swap", "surface.move", "window.create"] {
            assert!(
                !methods.iter().any(|entry| entry == method),
                "{method} must stay out of capabilities while explicitly deferred"
            );
        }
    }

    #[test]
    fn wave1_mutation_route_returns_explicit_unwired_error() {
        let _env_guard =
            EnvVarGuard::clear(crate::control_registry::WAVE1_MUTATION_KILL_SWITCH_ENV);
        let response = dispatch_request(
            r#"{"id":1,"method":"notification.clear","params":{"id":"notification:7"}}"#,
            &|command| panic!("unwired Wave-1 route should not dispatch: {command:?}"),
        );

        assert_eq!(response.result, None);
        let error = response.error.expect("error");
        assert_eq!(error.code, UNKNOWN_METHOD_CODE);
        assert!(error.message.contains("Wave-1 mutation"));
        assert!(error.message.contains("live GTK"));
        assert!(error.message.contains("notification.clear"));
    }

    #[test]
    fn deferred_route_returns_explicit_prd_e_error() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.move","params":{}}"#,
            &|command| panic!("deferred route should not dispatch: {command:?}"),
        );

        assert_eq!(response.result, None);
        let error = response.error.expect("error");
        assert_eq!(error.code, UNKNOWN_METHOD_CODE);
        assert!(error.message.contains("deferred"));
        assert!(error.message.contains("surface.move"));
    }

    #[test]
    fn cursor_restricted_socket_path_is_sibling_of_runtime_socket() {
        assert_eq!(
            cursor_restricted_socket_path(Path::new("/tmp/limux.sock")),
            PathBuf::from("/tmp/limux.cursor.sock")
        );
        assert_eq!(
            cursor_restricted_socket_path(Path::new("/tmp/custom")),
            PathBuf::from("/tmp/custom.cursor")
        );
        assert_eq!(
            cursor_restricted_socket_path(Path::new("/tmp/limux.cursor.sock")),
            PathBuf::from("/tmp/limux.cursor.sock")
        );
        assert_eq!(
            cursor_restricted_socket_path(Path::new("/tmp/custom.cursor")),
            PathBuf::from("/tmp/custom.cursor")
        );
    }

    #[test]
    fn restricted_capabilities_returns_cursor_allowlist() {
        let response = dispatch_restricted_request(
            r#"{"id":1,"method":"system.capabilities","params":{}}"#,
            &|command| panic!("system.capabilities should not dispatch: {command:?}"),
        );

        assert_eq!(response.error, None);
        let result = response.result.expect("capabilities result");
        assert_eq!(result["surface"], "cursor-restricted");
        assert_eq!(
            result["methods"],
            json!([
                "workspace.list",
                "workspace.select",
                "window.present",
                "cursor.pane_create_empty",
                "surface.read_text",
                "cursor.workspace_open_folder"
            ])
        );
    }

    #[test]
    fn restricted_surface_rejects_forbidden_terminal_methods_and_aliases() {
        for method in [
            "surface.send_text",
            "surface.send_key",
            "send",
            "pane.create.command",
        ] {
            let request = json!({
                "id": 1,
                "method": method,
                "params": {}
            })
            .to_string();
            let response = dispatch_restricted_request(&request, &|command| {
                panic!("restricted method should not dispatch: {command:?}")
            });

            assert_eq!(response.result, None);
            assert_eq!(
                response.error.as_ref().map(|error| error.code),
                Some(UNKNOWN_METHOD_CODE),
                "{method} should be rejected"
            );
        }
    }

    #[test]
    fn restricted_surface_rejects_unexpected_payload_fields() {
        for request in [
            json!({
                "id": 1,
                "method": "workspace.list",
                "params": { "unexpected": true }
            }),
            json!({
                "id": 2,
                "method": "cursor.pane_create_empty",
                "params": { "command": "codex" }
            }),
            json!({
                "id": 3,
                "method": "surface.read_text",
                "params": { "scrollback": true }
            }),
        ] {
            let response = dispatch_restricted_request(&request.to_string(), &|command| {
                panic!("restricted request with unexpected fields should not dispatch: {command:?}")
            });
            assert_eq!(response.result, None);
            assert_eq!(
                response.error.as_ref().map(|error| error.code),
                Some(INVALID_PARAMS_CODE)
            );
        }
    }

    #[test]
    fn restricted_surface_allows_pinned_cursor_methods() {
        let workspace_list = dispatch_restricted_request(
            r#"{"id":1,"method":"workspace.list","params":{}}"#,
            &|command| match command {
                ControlCommand::ListWorkspaces { reply } => {
                    let _ = reply.send(Ok(json!({ "workspaces": [] })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );
        assert_eq!(workspace_list.error, None);

        let workspace_select = dispatch_restricted_request(
            r#"{"id":2,"method":"workspace.select","params":{"workspace_id":"workspace:abc"}}"#,
            &|command| match command {
                ControlCommand::SelectWorkspace { target, reply } => {
                    assert_eq!(target, WorkspaceTarget::Handle("workspace:abc".to_string()));
                    let _ = reply.send(Ok(json!({ "selected": true })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );
        assert_eq!(workspace_select.error, None);

        let window_present = dispatch_restricted_request(
            r#"{"id":3,"method":"window.present","params":{}}"#,
            &|command| match command {
                ControlCommand::PresentWindow { reply } => {
                    let _ = reply.send(Ok(json!({
                        "state": "presentation-requested",
                        "success_confirmed": false
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );
        assert_eq!(window_present.error, None);
        let result = window_present.result.expect("window.present result");
        assert_eq!(result["state"], "presentation-requested");
        assert_eq!(result["success_confirmed"], false);

        let pane_create = dispatch_restricted_request(
            r#"{"id":4,"method":"cursor.pane_create_empty","params":{"surface_id":"surface:4:tab","direction":"down"}}"#,
            &|command| match command {
                ControlCommand::CreatePane { request, reply } => {
                    assert_eq!(request.source_surface_id, Some("4:tab".to_string()));
                    assert_eq!(request.direction, PaneCreateDirection::Down);
                    assert_eq!(request.pane_type, PaneCreateType::Terminal);
                    assert_eq!(request.command, None);
                    let _ = reply.send(Ok(json!({
                        "pane_ref": "pane:9",
                        "surface_ref": "surface:9:tab"
                    })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );
        assert_eq!(pane_create.error, None);

        let read_text = dispatch_restricted_request(
            r#"{"id":5,"method":"surface.read_text","params":{"surface_id":"surface:9:tab"}}"#,
            &|command| match command {
                ControlCommand::ReadSurfaceText {
                    target,
                    surface_hint,
                    reply,
                    ..
                } => {
                    assert_eq!(target, WorkspaceTarget::Active);
                    assert_eq!(surface_hint, Some("9:tab".to_string()));
                    let _ = reply.send(Ok(json!({ "text": "ready" })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );
        assert_eq!(read_text.error, None);

        let expected_cwd = std::env::current_dir()
            .expect("current dir")
            .canonicalize()
            .expect("canonical current dir")
            .to_string_lossy()
            .to_string();
        let open_folder_request = json!({
            "id": 6,
            "method": "cursor.workspace_open_folder",
            "params": { "path": expected_cwd.clone(), "name": "current" }
        })
        .to_string();
        let open_folder =
            dispatch_restricted_request(&open_folder_request, &|command| match command {
                ControlCommand::CreateWorkspace {
                    name,
                    cwd,
                    command,
                    reply,
                } => {
                    assert_eq!(name, Some("current".to_string()));
                    assert_eq!(cwd.expect("cwd"), expected_cwd);
                    assert_eq!(command, None);
                    let _ = reply.send(Ok(json!({ "workspace_ref": "workspace:1" })));
                }
                other => panic!("unexpected command: {other:?}"),
            });
        assert_eq!(open_folder.error, None);
    }

    #[test]
    fn unrestricted_surface_remains_compatible_with_agent_terminal_methods() {
        let response = dispatch_request(
            r#"{"id":1,"method":"surface.send_text","params":{"workspace_id":"codex","surface_id":"surface:4:tab","text":"hello"}}"#,
            &|command| match command {
                ControlCommand::SendText {
                    target,
                    surface_hint,
                    text,
                    reply,
                } => {
                    assert_eq!(target, WorkspaceTarget::Name("codex".to_string()));
                    assert_eq!(surface_hint, Some("surface:4:tab".to_string()));
                    assert_eq!(text, "hello");
                    let _ = reply.send(Ok(json!({ "sent": true })));
                }
                other => panic!("unexpected command: {other:?}"),
            },
        );

        assert_eq!(response.error, None);
    }
}
