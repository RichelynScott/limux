use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use limux_control::socket_path::RuntimeChannel;

pub const SESSION_VERSION: u32 = 1;
pub const PERSISTENCE_DIR_NAME: &str = "limux";
pub const LIMUX_SESSION_DIR_ENV: &str = "LIMUX_SESSION_DIR";
pub const SESSION_FILE_NAME: &str = "session.json";
pub const LEGACY_WORKSPACES_FILE_NAME: &str = "workspaces.json";
pub const DEFAULT_SIDEBAR_WIDTH: i32 = 220;
pub const MIN_SIDEBAR_WIDTH: i32 = 84;
pub const DEFAULT_SPLIT_RATIO: f64 = 0.5;
const MIN_SPLIT_RATIO: f64 = 0.08;
const MAX_SPLIT_RATIO: f64 = 0.92;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionLoadSource {
    Canonical,
    Legacy,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedSession {
    pub state: AppSessionState,
    pub source: SessionLoadSource,
    pub persisted_at: Option<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SidebarState {
    #[serde(default = "default_sidebar_visible")]
    pub visible: bool,
    #[serde(default = "default_sidebar_width")]
    pub width: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct AppSessionState {
    #[serde(default = "default_session_version")]
    pub version: u32,
    #[serde(default)]
    pub active_workspace_index: usize,
    #[serde(default = "default_top_bar_visible")]
    pub top_bar_visible: bool,
    #[serde(default)]
    pub sidebar: SidebarState,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceState>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct WorkspaceState {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub highlight: Option<WorkspaceHighlightColor>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub folder_path: Option<String>,
    pub layout: LayoutNodeState,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceHighlightColor {
    Orange,
    Red,
    Purple,
    Pink,
    Green,
    Yellow,
    Teal,
    Cyan,
}

impl WorkspaceHighlightColor {
    pub const ALL: [Self; 8] = [
        Self::Orange,
        Self::Red,
        Self::Purple,
        Self::Pink,
        Self::Green,
        Self::Yellow,
        Self::Teal,
        Self::Cyan,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Orange => "Orange",
            Self::Red => "Red",
            Self::Purple => "Purple",
            Self::Pink => "Pink",
            Self::Green => "Green",
            Self::Yellow => "Yellow",
            Self::Teal => "Teal",
            Self::Cyan => "Cyan",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Orange => "limux-sidebar-row-highlight-orange",
            Self::Red => "limux-sidebar-row-highlight-red",
            Self::Purple => "limux-sidebar-row-highlight-purple",
            Self::Pink => "limux-sidebar-row-highlight-pink",
            Self::Green => "limux-sidebar-row-highlight-green",
            Self::Yellow => "limux-sidebar-row-highlight-yellow",
            Self::Teal => "limux-sidebar-row-highlight-teal",
            Self::Cyan => "limux-sidebar-row-highlight-cyan",
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaneFlagColor {
    Orange,
    Red,
    Purple,
    Pink,
    Green,
    Yellow,
    Teal,
    Cyan,
}

impl PaneFlagColor {
    pub const ALL: [Self; 8] = [
        Self::Orange,
        Self::Red,
        Self::Purple,
        Self::Pink,
        Self::Green,
        Self::Yellow,
        Self::Teal,
        Self::Cyan,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Orange => "Orange",
            Self::Red => "Red",
            Self::Purple => "Purple",
            Self::Pink => "Pink",
            Self::Green => "Green",
            Self::Yellow => "Yellow",
            Self::Teal => "Teal",
            Self::Cyan => "Cyan",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Orange => "orange",
            Self::Red => "red",
            Self::Purple => "purple",
            Self::Pink => "pink",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Teal => "teal",
            Self::Cyan => "cyan",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Orange => "limux-pane-flag-orange",
            Self::Red => "limux-pane-flag-red",
            Self::Purple => "limux-pane-flag-purple",
            Self::Pink => "limux-pane-flag-pink",
            Self::Green => "limux-pane-flag-green",
            Self::Yellow => "limux-pane-flag-yellow",
            Self::Teal => "limux-pane-flag-teal",
            Self::Cyan => "limux-pane-flag-cyan",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|color| color.name() == name.trim().to_ascii_lowercase())
    }

    pub fn allowed_names() -> &'static str {
        "orange, red, purple, pink, green, yellow, teal, cyan"
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutNodeState {
    Pane(PaneState),
    Split(SplitState),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct SplitState {
    pub orientation: SplitOrientation,
    #[serde(default = "default_split_ratio")]
    pub ratio: f64,
    pub start: Box<LayoutNodeState>,
    pub end: Box<LayoutNodeState>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PaneState {
    #[serde(default)]
    pub pane_id: Option<u32>,
    #[serde(default)]
    pub active_tab_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<PaneFlagColor>,
    #[serde(default)]
    pub tabs: Vec<TabState>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TabState {
    pub id: String,
    #[serde(default)]
    pub custom_name: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(flatten)]
    pub content: TabContentState,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestorableAgentKind {
    Claude,
    Codex,
    OpenCode,
    Gemini,
    Hermes,
}

impl RestorableAgentKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Hermes => "hermes",
        }
    }

    pub fn resume_command(
        self,
        session_id: &str,
        launch_command: Option<&AgentLaunchCommandState>,
        cwd: Option<&str>,
    ) -> Option<String> {
        build_resume_command(self, session_id, launch_command, cwd)
    }

    fn fallback_executable(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Hermes => "hermes",
        }
    }

    fn store_name(self) -> &'static str {
        self.name()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct AgentLaunchCommandState {
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub captured_at: Option<f64>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct RestorableAgentState {
    pub kind: RestorableAgentKind,
    pub session_id: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub launch_command: Option<AgentLaunchCommandState>,
    #[serde(default)]
    pub restore_on_startup: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspension_reason: Option<AgentSuspensionReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspended_at: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook_updated_at: Option<f64>,
    #[serde(default)]
    pub hook_observation_initialized: bool,
}

impl RestorableAgentState {
    pub fn resume_command(&self) -> Option<String> {
        if !self.restore_on_startup {
            return None;
        }
        self.kind.resume_command(
            &self.session_id,
            self.launch_command.as_ref(),
            self.cwd.as_deref(),
        )
    }

    pub fn is_suspended(&self) -> bool {
        self.suspension_reason.is_some()
    }

    pub fn hcom_name(&self) -> Option<&str> {
        let launch = self.launch_command.as_ref()?;
        ["HCOM_NAME", "HCOM_INSTANCE_NAME"]
            .iter()
            .find_map(|key| launch.environment.get(*key))
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    pub fn suspension_tooltip(&self) -> Option<String> {
        self.suspension_reason.map(|reason| match reason {
            AgentSuspensionReason::UncleanRestore => {
                "Agent suspended after an unclean Limux shutdown".to_string()
            }
            AgentSuspensionReason::PressureGating => {
                "Agent suspended by the runtime pressure gate".to_string()
            }
            AgentSuspensionReason::Cancelled => "Agent resume was cancelled".to_string(),
            AgentSuspensionReason::UserChoice => "Agent resume is paused".to_string(),
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentSuspensionReason {
    UncleanRestore,
    PressureGating,
    Cancelled,
    UserChoice,
}

impl AgentSuspensionReason {
    pub fn name(self) -> &'static str {
        match self {
            Self::UncleanRestore => "unclean_restore",
            Self::PressureGating => "pressure_gating",
            Self::Cancelled => "cancelled",
            Self::UserChoice => "user_choice",
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "tab_kind", rename_all = "snake_case")]
pub enum TabContentState {
    Terminal {
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        agent: Option<RestorableAgentState>,
    },
    Browser {
        #[serde(default)]
        uri: Option<String>,
    },
    Keybinds {},
    Settings {},
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LegacySavedWorkspace {
    pub name: String,
    pub favorite: bool,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub folder_path: Option<String>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            visible: default_sidebar_visible(),
            width: default_sidebar_width(),
        }
    }
}

impl Default for AppSessionState {
    fn default() -> Self {
        Self {
            version: default_session_version(),
            active_workspace_index: 0,
            top_bar_visible: default_top_bar_visible(),
            sidebar: SidebarState::default(),
            workspaces: Vec::new(),
        }
    }
}

impl PaneState {
    pub fn fallback(working_directory: Option<&str>) -> Self {
        let tab = TabState::terminal(default_tab_id("terminal"), working_directory);
        Self {
            pane_id: None,
            active_tab_id: Some(tab.id.clone()),
            flag_color: None,
            tabs: vec![tab],
        }
    }

    pub fn browser_only(uri: Option<&str>) -> Self {
        let tab = TabState::browser(default_tab_id("browser"), uri);
        Self {
            pane_id: None,
            active_tab_id: Some(tab.id.clone()),
            flag_color: None,
            tabs: vec![tab],
        }
    }
}

impl TabState {
    pub fn terminal(id: impl Into<String>, cwd: Option<&str>) -> Self {
        Self {
            id: id.into(),
            custom_name: None,
            pinned: false,
            content: TabContentState::Terminal {
                cwd: cwd.map(|value| value.to_string()),
                agent: None,
            },
        }
    }

    pub fn browser(id: impl Into<String>, uri: Option<&str>) -> Self {
        Self {
            id: id.into(),
            custom_name: None,
            pinned: false,
            content: TabContentState::Browser {
                uri: uri.map(|value| value.to_string()),
            },
        }
    }
}

pub fn persistence_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(LIMUX_SESSION_DIR_ENV).filter(|value| !value.is_empty()) {
        return PathBuf::from(dir);
    }
    if let Some(channel) = RuntimeChannel::from_env() {
        return channel_persistence_dir(&channel);
    }

    base_persistence_dir()
}

fn base_persistence_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join(PERSISTENCE_DIR_NAME);
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share").join(PERSISTENCE_DIR_NAME)
}

pub fn channel_persistence_dir(channel: &RuntimeChannel) -> PathBuf {
    let base = base_persistence_dir();
    match channel {
        RuntimeChannel::Stable => base.join("stable").join("session"),
        RuntimeChannel::Preview(id) => base.join("preview").join(id).join("session"),
    }
}

pub fn canonical_session_path_in(dir: &Path) -> PathBuf {
    dir.join(SESSION_FILE_NAME)
}

pub fn legacy_workspaces_path_in(dir: &Path) -> PathBuf {
    dir.join(LEGACY_WORKSPACES_FILE_NAME)
}

pub fn load_session() -> LoadedSession {
    load_session_from_dir(&persistence_dir())
}

pub fn load_session_from_dir(dir: &Path) -> LoadedSession {
    let canonical_path = canonical_session_path_in(dir);
    if canonical_path.exists() {
        let state = fs::read_to_string(&canonical_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<AppSessionState>(&raw).ok())
            .map(normalize_session)
            .unwrap_or_default();
        return LoadedSession {
            state,
            source: SessionLoadSource::Canonical,
            persisted_at: file_modified_seconds(&canonical_path),
        };
    }

    let legacy_path = legacy_workspaces_path_in(dir);
    if legacy_path.exists() {
        let state = fs::read_to_string(&legacy_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<LegacySavedWorkspace>>(&raw).ok())
            .map(AppSessionState::from_legacy)
            .unwrap_or_default();
        return LoadedSession {
            state,
            source: SessionLoadSource::Legacy,
            persisted_at: file_modified_seconds(&legacy_path),
        };
    }

    LoadedSession {
        state: AppSessionState::default(),
        source: SessionLoadSource::Empty,
        persisted_at: None,
    }
}

fn file_modified_seconds(path: &Path) -> Option<f64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs_f64())
}

pub fn save_session_atomic(state: &AppSessionState) -> io::Result<PathBuf> {
    save_session_atomic_in(&persistence_dir(), state)
}

pub fn save_session_atomic_in(dir: &Path, state: &AppSessionState) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let path = canonical_session_path_in(dir);
    let normalized = normalize_session(state.clone());
    let json = serde_json::to_vec_pretty(&normalized)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    crate::durable_atomic::write_bytes_atomic_durable(&path, &json)?;
    Ok(path)
}

pub fn clamp_split_ratio(ratio: f64) -> f64 {
    if !ratio.is_finite() {
        return DEFAULT_SPLIT_RATIO;
    }
    ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO)
}

pub fn clamp_split_ratio_for_size(ratio: f64, total_size: i32, min_child_size: i32) -> f64 {
    let ratio = clamp_split_ratio(ratio);
    if total_size <= 0 || min_child_size <= 0 {
        return ratio;
    }
    if total_size <= min_child_size.saturating_mul(2) {
        return DEFAULT_SPLIT_RATIO;
    }

    let min_ratio =
        (min_child_size as f64 / total_size as f64).clamp(MIN_SPLIT_RATIO, DEFAULT_SPLIT_RATIO);
    ratio.clamp(min_ratio, 1.0 - min_ratio)
}

#[cfg(test)]
fn split_ratio_from_position(position: i32, total_size: i32) -> f64 {
    if total_size <= 0 {
        return DEFAULT_SPLIT_RATIO;
    }
    clamp_split_ratio(position as f64 / total_size as f64)
}

pub fn split_ratio_from_position_with_min(
    position: i32,
    total_size: i32,
    min_child_size: i32,
) -> f64 {
    if total_size <= 0 {
        return DEFAULT_SPLIT_RATIO;
    }
    clamp_split_ratio_for_size(
        position as f64 / total_size as f64,
        total_size,
        min_child_size,
    )
}

#[cfg(test)]
fn snapshot_split_ratio(position: i32, total_size: i32, stored_ratio: Option<f64>) -> f64 {
    if total_size <= 0 {
        return stored_ratio
            .map(clamp_split_ratio)
            .unwrap_or(DEFAULT_SPLIT_RATIO);
    }
    split_ratio_from_position(position, total_size)
}

pub fn snapshot_split_ratio_with_min(
    position: i32,
    total_size: i32,
    stored_ratio: Option<f64>,
    min_child_size: i32,
) -> f64 {
    if total_size <= 0 {
        return stored_ratio
            .map(clamp_split_ratio)
            .unwrap_or(DEFAULT_SPLIT_RATIO);
    }
    if min_child_size > 0 && total_size <= min_child_size.saturating_mul(2) {
        return stored_ratio
            .map(clamp_split_ratio)
            .unwrap_or(DEFAULT_SPLIT_RATIO);
    }
    split_ratio_from_position_with_min(position, total_size, min_child_size)
}

#[cfg(test)]
fn split_position_from_ratio(ratio: f64, total_size: i32) -> i32 {
    if total_size <= 0 {
        return 0;
    }
    (clamp_split_ratio(ratio) * total_size as f64).round() as i32
}

pub fn split_position_from_ratio_with_min(ratio: f64, total_size: i32, min_child_size: i32) -> i32 {
    if total_size <= 0 {
        return 0;
    }
    (clamp_split_ratio_for_size(ratio, total_size, min_child_size) * total_size as f64).round()
        as i32
}

pub fn normalize_session(mut state: AppSessionState) -> AppSessionState {
    state.version = SESSION_VERSION;
    state.sidebar.width = state.sidebar.width.max(MIN_SIDEBAR_WIDTH);
    if state.workspaces.is_empty() {
        state.active_workspace_index = 0;
    } else if state.active_workspace_index >= state.workspaces.len() {
        state.active_workspace_index = state.workspaces.len() - 1;
    }
    for workspace in &mut state.workspaces {
        normalize_layout(
            &mut workspace.layout,
            workspace
                .folder_path
                .as_deref()
                .or(workspace.cwd.as_deref()),
        );
    }
    state
}

pub fn normalize_layout(layout: &mut LayoutNodeState, working_directory: Option<&str>) {
    match layout {
        LayoutNodeState::Pane(pane) => {
            if pane.tabs.is_empty() {
                *pane = PaneState::fallback(working_directory);
                return;
            }
            normalize_pane_tab_ids(pane);
            let mut active_exists = false;
            for tab in &pane.tabs {
                if pane.active_tab_id.as_deref() == Some(tab.id.as_str()) {
                    active_exists = true;
                    break;
                }
            }
            if !active_exists {
                pane.active_tab_id = pane.tabs.first().map(|tab| tab.id.clone());
            }
        }
        LayoutNodeState::Split(split) => {
            split.ratio = clamp_split_ratio(split.ratio);
            normalize_layout(&mut split.start, working_directory);
            normalize_layout(&mut split.end, working_directory);
        }
    }
}

fn normalize_pane_tab_ids(pane: &mut PaneState) {
    let mut used = HashSet::new();
    for tab in &mut pane.tabs {
        let base = if tab.id.trim().is_empty() {
            "tab".to_string()
        } else {
            tab.id.clone()
        };
        if used.insert(base.clone()) {
            tab.id = base;
            continue;
        }

        for suffix in 1.. {
            let candidate = format!("{base}-{suffix}");
            if used.insert(candidate.clone()) {
                tab.id = candidate;
                break;
            }
        }
    }
}

impl AppSessionState {
    pub fn from_legacy(workspaces: Vec<LegacySavedWorkspace>) -> Self {
        let workspaces = workspaces
            .into_iter()
            .map(|workspace| {
                let working_directory = workspace
                    .folder_path
                    .as_deref()
                    .or(workspace.cwd.as_deref());
                let tab = TabState::terminal(default_tab_id("legacy-terminal"), working_directory);
                WorkspaceState {
                    id: None,
                    name: workspace.name,
                    favorite: workspace.favorite,
                    highlight: None,
                    cwd: workspace.cwd,
                    folder_path: workspace.folder_path,
                    // Legacy files only knew "workspace exists"; rehydrate a fresh terminal at the
                    // last known directory instead of pretending process state can be restored.
                    layout: LayoutNodeState::Pane(PaneState {
                        active_tab_id: Some(tab.id.clone()),
                        pane_id: None,
                        flag_color: None,
                        tabs: vec![tab],
                    }),
                }
            })
            .collect();
        normalize_session(Self {
            workspaces,
            ..Self::default()
        })
    }
}

#[derive(serde::Deserialize)]
struct HookSessionRecord {
    session_id: String,
    workspace_id: String,
    surface_id: String,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    launch_command: Option<AgentLaunchCommandState>,
    updated_at: f64,
}

#[derive(serde::Deserialize)]
struct HookSessionFile {
    version: u32,
    sessions: BTreeMap<String, HookSessionRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct RestorableAgentIndex {
    by_surface: HashMap<(String, String), (RestorableAgentState, f64)>,
    by_any_workspace_surface: HashMap<String, Option<(RestorableAgentState, f64)>>,
    by_tab_id: HashMap<String, Option<(RestorableAgentState, f64)>>,
    loaded_kinds: HashSet<RestorableAgentKind>,
}

impl RestorableAgentIndex {
    pub fn load() -> Self {
        Self::load_from_dir(&agent_hook_state_dir())
    }

    pub fn load_from_dir(dir: &Path) -> Self {
        let mut index = Self::default();
        for (kind, file_name) in [
            (RestorableAgentKind::Claude, "claude-hook-sessions.json"),
            (RestorableAgentKind::Codex, "codex-hook-sessions.json"),
            (RestorableAgentKind::OpenCode, "opencode-hook-sessions.json"),
            (RestorableAgentKind::Gemini, "gemini-hook-sessions.json"),
            (RestorableAgentKind::Hermes, "hermes-hook-sessions.json"),
        ] {
            let path = dir.join(file_name);
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(file) = serde_json::from_str::<HookSessionFile>(&raw) else {
                continue;
            };
            if file.version == 0 {
                continue;
            }
            index.loaded_kinds.insert(kind);
            for record in file.sessions.values() {
                let Some(session_id) = normalized_str(&record.session_id) else {
                    continue;
                };
                let Some(workspace_id) = normalized_str(&record.workspace_id) else {
                    continue;
                };
                let Some(surface_id) = normalized_str(&record.surface_id) else {
                    continue;
                };
                let tab_id = surface_id
                    .rsplit_once(':')
                    .map(|(_, tab_id)| tab_id.to_string());
                let key = (workspace_id, surface_id);
                if index
                    .by_surface
                    .get(&key)
                    .is_some_and(|(_, updated_at)| *updated_at > record.updated_at)
                {
                    continue;
                }
                index.by_surface.insert(
                    key.clone(),
                    (
                        RestorableAgentState {
                            kind,
                            session_id: session_id.clone(),
                            cwd: record.cwd.clone(),
                            launch_command: record.launch_command.clone(),
                            restore_on_startup: true,
                            suspension_reason: None,
                            suspended_at: None,
                            hook_updated_at: Some(record.updated_at),
                            hook_observation_initialized: true,
                        },
                        record.updated_at,
                    ),
                );
                match index.by_any_workspace_surface.entry(key.1) {
                    Entry::Vacant(entry) => {
                        entry.insert(Some((
                            RestorableAgentState {
                                kind,
                                session_id: session_id.clone(),
                                cwd: record.cwd.clone(),
                                launch_command: record.launch_command.clone(),
                                restore_on_startup: true,
                                suspension_reason: None,
                                suspended_at: None,
                                hook_updated_at: Some(record.updated_at),
                                hook_observation_initialized: true,
                            },
                            record.updated_at,
                        )));
                    }
                    Entry::Occupied(mut entry) => {
                        entry.insert(None);
                    }
                }
                if let Some(tab_id) = tab_id {
                    match index.by_tab_id.entry(tab_id) {
                        Entry::Vacant(entry) => {
                            entry.insert(Some((
                                RestorableAgentState {
                                    kind,
                                    session_id: session_id.clone(),
                                    cwd: record.cwd.clone(),
                                    launch_command: record.launch_command.clone(),
                                    restore_on_startup: true,
                                    suspension_reason: None,
                                    suspended_at: None,
                                    hook_updated_at: Some(record.updated_at),
                                    hook_observation_initialized: true,
                                },
                                record.updated_at,
                            )));
                        }
                        Entry::Occupied(mut entry) => {
                            entry.insert(None);
                        }
                    }
                }
            }
        }
        index
    }

    #[cfg(test)]
    fn agent_for_surface(
        &self,
        workspace_id: &str,
        pane_id: Option<u32>,
        tab_id: &str,
    ) -> Option<RestorableAgentState> {
        self.agent_for_surface_entry(workspace_id, pane_id, tab_id)
            .map(|(agent, _)| agent)
    }

    fn agent_for_surface_entry(
        &self,
        workspace_id: &str,
        pane_id: Option<u32>,
        tab_id: &str,
    ) -> Option<(RestorableAgentState, f64)> {
        let surface_id = pane_id.map(|pane_id| format!("{pane_id}:{tab_id}"));
        surface_id
            .as_ref()
            .and_then(|surface_id| {
                self.by_surface
                    .get(&(workspace_id.to_string(), surface_id.clone()))
                    .or_else(|| {
                        self.by_any_workspace_surface
                            .get(surface_id)
                            .and_then(|candidate| candidate.as_ref())
                    })
            })
            .or_else(|| {
                self.by_tab_id
                    .get(tab_id)
                    .and_then(|candidate| candidate.as_ref())
            })
            .map(|(agent, updated_at)| (agent.clone(), *updated_at))
    }

    fn has_loaded_kind(&self, kind: RestorableAgentKind) -> bool {
        self.loaded_kinds.contains(&kind)
    }
}

pub fn attach_restorable_agents_to_layout(
    layout: &mut LayoutNodeState,
    workspace_id: &str,
    index: &RestorableAgentIndex,
) {
    match layout {
        LayoutNodeState::Pane(pane) => {
            for tab in &mut pane.tabs {
                if let TabContentState::Terminal { agent, .. } = &mut tab.content {
                    let restored_agent =
                        index.agent_for_surface_entry(workspace_id, pane.pane_id, &tab.id);
                    if let Some(existing) = agent.as_ref().filter(|agent| !agent.restore_on_startup)
                    {
                        if existing.suspension_reason != Some(AgentSuspensionReason::UncleanRestore)
                        {
                            continue;
                        }
                        let is_fresh =
                            restored_agent
                                .as_ref()
                                .is_some_and(|(candidate, updated_at)| {
                                    if existing.hook_observation_initialized {
                                        candidate.kind == existing.kind
                                            && candidate.session_id == existing.session_id
                                            && existing.hook_updated_at.is_none_or(|observed_at| {
                                                *updated_at != observed_at
                                            })
                                    } else {
                                        existing
                                            .suspended_at
                                            .is_some_and(|suspended_at| *updated_at > suspended_at)
                                    }
                                });
                        if !is_fresh {
                            continue;
                        }
                    }
                    if let Some((restored_agent, _)) = restored_agent {
                        *agent = Some(restored_agent);
                    } else if agent
                        .as_ref()
                        .is_some_and(|agent| index.has_loaded_kind(agent.kind))
                    {
                        *agent = None;
                    }
                }
            }
        }
        LayoutNodeState::Split(split) => {
            attach_restorable_agents_to_layout(&mut split.start, workspace_id, index);
            attach_restorable_agents_to_layout(&mut split.end, workspace_id, index);
        }
    }
}

pub fn suspend_agents_for_unclean_restore(layout: &mut LayoutNodeState) -> usize {
    let suspended_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(f64::MAX);
    suspend_agents_for_unclean_restore_at(layout, suspended_at)
}

fn suspend_agents_for_unclean_restore_at(layout: &mut LayoutNodeState, suspended_at: f64) -> usize {
    match layout {
        LayoutNodeState::Pane(pane) => pane
            .tabs
            .iter_mut()
            .filter_map(|tab| match &mut tab.content {
                TabContentState::Terminal {
                    agent: Some(agent), ..
                } => Some(agent),
                _ => None,
            })
            .filter(|agent| {
                agent.restore_on_startup
                    || agent.suspension_reason == Some(AgentSuspensionReason::UncleanRestore)
            })
            .map(|agent| {
                agent.restore_on_startup = false;
                agent.suspension_reason = Some(AgentSuspensionReason::UncleanRestore);
                agent.suspended_at = Some(suspended_at);
                1usize
            })
            .sum(),
        LayoutNodeState::Split(split) => {
            suspend_agents_for_unclean_restore_at(&mut split.start, suspended_at)
                + suspend_agents_for_unclean_restore_at(&mut split.end, suspended_at)
        }
    }
}

pub fn seed_legacy_unclean_suspension_baseline(
    layout: &mut LayoutNodeState,
    workspace_id: &str,
    index: &RestorableAgentIndex,
    persisted_at: Option<f64>,
) -> usize {
    match layout {
        LayoutNodeState::Pane(pane) => {
            let mut seeded = 0;
            for tab in &mut pane.tabs {
                let TabContentState::Terminal {
                    agent: Some(agent), ..
                } = &mut tab.content
                else {
                    continue;
                };
                if agent.restore_on_startup
                    || agent.suspension_reason != Some(AgentSuspensionReason::UncleanRestore)
                {
                    continue;
                }

                let mut changed = false;
                if !agent.hook_observation_initialized && agent.hook_updated_at.is_none() {
                    if let Some((observed_agent, observed_at)) =
                        index.agent_for_surface_entry(workspace_id, pane.pane_id, &tab.id)
                    {
                        if observed_agent.kind == agent.kind
                            && observed_agent.session_id == agent.session_id
                        {
                            agent.hook_updated_at = Some(observed_at);
                            changed = true;
                        }
                    }
                }
                if !agent.hook_observation_initialized {
                    agent.hook_observation_initialized = true;
                    changed = true;
                }
                if agent.suspended_at.is_none() {
                    if let Some(persisted_at) = persisted_at {
                        agent.suspended_at = Some(persisted_at);
                        changed = true;
                    }
                }
                seeded += usize::from(changed);
            }
            seeded
        }
        LayoutNodeState::Split(split) => {
            seed_legacy_unclean_suspension_baseline(
                &mut split.start,
                workspace_id,
                index,
                persisted_at,
            ) + seed_legacy_unclean_suspension_baseline(
                &mut split.end,
                workspace_id,
                index,
                persisted_at,
            )
        }
    }
}

fn agent_hook_state_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("LIMUX_AGENT_HOOK_STATE_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = dirs::state_dir() {
        return dir.join(PERSISTENCE_DIR_NAME);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/state")
        .join(PERSISTENCE_DIR_NAME)
}

fn default_session_version() -> u32 {
    SESSION_VERSION
}

fn default_sidebar_visible() -> bool {
    true
}

fn default_top_bar_visible() -> bool {
    true
}

fn default_sidebar_width() -> i32 {
    DEFAULT_SIDEBAR_WIDTH
}

fn default_split_ratio() -> f64 {
    DEFAULT_SPLIT_RATIO
}

fn default_tab_id(prefix: &str) -> String {
    format!("{prefix}-0")
}

fn build_resume_command(
    kind: RestorableAgentKind,
    session_id: &str,
    launch: Option<&AgentLaunchCommandState>,
    cwd: Option<&str>,
) -> Option<String> {
    let session_id = normalized_str(session_id)?;
    if let Some(command) = build_hcom_resume_command(kind, &session_id, launch, cwd) {
        return Some(command);
    }
    let fallback = kind.fallback_executable().to_string();
    let args = launch
        .map(|launch| launch.arguments.clone())
        .filter(|args| !args.is_empty())
        .unwrap_or_else(|| vec![fallback.clone()]);
    let sanitized = sanitize_launch_arguments(kind, &args);
    let executable = launch
        .and_then(|launch| normalized_str(&launch.executable))
        .or_else(|| sanitized.first().cloned())
        .unwrap_or(fallback);
    let preserved_tail = sanitized
        .get(1..)
        .map(|tail| tail.to_vec())
        .unwrap_or_default();

    let mut parts = vec![executable];
    match kind {
        RestorableAgentKind::Codex => {
            parts.push("resume".to_string());
            parts.extend(preserved_tail);
            parts.push(session_id.clone());
        }
        RestorableAgentKind::OpenCode => {
            parts.push("--session".to_string());
            parts.push(session_id.clone());
            parts.extend(preserved_tail);
        }
        RestorableAgentKind::Claude | RestorableAgentKind::Gemini | RestorableAgentKind::Hermes => {
            parts.push("--resume".to_string());
            parts.push(session_id.clone());
            parts.extend(preserved_tail);
        }
    }

    let command = parts
        .iter()
        .map(|part| shell_single_quote(part))
        .collect::<Vec<_>>()
        .join(" ");
    let cwd = cwd.and_then(normalized_str).or_else(|| {
        launch
            .and_then(|launch| launch.cwd.as_deref())
            .and_then(normalized_str)
    });
    let run_command = match cwd {
        Some(cwd) => format!("cd {} && {command}", shell_single_quote(&cwd)),
        None => command,
    };
    Some(wrap_restored_agent_command(kind, &session_id, &run_command))
}

fn build_hcom_resume_command(
    kind: RestorableAgentKind,
    session_id: &str,
    launch: Option<&AgentLaunchCommandState>,
    cwd: Option<&str>,
) -> Option<String> {
    let launch = launch?;
    if !is_hcom_managed_launch(launch) {
        return None;
    }
    let target = hcom_resume_name(launch).unwrap_or_else(|| session_id.to_string());
    let executable = hcom_executable(launch);
    let command = [
        executable.as_str(),
        "r",
        target.as_str(),
        "--run-here",
        "--go",
    ]
    .iter()
    .map(|part| shell_single_quote(part))
    .collect::<Vec<_>>()
    .join(" ");
    let cwd = cwd
        .and_then(normalized_str)
        .or_else(|| launch.cwd.as_deref().and_then(normalized_str));
    let run_command = match cwd {
        Some(cwd) => format!("cd {} && {command}", shell_single_quote(&cwd)),
        None => command,
    };
    Some(wrap_restored_agent_command(kind, session_id, &run_command))
}

fn is_hcom_managed_launch(launch: &AgentLaunchCommandState) -> bool {
    launch
        .environment
        .get("HCOM_PROCESS_ID")
        .is_some_and(|value| normalized_str(value).is_some())
        || launch
            .environment
            .get("HCOM_LAUNCHED")
            .is_some_and(|value| value.trim() == "1")
        || is_hcom_executable(&launch.executable)
        || launch
            .arguments
            .first()
            .is_some_and(|argument| is_hcom_executable(argument))
}

fn hcom_resume_name(launch: &AgentLaunchCommandState) -> Option<String> {
    ["HCOM_NAME", "HCOM_INSTANCE_NAME"].iter().find_map(|name| {
        launch
            .environment
            .get(*name)
            .and_then(|value| normalized_str(value))
    })
}

fn hcom_executable(launch: &AgentLaunchCommandState) -> String {
    launch
        .arguments
        .first()
        .filter(|value| is_hcom_executable(value))
        .or_else(|| is_hcom_executable(&launch.executable).then_some(&launch.executable))
        .cloned()
        .unwrap_or_else(|| "hcom".to_string())
}

fn is_hcom_executable(value: &str) -> bool {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "hcom")
}

fn wrap_restored_agent_command(
    kind: RestorableAgentKind,
    session_id: &str,
    run_command: &str,
) -> String {
    let payload = format!(
        "{{\"session_id\":{},\"hook_event_name\":\"Cleanup\"}}",
        serde_json::to_string(session_id).unwrap_or_else(|_| "\"\"".to_string())
    );
    let cleanup = format!(
        "printf %s {} | {} --json hooks {} cleanup >/dev/null 2>&1 || true",
        shell_single_quote(&payload),
        shell_single_quote(&limux_cli_executable()),
        kind.store_name()
    );
    format!("{run_command}; limux_agent_status=$?; {cleanup}; exec \"${{SHELL:-/bin/sh}}\" -l")
}

fn limux_cli_executable() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            limux_cli_candidates(&path)
                .into_iter()
                .find(|candidate| candidate.exists())
        })
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "limux".to_string())
}

fn limux_cli_candidates(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(dir) = exe.parent() {
        candidates.push(dir.join("limux-cli"));

        if let Some(libexec_dir) = dir.parent() {
            if let Some(prefix) = libexec_dir.parent() {
                candidates.push(prefix.join("bin/limux"));
            }
        }
    }

    candidates.push(PathBuf::from("limux"));
    candidates
}

fn sanitize_launch_arguments(kind: RestorableAgentKind, arguments: &[String]) -> Vec<String> {
    if arguments.is_empty() {
        return vec![kind.fallback_executable().to_string()];
    }
    let mut result = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let arg = &arguments[index];
        if index == 0 {
            result.push(arg.clone());
            index += 1;
            continue;
        }
        if is_resume_selector(kind, arg) || option_takes_secret_value(arg) {
            index += 1;
            if index < arguments.len() && !arguments[index].starts_with('-') {
                index += 1;
            }
            continue;
        }
        if option_is_secret_assignment(arg) {
            index += 1;
            continue;
        }
        if option_takes_safe_value(arg) {
            result.push(arg.clone());
            if index + 1 < arguments.len() {
                result.push(arguments[index + 1].clone());
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if option_is_safe_flag_or_assignment(arg) {
            result.push(arg.clone());
            index += 1;
            continue;
        }
        if arg.starts_with('-') {
            index += 1;
            continue;
        }
        break;
    }
    result
}

fn is_resume_selector(kind: RestorableAgentKind, arg: &str) -> bool {
    match kind {
        RestorableAgentKind::Codex => {
            arg == "resume" || arg == "--resume" || arg.starts_with("--resume=")
        }
        RestorableAgentKind::OpenCode => arg == "--session" || arg.starts_with("--session="),
        RestorableAgentKind::Claude | RestorableAgentKind::Gemini | RestorableAgentKind::Hermes => {
            arg == "--resume" || arg.starts_with("--resume=") || arg == "--continue"
        }
    }
}

fn option_takes_secret_value(arg: &str) -> bool {
    matches!(
        arg,
        "--api-key" | "--apikey" | "--token" | "--auth-token" | "--password"
    )
}

fn option_is_secret_assignment(arg: &str) -> bool {
    let lower = arg.to_ascii_lowercase();
    lower.starts_with("--api-key=")
        || lower.starts_with("--apikey=")
        || lower.starts_with("--token=")
        || lower.starts_with("--auth-token=")
        || lower.starts_with("--password=")
}

fn option_takes_safe_value(arg: &str) -> bool {
    matches!(
        arg,
        "--model"
            | "-m"
            | "--config"
            | "-c"
            | "--profile"
            | "--sandbox"
            | "--approval-policy"
            | "--cwd"
            | "--cd"
            | "--working-directory"
    )
}

fn option_is_safe_flag_or_assignment(arg: &str) -> bool {
    if matches!(arg, "--search" | "--no-search") {
        return true;
    }
    let Some((name, _)) = arg.split_once('=') else {
        return false;
    };
    option_takes_safe_value(name)
}

fn normalized_str(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let old = std::env::var_os(key);
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => unsafe { std::env::set_var(self.key, value) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn persistence_dir_uses_xdg_data_home_directly() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _session_dir = EnvGuard::set(LIMUX_SESSION_DIR_ENV, None);
        let _channel = EnvGuard::set(limux_control::socket_path::LIMUX_CHANNEL_ENV, None);
        let _xdg = EnvGuard::set("XDG_DATA_HOME", Some("/tmp/limux-xdg-data"));
        let _home = EnvGuard::set("HOME", Some("/tmp/limux-home"));

        assert_eq!(
            persistence_dir(),
            PathBuf::from("/tmp/limux-xdg-data").join(PERSISTENCE_DIR_NAME)
        );
    }

    #[test]
    fn persistence_dir_falls_back_to_home_local_share_when_data_dir_missing() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _session_dir = EnvGuard::set(LIMUX_SESSION_DIR_ENV, None);
        let _channel = EnvGuard::set(limux_control::socket_path::LIMUX_CHANNEL_ENV, None);
        let _xdg = EnvGuard::set("XDG_DATA_HOME", None);
        let _home = EnvGuard::set("HOME", Some("/tmp/limux-home"));

        assert_eq!(
            persistence_dir(),
            PathBuf::from("/tmp/limux-home/.local/share").join(PERSISTENCE_DIR_NAME)
        );
    }

    #[test]
    fn persistence_dir_uses_session_dir_override() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _session_dir = EnvGuard::set(LIMUX_SESSION_DIR_ENV, Some("/tmp/limux-session"));
        let _channel = EnvGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Some("preview:test"),
        );
        let _xdg = EnvGuard::set("XDG_DATA_HOME", Some("/tmp/limux-xdg-data"));

        assert_eq!(persistence_dir(), PathBuf::from("/tmp/limux-session"));
    }

    #[test]
    fn persistence_dir_ignores_empty_session_dir_override() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _session_dir = EnvGuard::set(LIMUX_SESSION_DIR_ENV, Some(""));
        let _channel = EnvGuard::set(limux_control::socket_path::LIMUX_CHANNEL_ENV, None);
        let _xdg = EnvGuard::set("XDG_DATA_HOME", Some("/tmp/limux-xdg-data"));

        assert_eq!(
            persistence_dir(),
            PathBuf::from("/tmp/limux-xdg-data").join(PERSISTENCE_DIR_NAME)
        );
    }

    #[test]
    fn persistence_dir_uses_channel_namespace_when_set() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _session_dir = EnvGuard::set(LIMUX_SESSION_DIR_ENV, None);
        let _channel = EnvGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Some("preview:branch"),
        );
        let _xdg = EnvGuard::set("XDG_DATA_HOME", Some("/tmp/limux-xdg-data"));

        assert_eq!(
            persistence_dir(),
            PathBuf::from("/tmp/limux-xdg-data/limux/preview/branch/session")
        );
    }

    #[test]
    fn limux_cli_candidates_cover_installed_and_dev_layouts() {
        let installed = Path::new("/usr/libexec/limux/limux-host");
        let candidates = limux_cli_candidates(installed);
        assert!(candidates.contains(&PathBuf::from("/usr/bin/limux")));

        let dev = Path::new("/repo/target/debug/limux");
        let candidates = limux_cli_candidates(dev);
        assert!(candidates.contains(&PathBuf::from("/repo/target/debug/limux-cli")));
        assert!(candidates.contains(&PathBuf::from("limux")));
    }

    #[test]
    fn load_prefers_canonical_session_over_legacy() {
        let dir = tempdir().expect("tempdir");
        let canonical_path = canonical_session_path_in(dir.path());
        let legacy_path = legacy_workspaces_path_in(dir.path());

        let canonical = AppSessionState {
            workspaces: vec![WorkspaceState {
                id: Some("11111111-1111-4111-8111-111111111111".to_string()),
                name: "canonical".to_string(),
                favorite: true,
                highlight: None,
                cwd: Some("/canonical".to_string()),
                folder_path: Some("/canonical".to_string()),
                layout: LayoutNodeState::Pane(PaneState::fallback(Some("/canonical"))),
            }],
            ..AppSessionState::default()
        };
        fs::write(
            &canonical_path,
            serde_json::to_string_pretty(&canonical).expect("canonical json"),
        )
        .expect("write canonical");
        fs::write(
            &legacy_path,
            serde_json::to_string_pretty(&vec![LegacySavedWorkspace {
                name: "legacy".to_string(),
                favorite: false,
                cwd: Some("/legacy".to_string()),
                folder_path: None,
            }])
            .expect("legacy json"),
        )
        .expect("write legacy");

        let loaded = load_session_from_dir(dir.path());
        assert_eq!(loaded.source, SessionLoadSource::Canonical);
        assert!(loaded.persisted_at.is_some_and(|value| value > 0.0));
        assert_eq!(loaded.state.workspaces[0].name, "canonical");
    }

    #[test]
    fn load_migrates_legacy_workspaces_when_canonical_missing() {
        let dir = tempdir().expect("tempdir");
        let legacy_path = legacy_workspaces_path_in(dir.path());
        fs::write(
            &legacy_path,
            serde_json::to_string_pretty(&vec![LegacySavedWorkspace {
                name: "legacy".to_string(),
                favorite: true,
                cwd: Some("/tmp/project".to_string()),
                folder_path: None,
            }])
            .expect("legacy json"),
        )
        .expect("write legacy");

        let loaded = load_session_from_dir(dir.path());
        assert_eq!(loaded.source, SessionLoadSource::Legacy);
        assert_eq!(loaded.state.workspaces.len(), 1);
        assert_eq!(loaded.state.workspaces[0].name, "legacy");
        let LayoutNodeState::Pane(pane) = &loaded.state.workspaces[0].layout else {
            panic!("legacy migration should create a pane layout");
        };
        assert_eq!(pane.tabs.len(), 1);
        match &pane.tabs[0].content {
            TabContentState::Terminal { cwd, .. } => {
                assert_eq!(cwd.as_deref(), Some("/tmp/project"));
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn load_returns_empty_state_for_corrupt_canonical_file() {
        let dir = tempdir().expect("tempdir");
        let canonical_path = canonical_session_path_in(dir.path());
        fs::write(&canonical_path, "{not-json").expect("write corrupt canonical");

        let loaded = load_session_from_dir(dir.path());
        assert_eq!(loaded.source, SessionLoadSource::Canonical);
        assert_eq!(loaded.state, AppSessionState::default());
    }

    #[test]
    fn load_defaults_top_bar_visible_when_omitted_from_session_json() {
        let dir = tempdir().expect("tempdir");
        let canonical_path = canonical_session_path_in(dir.path());
        fs::write(
            &canonical_path,
            r#"{
                "version": 1,
                "active_workspace_index": 0,
                "sidebar": {
                    "visible": true,
                    "width": 220
                },
                "workspaces": []
            }"#,
        )
        .expect("write canonical");

        let loaded = load_session_from_dir(dir.path());
        assert!(loaded.state.top_bar_visible);
    }

    #[test]
    fn save_session_atomic_writes_canonical_file() {
        let dir = tempdir().expect("tempdir");
        let state = AppSessionState {
            workspaces: vec![WorkspaceState {
                id: Some("22222222-2222-4222-8222-222222222222".to_string()),
                name: "workspace".to_string(),
                favorite: false,
                highlight: None,
                cwd: Some("/tmp".to_string()),
                folder_path: Some("/tmp".to_string()),
                layout: LayoutNodeState::Pane(PaneState::fallback(Some("/tmp"))),
            }],
            ..AppSessionState::default()
        };

        let path = save_session_atomic_in(dir.path(), &state).expect("save canonical session");
        assert_eq!(path, canonical_session_path_in(dir.path()));
        let raw = fs::read_to_string(path).expect("read canonical session");
        let decoded: AppSessionState =
            serde_json::from_str(&raw).expect("decode canonical session");
        assert_eq!(decoded.version, SESSION_VERSION);
        assert_eq!(
            decoded.workspaces[0].id.as_deref(),
            Some("22222222-2222-4222-8222-222222222222")
        );
        assert_eq!(decoded.workspaces[0].name, "workspace");
    }

    #[test]
    fn repeated_session_commit_failures_leave_one_bounded_pending_file() {
        let dir = tempdir().expect("tempdir");
        let target = canonical_session_path_in(dir.path());
        fs::create_dir(&target).expect("blocking target directory");

        assert!(save_session_atomic_in(dir.path(), &AppSessionState::default()).is_err());
        let changed = AppSessionState {
            top_bar_visible: false,
            ..AppSessionState::default()
        };
        assert!(save_session_atomic_in(dir.path(), &changed).is_err());

        let pending = dir.path().join(".session.json.pending");
        let raw = fs::read_to_string(pending).expect("bounded pending session");
        let decoded: AppSessionState = serde_json::from_str(&raw).expect("decode pending session");
        assert!(!decoded.top_bar_visible);
    }

    #[test]
    fn workspace_id_defaults_for_legacy_session_json() {
        let raw = r#"{
            "version": 1,
            "workspaces": [{
                "name": "legacy-shape",
                "favorite": false,
                "layout": {
                    "kind": "pane",
                    "active_tab_id": "terminal-0",
                    "tabs": [{
                        "id": "terminal-0",
                        "tab_kind": "terminal",
                        "cwd": "/tmp/project"
                    }]
                }
            }]
        }"#;

        let decoded: AppSessionState = serde_json::from_str(raw).expect("decode legacy shape");
        assert_eq!(decoded.workspaces[0].id, None);
    }

    #[test]
    fn hook_index_attaches_agent_to_matching_workspace_surface() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState::terminal("tab-a", Some("/tmp/project"))],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                let agent = agent.as_ref().expect("agent metadata");
                assert_eq!(agent.kind, RestorableAgentKind::Codex);
                assert_eq!(agent.session_id, "session-a");
                let command = agent.resume_command().expect("resume command");
                assert!(command.contains("cd '/tmp/project' && 'codex' 'resume' 'session-a'"));
                assert!(command.contains("hooks codex cleanup"));
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_index_falls_back_to_surface_when_workspace_id_drifted() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "old-workspace",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());

        let agent = index
            .agent_for_surface("new-workspace", Some(42), "tab-a")
            .expect("agent by surface fallback");
        assert_eq!(agent.kind, RestorableAgentKind::Codex);
        assert_eq!(agent.session_id, "session-a");
    }

    #[test]
    fn hook_index_does_not_fall_back_to_ambiguous_surface() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "29:terminal-0",
                        "cwd": "/tmp/project-a",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project-a",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    },
                    "session-b": {
                        "session_id": "session-b",
                        "workspace_id": "workspace-b",
                        "surface_id": "29:terminal-0",
                        "cwd": "/tmp/project-b",
                        "pid": 2,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project-b",
                            "environment": {},
                            "captured_at": 11.0
                        },
                        "updated_at": 11.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());

        assert!(
            index
                .agent_for_surface("workspace-c", Some(29), "terminal-0")
                .is_none(),
            "duplicate surfaces across workspaces must not pick an unrelated latest session"
        );
        let exact_agent = index
            .agent_for_surface("workspace-a", Some(29), "terminal-0")
            .expect("exact workspace/surface match");
        assert_eq!(exact_agent.session_id, "session-a");
    }

    #[test]
    fn hook_index_falls_back_to_tab_id_when_pane_id_is_missing() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "old-workspace",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());

        let agent = index
            .agent_for_surface("new-workspace", None, "tab-a")
            .expect("agent by tab id fallback");
        assert_eq!(agent.kind, RestorableAgentKind::Codex);
        assert_eq!(agent.session_id, "session-a");
    }

    #[test]
    fn hook_index_does_not_fall_back_to_ambiguous_tab_id() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:terminal-0",
                        "cwd": "/tmp/project-a",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project-a",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    },
                    "session-b": {
                        "session_id": "session-b",
                        "workspace_id": "workspace-b",
                        "surface_id": "99:terminal-0",
                        "cwd": "/tmp/project-b",
                        "pid": 2,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project-b",
                            "environment": {},
                            "captured_at": 11.0
                        },
                        "updated_at": 11.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());

        assert!(
            index
                .agent_for_surface("workspace-c", None, "terminal-0")
                .is_none(),
            "duplicate tab ids must not pick an unrelated latest session"
        );
        let exact_agent = index
            .agent_for_surface("workspace-a", Some(42), "terminal-0")
            .expect("exact surface match");
        assert_eq!(exact_agent.session_id, "session-a");
    }

    #[test]
    fn hook_merge_clears_persisted_agent_when_index_misses() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{"version":1,"sessions":{}}"#,
        )
        .expect("write empty hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "tab-a".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Terminal {
                    cwd: Some("/tmp/project".to_string()),
                    agent: Some(RestorableAgentState {
                        kind: RestorableAgentKind::Codex,
                        session_id: "persisted-session".to_string(),
                        cwd: Some("/tmp/project".to_string()),
                        launch_command: None,
                        restore_on_startup: true,
                        suspension_reason: None,
                        suspended_at: None,
                        hook_updated_at: None,
                        hook_observation_initialized: false,
                    }),
                },
            }],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                assert_eq!(agent, &None);
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_merge_preserves_persisted_agent_when_kind_store_unavailable() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("codex-hook-sessions.json"), "not json")
            .expect("write malformed hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "tab-a".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Terminal {
                    cwd: Some("/tmp/project".to_string()),
                    agent: Some(RestorableAgentState {
                        kind: RestorableAgentKind::Codex,
                        session_id: "persisted-session".to_string(),
                        cwd: Some("/tmp/project".to_string()),
                        launch_command: None,
                        restore_on_startup: true,
                        suspension_reason: None,
                        suspended_at: None,
                        hook_updated_at: None,
                        hook_observation_initialized: false,
                    }),
                },
            }],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                assert_eq!(
                    agent.as_ref().map(|agent| agent.session_id.as_str()),
                    Some("persisted-session")
                );
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_merge_preserves_persisted_agent_when_sessions_field_is_missing() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{"version":1}"#,
        )
        .expect("write malformed hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "tab-a".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Terminal {
                    cwd: Some("/tmp/project".to_string()),
                    agent: Some(RestorableAgentState {
                        kind: RestorableAgentKind::Codex,
                        session_id: "persisted-session".to_string(),
                        cwd: Some("/tmp/project".to_string()),
                        launch_command: None,
                        restore_on_startup: true,
                        suspension_reason: None,
                        suspended_at: None,
                        hook_updated_at: None,
                        hook_observation_initialized: false,
                    }),
                },
            }],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                assert_eq!(
                    agent.as_ref().map(|agent| agent.session_id.as_str()),
                    Some("persisted-session")
                );
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_merge_preserves_persisted_agent_when_different_kind_store_loaded() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("claude-hook-sessions.json"),
            r#"{"version":1,"sessions":{}}"#,
        )
        .expect("write empty hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "tab-a".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Terminal {
                    cwd: Some("/tmp/project".to_string()),
                    agent: Some(RestorableAgentState {
                        kind: RestorableAgentKind::Codex,
                        session_id: "persisted-session".to_string(),
                        cwd: Some("/tmp/project".to_string()),
                        launch_command: None,
                        restore_on_startup: true,
                        suspension_reason: None,
                        suspended_at: None,
                        hook_updated_at: None,
                        hook_observation_initialized: false,
                    }),
                },
            }],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                assert_eq!(
                    agent.as_ref().map(|agent| agent.session_id.as_str()),
                    Some("persisted-session")
                );
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_merge_preserves_no_resume_marker_when_index_misses() {
        let index = RestorableAgentIndex::default();
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "tab-a".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Terminal {
                    cwd: Some("/tmp/project".to_string()),
                    agent: Some(RestorableAgentState {
                        kind: RestorableAgentKind::Codex,
                        session_id: "manual-no-resume".to_string(),
                        cwd: Some("/tmp/project".to_string()),
                        launch_command: None,
                        restore_on_startup: false,
                        suspension_reason: None,
                        suspended_at: None,
                        hook_updated_at: None,
                        hook_observation_initialized: false,
                    }),
                },
            }],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                let agent = agent.as_ref().expect("no-resume marker");
                assert_eq!(agent.session_id, "manual-no-resume");
                assert!(!agent.restore_on_startup);
                assert_eq!(agent.resume_command(), None);
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn hook_merge_reactivates_unclean_suspension_after_fresh_hook_evidence() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 20.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false,
                    "suspension_reason": "unclean_restore",
                    "suspended_at": 15.0
                }
            }]
        }))
        .expect("decode suspended layout");

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected restored agent");
        };
        assert!(agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
    }

    #[test]
    fn hook_merge_keeps_unclean_suspension_when_hook_evidence_is_stale() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 10.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false,
                    "suspension_reason": "unclean_restore",
                    "suspended_at": 15.0
                }
            }]
        }))
        .expect("decode suspended layout");

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected retained agent");
        };
        assert!(!agent.restore_on_startup);
        assert_eq!(
            agent.suspension_reason,
            Some(AgentSuspensionReason::UncleanRestore)
        );
    }

    #[test]
    fn fresh_hook_reactivation_survives_clean_save_and_resumes_same_hcom_identity_once() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {
                                "HCOM_NAME": "lifo",
                                "HCOM_PROCESS_ID": "hcom-process-1"
                            },
                            "captured_at": 20.0
                        },
                        "updated_at": 20.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "launch_command": {
                        "executable": "codex",
                        "arguments": ["codex"],
                        "cwd": "/tmp/project",
                        "environment": {
                            "HCOM_NAME": "lifo",
                            "HCOM_PROCESS_ID": "hcom-process-1"
                        },
                        "captured_at": 10.0
                    },
                    "restore_on_startup": true
                }
            }]
        }))
        .expect("decode restorable layout");

        assert_eq!(suspend_agents_for_unclean_restore_at(&mut layout, 15.0), 1);
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let clean_save = serde_json::to_vec(&layout).expect("serialize clean save");
        let mut next_clean_start: LayoutNodeState =
            serde_json::from_slice(&clean_save).expect("reload clean save");
        attach_restorable_agents_to_layout(&mut next_clean_start, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = next_clean_start else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected restored agent");
        };
        let command = agent.resume_command().expect("reactivated resume command");
        assert_eq!(command.matches("'hcom' 'r' 'lifo'").count(), 1);
        assert!(command.contains("'--run-here' '--go'"));
        assert_eq!(agent.session_id, "session-a");
        assert!(agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
        assert_eq!(agent.suspended_at, None);
    }

    #[test]
    fn unchanged_future_dated_hook_stays_suspended_after_clock_rollback() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 200.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState::terminal("tab-a", Some("/tmp/project"))],
        });

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);
        assert_eq!(suspend_agents_for_unclean_restore_at(&mut layout, 150.0), 1);
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected retained agent");
        };
        assert!(!agent.restore_on_startup);
        assert_eq!(agent.resume_command(), None);
    }

    #[test]
    fn changed_hook_reactivates_even_when_clock_moves_backward() {
        let dir = tempdir().expect("tempdir");
        let hook_path = dir.path().join("codex-hook-sessions.json");
        fs::write(
            &hook_path,
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 200.0
                    }
                }
            }"#,
        )
        .expect("write initial hook state");
        let initial_index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(42),
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState::terminal("tab-a", Some("/tmp/project"))],
        });
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &initial_index);
        assert_eq!(suspend_agents_for_unclean_restore_at(&mut layout, 150.0), 1);

        fs::write(
            &hook_path,
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 140.0
                    }
                }
            }"#,
        )
        .expect("write post-rollback hook state");
        let changed_index = RestorableAgentIndex::load_from_dir(dir.path());
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &changed_index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected restored agent");
        };
        assert!(agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
    }

    #[test]
    fn unclean_suspend_preserves_manual_no_resume_through_fresh_hook_and_reload() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 20.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false
                }
            }]
        }))
        .expect("decode manual no-resume layout");

        assert_eq!(suspend_agents_for_unclean_restore_at(&mut layout, 15.0), 0);
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);
        let saved = serde_json::to_vec(&layout).expect("serialize layout");
        let reloaded: LayoutNodeState = serde_json::from_slice(&saved).expect("reload layout");

        let LayoutNodeState::Pane(pane) = reloaded else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected retained manual agent");
        };
        assert!(!agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
        assert_eq!(agent.resume_command(), None);
    }

    #[test]
    fn unclean_suspend_preserves_user_choice_suspension_through_fresh_hook() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "updated_at": 20.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false,
                    "suspension_reason": "user_choice"
                }
            }]
        }))
        .expect("decode user-choice suspension");

        assert_eq!(suspend_agents_for_unclean_restore_at(&mut layout, 15.0), 0);
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected retained user-choice agent");
        };
        assert!(!agent.restore_on_startup);
        assert_eq!(
            agent.suspension_reason,
            Some(AgentSuspensionReason::UserChoice)
        );
        assert_eq!(agent.resume_command(), None);
    }

    #[test]
    fn legacy_unclean_suspension_uses_changed_hook_observation_across_clock_rollback() {
        let stale_dir = tempdir().expect("stale tempdir");
        let fresh_dir = tempdir().expect("fresh tempdir");
        for (dir, updated_at) in [(stale_dir.path(), 200.0), (fresh_dir.path(), 140.0)] {
            fs::write(
                dir.join("codex-hook-sessions.json"),
                format!(
                    r#"{{
                        "version": 1,
                        "sessions": {{
                            "session-a": {{
                                "session_id": "session-a",
                                "workspace_id": "workspace-a",
                                "surface_id": "42:tab-a",
                                "cwd": "/tmp/project",
                                "launch_command": {{
                                    "executable": "codex",
                                    "arguments": ["codex"],
                                    "cwd": "/tmp/project",
                                    "environment": {{
                                        "HCOM_NAME": "lifo",
                                        "HCOM_PROCESS_ID": "hcom-process-1"
                                    }},
                                    "captured_at": {updated_at}
                                }},
                                "updated_at": {updated_at}
                            }}
                        }}
                    }}"#
                ),
            )
            .expect("write hook state");
        }
        let stale_index = RestorableAgentIndex::load_from_dir(stale_dir.path());
        let fresh_index = RestorableAgentIndex::load_from_dir(fresh_dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false,
                    "suspension_reason": "unclean_restore"
                }
            }]
        }))
        .expect("decode legacy suspension");

        seed_legacy_unclean_suspension_baseline(
            &mut layout,
            "workspace-a",
            &stale_index,
            Some(200.0),
        );
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &stale_index);
        let LayoutNodeState::Pane(stale_pane) = &layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(stale_agent),
            ..
        } = &stale_pane.tabs[0].content
        else {
            panic!("expected stale agent");
        };
        assert!(!stale_agent.restore_on_startup);
        assert_eq!(stale_agent.resume_command(), None);
        assert_eq!(stale_agent.hook_updated_at, Some(200.0));
        assert!(stale_agent.hook_observation_initialized);

        let migrated = serde_json::to_vec(&layout).expect("serialize legacy migration");
        let mut layout: LayoutNodeState =
            serde_json::from_slice(&migrated).expect("reload legacy migration");

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &fresh_index);
        let saved = serde_json::to_vec(&layout).expect("serialize migrated layout");
        let reloaded: LayoutNodeState = serde_json::from_slice(&saved).expect("reload layout");
        let LayoutNodeState::Pane(pane) = reloaded else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected migrated agent");
        };
        assert!(agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
        assert_eq!(agent.session_id, "session-a");
        let command = agent
            .resume_command()
            .expect("migrated hcom resume command");
        assert_eq!(command.matches("'hcom' 'r' 'lifo'").count(), 1);
        assert!(command.contains("'--run-here' '--go'"));
    }

    #[test]
    fn legacy_unclean_suspension_treats_first_hook_after_observed_absence_as_fresh() {
        let absent_dir = tempdir().expect("absent tempdir");
        let fresh_dir = tempdir().expect("fresh tempdir");
        fs::write(
            fresh_dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "workspace-a",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {
                                "HCOM_NAME": "lifo",
                                "HCOM_PROCESS_ID": "hcom-process-1"
                            },
                            "captured_at": 140.0
                        },
                        "updated_at": 140.0
                    }
                }
            }"#,
        )
        .expect("write fresh hook state");
        let absent_index = RestorableAgentIndex::load_from_dir(absent_dir.path());
        let fresh_index = RestorableAgentIndex::load_from_dir(fresh_dir.path());
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 42,
            "active_tab_id": "tab-a",
            "tabs": [{
                "id": "tab-a",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "session-a",
                    "cwd": "/tmp/project",
                    "restore_on_startup": false,
                    "suspension_reason": "unclean_restore"
                }
            }]
        }))
        .expect("decode legacy suspension");

        seed_legacy_unclean_suspension_baseline(
            &mut layout,
            "workspace-a",
            &absent_index,
            Some(200.0),
        );
        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &absent_index);
        let migrated = serde_json::to_vec(&layout).expect("serialize absent observation");
        let mut layout: LayoutNodeState =
            serde_json::from_slice(&migrated).expect("reload absent observation");

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &absent_index);
        let LayoutNodeState::Pane(absent_pane) = &layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(absent_agent),
            ..
        } = &absent_pane.tabs[0].content
        else {
            panic!("expected suspended agent");
        };
        assert!(!absent_agent.restore_on_startup);
        assert_eq!(absent_agent.resume_command(), None);
        assert_eq!(absent_agent.hook_updated_at, None);
        assert!(absent_agent.hook_observation_initialized);

        attach_restorable_agents_to_layout(&mut layout, "workspace-a", &fresh_index);
        let saved = serde_json::to_vec(&layout).expect("serialize first fresh hook");
        let reloaded: LayoutNodeState = serde_json::from_slice(&saved).expect("reload fresh hook");
        let LayoutNodeState::Pane(pane) = reloaded else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected reactivated agent");
        };
        assert!(agent.restore_on_startup);
        assert_eq!(agent.suspension_reason, None);
        let command = agent
            .resume_command()
            .expect("reactivated hcom resume command");
        assert_eq!(command.matches("'hcom' 'r' 'lifo'").count(), 1);
        assert!(command.contains("'--run-here' '--go'"));
    }

    #[test]
    fn hook_merge_recovers_agent_without_workspace_or_pane_id() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("codex-hook-sessions.json"),
            r#"{
                "version": 1,
                "sessions": {
                    "session-a": {
                        "session_id": "session-a",
                        "workspace_id": "old-workspace",
                        "surface_id": "42:tab-a",
                        "cwd": "/tmp/project",
                        "pid": 1,
                        "launch_command": {
                            "executable": "codex",
                            "arguments": ["codex"],
                            "cwd": "/tmp/project",
                            "environment": {},
                            "captured_at": 10.0
                        },
                        "updated_at": 10.0
                    }
                }
            }"#,
        )
        .expect("write hook state");
        let index = RestorableAgentIndex::load_from_dir(dir.path());
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: None,
            active_tab_id: Some("tab-a".to_string()),
            flag_color: None,
            tabs: vec![TabState::terminal("tab-a", Some("/tmp/project"))],
        });

        attach_restorable_agents_to_layout(&mut layout, "", &index);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        match &pane.tabs[0].content {
            TabContentState::Terminal { agent, .. } => {
                assert_eq!(
                    agent.as_ref().map(|agent| agent.session_id.as_str()),
                    Some("session-a")
                );
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn terminal_tab_state_round_trips_restorable_agent_metadata() {
        let tab = TabState {
            id: "tab-a".to_string(),
            custom_name: None,
            pinned: false,
            content: TabContentState::Terminal {
                cwd: Some("/tmp/project".to_string()),
                agent: Some(RestorableAgentState {
                    kind: RestorableAgentKind::Codex,
                    session_id: "sess-123".to_string(),
                    cwd: Some("/tmp/project".to_string()),
                    launch_command: Some(AgentLaunchCommandState {
                        executable: "codex".to_string(),
                        arguments: vec![
                            "codex".to_string(),
                            "--model".to_string(),
                            "gpt-5.5".to_string(),
                        ],
                        cwd: Some("/tmp/project".to_string()),
                        environment: Default::default(),
                        captured_at: Some(12.0),
                    }),
                    restore_on_startup: true,
                    suspension_reason: None,
                    suspended_at: None,
                    hook_updated_at: None,
                    hook_observation_initialized: false,
                }),
            },
        };

        let raw = serde_json::to_string(&tab).expect("encode tab");
        let decoded: TabState = serde_json::from_str(&raw).expect("decode tab");

        match decoded.content {
            TabContentState::Terminal { agent, .. } => {
                let agent = agent.expect("agent metadata");
                assert_eq!(agent.kind, RestorableAgentKind::Codex);
                assert_eq!(agent.session_id, "sess-123");
                assert_eq!(
                    agent
                        .launch_command
                        .expect("launch command")
                        .arguments
                        .as_slice(),
                    ["codex", "--model", "gpt-5.5"]
                );
            }
            other => panic!("expected terminal tab, got {other:?}"),
        }
    }

    #[test]
    fn restorable_agent_resume_command_runs_from_cwd() {
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Codex,
            session_id: "sess-123".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: Some(AgentLaunchCommandState {
                executable: "codex".to_string(),
                arguments: vec!["codex".to_string()],
                cwd: Some("/tmp/project".to_string()),
                environment: Default::default(),
                captured_at: Some(12.0),
            }),
            restore_on_startup: true,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        let command = agent.resume_command().expect("resume command");
        assert!(command.contains("cd '/tmp/project' && 'codex' 'resume' 'sess-123'"));
        assert!(command.contains("hooks codex cleanup"));
        assert!(command.contains("exec \"${SHELL:-/bin/sh}\" -l"));
    }

    #[test]
    fn restorable_hcom_agent_resume_command_prefers_hcom_name() {
        let mut environment = BTreeMap::new();
        environment.insert("HCOM_PROCESS_ID".to_string(), "hcom-process-1".to_string());
        environment.insert("HCOM_NAME".to_string(), "lifo".to_string());
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Codex,
            session_id: "sess-123".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: Some(AgentLaunchCommandState {
                executable: "codex".to_string(),
                arguments: vec!["codex".to_string()],
                cwd: Some("/tmp/project".to_string()),
                environment,
                captured_at: Some(12.0),
            }),
            restore_on_startup: true,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        let command = agent.resume_command().expect("resume command");
        assert!(command.contains("cd '/tmp/project' && 'hcom' 'r' 'lifo' '--run-here' '--go'"));
        assert!(!command.contains("'codex' 'resume'"));
        assert!(command.contains("hooks codex cleanup"));
    }

    #[test]
    fn restorable_hcom_agent_resume_command_falls_back_to_session_id() {
        let mut environment = BTreeMap::new();
        environment.insert("HCOM_PROCESS_ID".to_string(), "hcom-process-1".to_string());
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Hermes,
            session_id: "20260624_132006_02638e".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: Some(AgentLaunchCommandState {
                executable: "hermes".to_string(),
                arguments: vec!["hermes".to_string()],
                cwd: Some("/tmp/project".to_string()),
                environment,
                captured_at: Some(12.0),
            }),
            restore_on_startup: true,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        let command = agent.resume_command().expect("resume command");
        assert!(command.contains(
            "cd '/tmp/project' && 'hcom' 'r' '20260624_132006_02638e' '--run-here' '--go'"
        ));
        assert!(!command.contains("'hermes' '--resume'"));
        assert!(command.contains("hooks hermes cleanup"));
    }

    #[test]
    fn restorable_hermes_resume_command_uses_native_resume_flag() {
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Hermes,
            session_id: "20260624_132006_02638e".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: Some(AgentLaunchCommandState {
                executable: "hermes".to_string(),
                arguments: vec![
                    "hermes".to_string(),
                    "--model".to_string(),
                    "anthropic/claude-sonnet-4.6".to_string(),
                ],
                cwd: Some("/tmp/project".to_string()),
                environment: Default::default(),
                captured_at: Some(12.0),
            }),
            restore_on_startup: true,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        let command = agent.resume_command().expect("resume command");
        assert!(
            command.contains("cd '/tmp/project' && 'hermes' '--resume' '20260624_132006_02638e'")
        );
        assert!(command.contains("hooks hermes cleanup"));
    }

    #[test]
    fn restorable_agent_resume_command_drops_dangerous_launch_flags() {
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Codex,
            session_id: "sess-123".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: Some(AgentLaunchCommandState {
                executable: "codex".to_string(),
                arguments: vec![
                    "codex".to_string(),
                    "--model".to_string(),
                    "gpt-5.5".to_string(),
                    "--dangerously-bypass-approvals-and-sandbox".to_string(),
                    "--dangerously-skip-permissions".to_string(),
                    "--search".to_string(),
                    "prompt text".to_string(),
                ],
                cwd: Some("/tmp/project".to_string()),
                environment: Default::default(),
                captured_at: Some(12.0),
            }),
            restore_on_startup: true,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        let command = agent.resume_command().expect("resume command");
        assert!(command.contains("'--model' 'gpt-5.5'"));
        assert!(command.contains("'--search'"));
        assert!(!command.contains("dangerously"));
        assert!(!command.contains("prompt text"));
    }

    #[test]
    fn legacy_restorable_agent_without_restore_marker_does_not_resume() {
        let agent = RestorableAgentState {
            kind: RestorableAgentKind::Codex,
            session_id: "old-stale-session".to_string(),
            cwd: Some("/tmp/project".to_string()),
            launch_command: None,
            restore_on_startup: false,
            suspension_reason: None,
            suspended_at: None,
            hook_updated_at: None,
            hook_observation_initialized: false,
        };

        assert_eq!(agent.resume_command(), None);
    }

    #[test]
    fn normalize_layout_falls_back_to_first_tab_when_active_tab_is_stale() {
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: None,
            active_tab_id: Some("missing".to_string()),
            flag_color: None,
            tabs: vec![TabState {
                id: "browser-1".to_string(),
                custom_name: None,
                pinned: false,
                content: TabContentState::Browser {
                    uri: Some("https://example.com".to_string()),
                },
            }],
        });

        normalize_layout(&mut layout, None);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        assert_eq!(pane.active_tab_id.as_deref(), Some("browser-1"));
    }

    #[test]
    fn normalize_layout_renames_duplicate_tab_ids_within_pane() {
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: Some(15),
            active_tab_id: Some("terminal-0".to_string()),
            flag_color: None,
            tabs: vec![
                TabState::terminal("terminal-0", Some("/tmp/first")),
                TabState::terminal("terminal-0", Some("/tmp/second")),
                TabState::terminal("terminal-0-1", Some("/tmp/third")),
                TabState::terminal("", Some("/tmp/fourth")),
                TabState::terminal("", Some("/tmp/fifth")),
            ],
        });

        normalize_layout(&mut layout, None);

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let tab_ids = pane
            .tabs
            .iter()
            .map(|tab| tab.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            tab_ids,
            vec![
                "terminal-0",
                "terminal-0-1",
                "terminal-0-1-1",
                "tab",
                "tab-1"
            ]
        );
        assert_eq!(pane.active_tab_id.as_deref(), Some("terminal-0"));
    }

    #[test]
    fn normalize_layout_rebuilds_empty_pane_from_working_directory() {
        let mut layout = LayoutNodeState::Pane(PaneState {
            pane_id: None,
            active_tab_id: None,
            flag_color: None,
            tabs: Vec::new(),
        });

        normalize_layout(&mut layout, Some("/tmp/project"));

        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        assert_eq!(pane.tabs.len(), 1);
        match &pane.tabs[0].content {
            TabContentState::Terminal { cwd, .. } => {
                assert_eq!(cwd.as_deref(), Some("/tmp/project"));
            }
            other => panic!("expected terminal fallback, got {other:?}"),
        }
    }

    #[test]
    fn browser_only_pane_creates_a_single_browser_tab() {
        let pane = PaneState::browser_only(Some("https://example.com"));

        assert_eq!(pane.tabs.len(), 1);
        assert_eq!(pane.active_tab_id.as_deref(), Some("browser-0"));
        match &pane.tabs[0].content {
            TabContentState::Browser { uri } => {
                assert_eq!(uri.as_deref(), Some("https://example.com"));
            }
            other => panic!("expected browser tab, got {other:?}"),
        }
    }

    #[test]
    fn pane_flag_color_round_trips_and_defaults_from_legacy_json() {
        let pane = PaneState {
            pane_id: Some(7),
            active_tab_id: Some("terminal-0".to_string()),
            flag_color: Some(PaneFlagColor::Purple),
            tabs: Vec::new(),
        };

        let raw = serde_json::to_string(&pane).expect("serialize pane state");
        assert!(raw.contains(r#""flag_color":"purple""#));
        let decoded: PaneState = serde_json::from_str(&raw).expect("decode pane state");
        assert_eq!(decoded.flag_color, Some(PaneFlagColor::Purple));

        let legacy: PaneState =
            serde_json::from_str(r#"{"pane_id":7,"active_tab_id":"terminal-0","tabs":[]}"#)
                .expect("decode legacy pane state");
        assert_eq!(legacy.flag_color, None);
    }

    #[test]
    fn keybind_tab_round_trips_through_session_json() {
        let state = AppSessionState {
            top_bar_visible: false,
            workspaces: vec![WorkspaceState {
                id: Some("33333333-3333-4333-8333-333333333333".to_string()),
                name: "workspace".to_string(),
                favorite: false,
                highlight: Some(WorkspaceHighlightColor::Orange),
                cwd: None,
                folder_path: None,
                layout: LayoutNodeState::Pane(PaneState {
                    pane_id: None,
                    active_tab_id: Some("keybinds-1".to_string()),
                    flag_color: None,
                    tabs: vec![TabState {
                        id: "keybinds-1".to_string(),
                        custom_name: None,
                        pinned: false,
                        content: TabContentState::Keybinds {},
                    }],
                }),
            }],
            ..AppSessionState::default()
        };

        let raw = serde_json::to_string(&state).expect("serialize session");
        let decoded: AppSessionState = serde_json::from_str(&raw).expect("deserialize session");

        assert!(!decoded.top_bar_visible);
        assert_eq!(
            decoded.workspaces[0].highlight,
            Some(WorkspaceHighlightColor::Orange)
        );
        let LayoutNodeState::Pane(pane) = &decoded.workspaces[0].layout else {
            panic!("expected pane");
        };
        assert_eq!(pane.active_tab_id.as_deref(), Some("keybinds-1"));
        assert!(matches!(pane.tabs[0].content, TabContentState::Keybinds {}));
    }

    #[test]
    fn split_ratio_helpers_clamp_invalid_values() {
        assert_eq!(clamp_split_ratio(f64::NAN), DEFAULT_SPLIT_RATIO);
        assert_eq!(split_ratio_from_position(0, 0), DEFAULT_SPLIT_RATIO);
        assert!(split_ratio_from_position(9999, 10) <= MAX_SPLIT_RATIO);
        assert_eq!(split_position_from_ratio(f64::INFINITY, 200), 100);
        assert_eq!(clamp_split_ratio(0.001), 0.08);
        assert_eq!(clamp_split_ratio(0.999), 0.92);
    }

    #[test]
    fn snapshot_split_ratio_preserves_stored_ratio_when_unallocated() {
        assert_eq!(snapshot_split_ratio(0, 0, Some(0.73)), 0.73);
        assert_eq!(
            snapshot_split_ratio(0, 0, Some(f64::INFINITY)),
            DEFAULT_SPLIT_RATIO
        );
        assert_eq!(snapshot_split_ratio(0, 0, None), DEFAULT_SPLIT_RATIO);
    }

    #[test]
    fn split_ratio_helpers_respect_child_pixel_minimums() {
        assert_eq!(clamp_split_ratio_for_size(0.05, 1000, 260), 0.26);
        assert_eq!(clamp_split_ratio_for_size(0.95, 1000, 260), 0.74);
        assert_eq!(
            clamp_split_ratio_for_size(0.2, 400, 260),
            DEFAULT_SPLIT_RATIO
        );
        assert_eq!(split_position_from_ratio_with_min(0.01, 1000, 260), 260);
        assert_eq!(split_ratio_from_position_with_min(30, 1000, 260), 0.26);
    }

    #[test]
    fn snapshot_split_ratio_with_min_preserves_stored_ratio_when_undersized() {
        assert_eq!(
            snapshot_split_ratio_with_min(200, 400, Some(0.31), 260),
            0.31
        );
        assert_eq!(
            snapshot_split_ratio_with_min(200, 400, None, 260),
            DEFAULT_SPLIT_RATIO
        );
    }

    #[test]
    fn normalize_session_preserves_compact_sidebar_width() {
        let state = normalize_session(AppSessionState {
            sidebar: SidebarState {
                visible: true,
                width: MIN_SIDEBAR_WIDTH,
            },
            ..AppSessionState::default()
        });

        assert_eq!(state.sidebar.width, MIN_SIDEBAR_WIDTH);
    }

    #[test]
    fn unclean_restore_suspends_agents_without_erasing_resume_metadata() {
        let mut layout: LayoutNodeState = serde_json::from_value(serde_json::json!({
            "kind": "pane",
            "pane_id": 7,
            "active_tab_id": "agent-tab",
            "tabs": [{
                "id": "agent-tab",
                "tab_kind": "terminal",
                "cwd": "/tmp/project",
                "agent": {
                    "kind": "codex",
                    "session_id": "native-session-123",
                    "cwd": "/tmp/project",
                    "launch_command": {
                        "executable": "hcom",
                        "arguments": ["r", "lifo", "--run-here"],
                        "cwd": "/tmp/project",
                        "environment": {"HCOM_NAME": "lifo"},
                        "captured_at": 42.0
                    },
                    "restore_on_startup": true
                }
            }]
        }))
        .expect("decode layout");

        let suspended = suspend_agents_for_unclean_restore(&mut layout);

        assert_eq!(suspended, 1);
        let LayoutNodeState::Pane(pane) = layout else {
            panic!("expected pane");
        };
        let TabContentState::Terminal {
            agent: Some(agent), ..
        } = &pane.tabs[0].content
        else {
            panic!("expected retained agent metadata");
        };
        assert!(!agent.restore_on_startup);
        assert_eq!(
            agent.suspension_reason,
            Some(AgentSuspensionReason::UncleanRestore)
        );
        assert_eq!(agent.kind, RestorableAgentKind::Codex);
        assert_eq!(agent.session_id, "native-session-123");
        assert_eq!(agent.cwd.as_deref(), Some("/tmp/project"));
        let launch = agent.launch_command.as_ref().expect("launch metadata");
        assert_eq!(launch.executable, "hcom");
        assert_eq!(launch.arguments, ["r", "lifo", "--run-here"]);
        assert_eq!(
            launch.environment.get("HCOM_NAME").map(String::as_str),
            Some("lifo")
        );
    }
}
