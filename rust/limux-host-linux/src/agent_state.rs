//! PRD-G agent lifecycle state machine (pure, GTK-free).
//!
//! Tracks a per-surface agent state (`unknown → running → needs-input →
//! idle`) fed by hook events, aggregates it per workspace, and handles the
//! `acknowledged` urgency bit, stale-`running` decay, and eviction on
//! surface/pane close. Time is injected (`now_ms`) so tests never sleep.
//!
//! GTK sidebar wiring, the `surface.agent_event` socket method, and the
//! `agents-status` CLI consume this module in the next PRD-G slice.
#![allow(dead_code)] // remove when the GTK/socket wiring slice lands

use std::collections::BTreeMap;

/// Default stale-`running` decay window: 30 minutes (PRD-G US-1).
pub(crate) const DEFAULT_STALE_AFTER_MS: u64 = 30 * 60 * 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AgentState {
    Unknown,
    Running,
    NeedsInput,
    Idle,
}

impl AgentState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Running => "running",
            Self::NeedsInput => "needs-input",
            Self::Idle => "idle",
        }
    }
}

/// Agent family, mirrored from the CLI-side `agent_hooks::AgentKind`
/// vocabulary (that type is crate-private to limux-cli).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AgentKind {
    Claude,
    Codex,
    OpenCode,
    Gemini,
    Hermes,
}

impl AgentKind {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "claude" | "claude-code" | "claudecode" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "opencode" | "open-code" => Some(Self::OpenCode),
            "gemini" => Some(Self::Gemini),
            "hermes" | "hermes-agent" | "hermes-cli" => Some(Self::Hermes),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Gemini => "gemini",
            Self::Hermes => "hermes",
        }
    }
}

/// Transition class of a hook event (PRD-G state model).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AgentEvent {
    /// Agent started / prompt submitted / tool activity → `running`.
    Activity,
    /// Awaiting user input / permission request / notification → `needs-input`.
    NeedsInput,
    /// Stop / end-of-turn / session end → `idle`.
    Stop,
}

impl AgentEvent {
    /// Classify a raw hook event name into a transition class.
    ///
    /// The vocabulary mirrors `canonical_agent_hook_display_event` in
    /// limux-cli so the `surface.agent_event` shim can forward event names
    /// unchanged. Unrecognized events return `None` (no transition — never
    /// guessed).
    pub(crate) fn from_hook_event(event: &str) -> Option<Self> {
        match event.trim() {
            "Notification" | "notification" | "pre_approval_request" | "pre-approval-request" => {
                Some(Self::NeedsInput)
            }
            "Stop"
            | "stop"
            | "SubagentStop"
            | "subagent-stop"
            | "subagent_stop"
            | "post_llm_call"
            | "post-llm-call"
            | "SessionEnd"
            | "session-end"
            | "session_end"
            | "on_session_end"
            | "on_session_finalize"
            | "session_finalize" => Some(Self::Stop),
            "SessionStart" | "session-start" | "session_start" | "on_session_start"
            | "session-started" | "UserPromptSubmit" | "prompt-submit" | "user-prompt-submit"
            | "user_prompt_submit" | "pre_llm_call" | "pre-llm-call" | "PreToolUse"
            | "pre-tool-use" | "pre_tool_use" | "PostToolUse" | "post-tool-use"
            | "post_tool_use" | "pre_tool_call" | "pre-tool-call" | "post_tool_call"
            | "post-tool-call" => Some(Self::Activity),
            _ => None,
        }
    }

    fn target_state(self) -> AgentState {
        match self {
            Self::Activity => AgentState::Running,
            Self::NeedsInput => AgentState::NeedsInput,
            Self::Stop => AgentState::Idle,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SurfaceAgentRecord {
    pub(crate) kind: Option<AgentKind>,
    pub(crate) state: AgentState,
    /// Injected monotonic timestamp of the last hook event (milliseconds).
    pub(crate) last_event_at_ms: u64,
    /// Urgency bit: `false` after a needs-input event until the operator
    /// focuses the workspace. State itself never changes on focus — only
    /// this bit — so socket-reported state and sidebar never disagree.
    pub(crate) acknowledged: bool,
}

impl SurfaceAgentRecord {
    fn needs_attention(&self) -> bool {
        self.state == AgentState::NeedsInput && !self.acknowledged
    }
}

/// Result of applying one event, so callers can do targeted row updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ApplyOutcome {
    pub(crate) workspace_state_before: AgentState,
    pub(crate) workspace_state_after: AgentState,
}

impl ApplyOutcome {
    pub(crate) fn workspace_changed(self) -> bool {
        self.workspace_state_before != self.workspace_state_after
    }
}

/// Per-surface agent state store with workspace aggregation.
///
/// Keys are the host-linux identifiers: workspace id (`String`) and surface
/// id (the `LIMUX_SURFACE_ID` contract, `"{pane_id}:{tab_id}"`).
#[derive(Debug, Default)]
pub(crate) struct AgentStateStore {
    workspaces: BTreeMap<String, BTreeMap<String, SurfaceAgentRecord>>,
    stale_after_ms: u64,
}

impl AgentStateStore {
    pub(crate) fn new() -> Self {
        Self::with_stale_after(DEFAULT_STALE_AFTER_MS)
    }

    pub(crate) fn with_stale_after(stale_after_ms: u64) -> Self {
        Self {
            workspaces: BTreeMap::new(),
            stale_after_ms,
        }
    }

    pub(crate) fn set_stale_after_ms(&mut self, stale_after_ms: u64) {
        self.stale_after_ms = stale_after_ms;
    }

    pub(crate) fn stale_after_ms(&self) -> u64 {
        self.stale_after_ms
    }

    /// Apply a hook event to a surface. `kind` updates the recorded agent
    /// family when known; `None` preserves the existing one.
    pub(crate) fn apply_event(
        &mut self,
        workspace_id: &str,
        surface_id: &str,
        kind: Option<AgentKind>,
        event: AgentEvent,
        now_ms: u64,
    ) -> ApplyOutcome {
        let before = self.workspace_state(workspace_id);
        let surfaces = self.workspaces.entry(workspace_id.to_string()).or_default();
        let record = surfaces
            .entry(surface_id.to_string())
            .or_insert(SurfaceAgentRecord {
                kind: None,
                state: AgentState::Unknown,
                last_event_at_ms: now_ms,
                acknowledged: true,
            });
        if kind.is_some() {
            record.kind = kind;
        }
        record.state = event.target_state();
        record.last_event_at_ms = now_ms;
        // A fresh needs-input re-arms the urgency bit; anything else leaves
        // nothing to acknowledge.
        record.acknowledged = event != AgentEvent::NeedsInput;
        ApplyOutcome {
            workspace_state_before: before,
            workspace_state_after: self.workspace_state(workspace_id),
        }
    }

    /// Operator focused the workspace: clear the urgency bit (like unread).
    /// State is untouched. Returns true if any surface's bit changed.
    pub(crate) fn acknowledge_workspace(&mut self, workspace_id: &str) -> bool {
        let Some(surfaces) = self.workspaces.get_mut(workspace_id) else {
            return false;
        };
        let mut changed = false;
        for record in surfaces.values_mut() {
            if !record.acknowledged {
                record.acknowledged = true;
                changed = true;
            }
        }
        changed
    }

    /// Degrade `running` surfaces with no events for `stale_after_ms` to
    /// `unknown` (never silently forever-running). Returns the workspaces
    /// whose aggregate state changed.
    pub(crate) fn decay_stale(&mut self, now_ms: u64) -> Vec<String> {
        let mut changed = Vec::new();
        for (workspace_id, surfaces) in &mut self.workspaces {
            let before = Self::aggregate(surfaces.values());
            let mut any = false;
            for record in surfaces.values_mut() {
                if record.state == AgentState::Running
                    && now_ms.saturating_sub(record.last_event_at_ms) >= self.stale_after_ms
                {
                    record.state = AgentState::Unknown;
                    any = true;
                }
            }
            if any && Self::aggregate(surfaces.values()) != before {
                changed.push(workspace_id.clone());
            }
        }
        changed
    }

    /// Aggregated workspace state: any `needs-input` → needs-input; else any
    /// `running` → running; else any `idle` → idle; else unknown.
    pub(crate) fn workspace_state(&self, workspace_id: &str) -> AgentState {
        self.workspaces
            .get(workspace_id)
            .map(|surfaces| Self::aggregate(surfaces.values()))
            .unwrap_or(AgentState::Unknown)
    }

    /// True when the workspace has an unacknowledged `needs-input` surface —
    /// the sidebar's visually-dominant urgency signal.
    pub(crate) fn workspace_needs_attention(&self, workspace_id: &str) -> bool {
        self.workspaces
            .get(workspace_id)
            .is_some_and(|surfaces| surfaces.values().any(SurfaceAgentRecord::needs_attention))
    }

    pub(crate) fn surface_record(
        &self,
        workspace_id: &str,
        surface_id: &str,
    ) -> Option<&SurfaceAgentRecord> {
        self.workspaces.get(workspace_id)?.get(surface_id)
    }

    /// Per-surface records for a workspace, sorted by surface id (feeds
    /// `sidebar-state` / `agents-status`).
    pub(crate) fn surfaces(
        &self,
        workspace_id: &str,
    ) -> impl Iterator<Item = (&str, &SurfaceAgentRecord)> {
        self.workspaces
            .get(workspace_id)
            .into_iter()
            .flat_map(|surfaces| surfaces.iter().map(|(id, record)| (id.as_str(), record)))
    }

    /// Workspace ids with at least one tracked surface, sorted.
    pub(crate) fn workspace_ids(&self) -> impl Iterator<Item = &str> {
        self.workspaces.keys().map(String::as_str)
    }

    /// Remove a closed surface's state so it can't pin the aggregate.
    /// Returns the aggregate transition like `apply_event`.
    pub(crate) fn evict_surface(&mut self, workspace_id: &str, surface_id: &str) -> ApplyOutcome {
        let before = self.workspace_state(workspace_id);
        if let Some(surfaces) = self.workspaces.get_mut(workspace_id) {
            surfaces.remove(surface_id);
            if surfaces.is_empty() {
                self.workspaces.remove(workspace_id);
            }
        }
        ApplyOutcome {
            workspace_state_before: before,
            workspace_state_after: self.workspace_state(workspace_id),
        }
    }

    /// Remove every surface belonging to a closed pane. Surface ids follow
    /// the `LIMUX_SURFACE_ID` contract `"{pane_id}:{tab_id}"`.
    pub(crate) fn evict_pane(&mut self, workspace_id: &str, pane_id: u32) -> ApplyOutcome {
        let before = self.workspace_state(workspace_id);
        let prefix = format!("{pane_id}:");
        if let Some(surfaces) = self.workspaces.get_mut(workspace_id) {
            surfaces.retain(|surface_id, _| !surface_id.starts_with(&prefix));
            if surfaces.is_empty() {
                self.workspaces.remove(workspace_id);
            }
        }
        ApplyOutcome {
            workspace_state_before: before,
            workspace_state_after: self.workspace_state(workspace_id),
        }
    }

    /// Remove all state for a closed workspace.
    pub(crate) fn evict_workspace(&mut self, workspace_id: &str) {
        self.workspaces.remove(workspace_id);
    }

    fn aggregate<'a>(records: impl Iterator<Item = &'a SurfaceAgentRecord>) -> AgentState {
        let mut state = AgentState::Unknown;
        for record in records {
            match record.state {
                AgentState::NeedsInput => return AgentState::NeedsInput,
                AgentState::Running => state = AgentState::Running,
                AgentState::Idle => {
                    if state != AgentState::Running {
                        state = AgentState::Idle;
                    }
                }
                AgentState::Unknown => {}
            }
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WS: &str = "workspace-1";

    fn store() -> AgentStateStore {
        AgentStateStore::new()
    }

    #[test]
    fn unseen_workspace_is_unknown() {
        let store = store();
        assert_eq!(store.workspace_state(WS), AgentState::Unknown);
        assert!(!store.workspace_needs_attention(WS));
        assert_eq!(store.surfaces(WS).count(), 0);
    }

    #[test]
    fn activity_event_moves_surface_to_running() {
        let mut store = store();
        let outcome =
            store.apply_event(WS, "1:a", Some(AgentKind::Claude), AgentEvent::Activity, 0);
        assert_eq!(outcome.workspace_state_before, AgentState::Unknown);
        assert_eq!(outcome.workspace_state_after, AgentState::Running);
        assert!(outcome.workspace_changed());
        let record = store.surface_record(WS, "1:a").unwrap();
        assert_eq!(record.state, AgentState::Running);
        assert_eq!(record.kind, Some(AgentKind::Claude));
    }

    #[test]
    fn full_lifecycle_running_needs_input_idle() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 0);
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 1);
        assert_eq!(store.workspace_state(WS), AgentState::NeedsInput);
        store.apply_event(WS, "1:a", None, AgentEvent::Stop, 2);
        assert_eq!(store.workspace_state(WS), AgentState::Idle);
    }

    #[test]
    fn aggregation_precedence_needs_input_over_running_over_idle() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::Stop, 0);
        assert_eq!(store.workspace_state(WS), AgentState::Idle);
        store.apply_event(WS, "2:b", None, AgentEvent::Activity, 0);
        assert_eq!(store.workspace_state(WS), AgentState::Running);
        store.apply_event(WS, "3:c", None, AgentEvent::NeedsInput, 0);
        assert_eq!(store.workspace_state(WS), AgentState::NeedsInput);
    }

    #[test]
    fn workspaces_are_independent() {
        let mut store = store();
        store.apply_event("ws-a", "1:a", None, AgentEvent::NeedsInput, 0);
        store.apply_event("ws-b", "1:a", None, AgentEvent::Stop, 0);
        assert_eq!(store.workspace_state("ws-a"), AgentState::NeedsInput);
        assert_eq!(store.workspace_state("ws-b"), AgentState::Idle);
    }

    #[test]
    fn evicting_needs_input_surface_unpins_workspace_aggregate() {
        // Codex-required: a closed surface stuck in needs-input must not pin
        // the workspace aggregate forever.
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        store.apply_event(WS, "2:b", None, AgentEvent::Stop, 0);
        assert_eq!(store.workspace_state(WS), AgentState::NeedsInput);
        let outcome = store.evict_surface(WS, "1:a");
        assert_eq!(outcome.workspace_state_before, AgentState::NeedsInput);
        assert_eq!(outcome.workspace_state_after, AgentState::Idle);
        assert!(!store.workspace_needs_attention(WS));
    }

    #[test]
    fn evict_pane_removes_all_its_tabs_and_only_its_tabs() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        store.apply_event(WS, "1:b", None, AgentEvent::Activity, 0);
        store.apply_event(WS, "12:c", None, AgentEvent::Activity, 0);
        store.evict_pane(WS, 1);
        assert!(store.surface_record(WS, "1:a").is_none());
        assert!(store.surface_record(WS, "1:b").is_none());
        // Pane 12 must survive eviction of pane 1 (prefix must not over-match).
        assert!(store.surface_record(WS, "12:c").is_some());
        assert_eq!(store.workspace_state(WS), AgentState::Running);
    }

    #[test]
    fn evict_last_surface_returns_workspace_to_unknown() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 0);
        store.evict_surface(WS, "1:a");
        assert_eq!(store.workspace_state(WS), AgentState::Unknown);
        assert_eq!(store.workspace_ids().count(), 0);
    }

    #[test]
    fn evict_workspace_drops_all_state() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        store.evict_workspace(WS);
        assert_eq!(store.workspace_state(WS), AgentState::Unknown);
        assert!(!store.workspace_needs_attention(WS));
    }

    #[test]
    fn stale_running_decays_to_unknown_with_injected_clock() {
        let mut store = AgentStateStore::with_stale_after(1_000);
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 0);
        assert_eq!(store.decay_stale(999), Vec::<String>::new());
        assert_eq!(store.workspace_state(WS), AgentState::Running);
        assert_eq!(store.decay_stale(1_000), vec![WS.to_string()]);
        assert_eq!(store.workspace_state(WS), AgentState::Unknown);
    }

    #[test]
    fn fresh_event_resets_decay_window() {
        let mut store = AgentStateStore::with_stale_after(1_000);
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 0);
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 900);
        assert_eq!(store.decay_stale(1_500), Vec::<String>::new());
        assert_eq!(store.workspace_state(WS), AgentState::Running);
    }

    #[test]
    fn idle_and_needs_input_do_not_decay() {
        let mut store = AgentStateStore::with_stale_after(1_000);
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        store.apply_event(WS, "2:b", None, AgentEvent::Stop, 0);
        assert_eq!(store.decay_stale(1_000_000), Vec::<String>::new());
        assert_eq!(store.workspace_state(WS), AgentState::NeedsInput);
    }

    #[test]
    fn decay_only_reports_workspaces_whose_aggregate_changed() {
        let mut store = AgentStateStore::with_stale_after(1_000);
        // ws-a: running goes stale but a needs-input surface keeps the
        // aggregate pinned — no report. ws-b: aggregate flips — reported.
        store.apply_event("ws-a", "1:a", None, AgentEvent::Activity, 0);
        store.apply_event("ws-a", "2:b", None, AgentEvent::NeedsInput, 0);
        store.apply_event("ws-b", "1:a", None, AgentEvent::Activity, 0);
        assert_eq!(store.decay_stale(2_000), vec!["ws-b".to_string()]);
        assert_eq!(store.workspace_state("ws-a"), AgentState::NeedsInput);
    }

    #[test]
    fn acknowledge_clears_urgency_but_not_state() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        assert!(store.workspace_needs_attention(WS));
        assert!(store.acknowledge_workspace(WS));
        assert!(!store.workspace_needs_attention(WS));
        // State is unchanged — socket-reported state and sidebar never disagree.
        assert_eq!(store.workspace_state(WS), AgentState::NeedsInput);
        // Second acknowledge is a no-op.
        assert!(!store.acknowledge_workspace(WS));
    }

    #[test]
    fn new_needs_input_event_rearms_urgency_after_acknowledge() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 0);
        store.acknowledge_workspace(WS);
        store.apply_event(WS, "1:a", None, AgentEvent::NeedsInput, 1);
        assert!(store.workspace_needs_attention(WS));
    }

    #[test]
    fn non_needs_input_events_do_not_flag_attention() {
        let mut store = store();
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 0);
        store.apply_event(WS, "2:b", None, AgentEvent::Stop, 0);
        assert!(!store.workspace_needs_attention(WS));
    }

    #[test]
    fn kind_is_recorded_and_preserved_when_event_omits_it() {
        let mut store = store();
        store.apply_event(WS, "1:a", Some(AgentKind::Codex), AgentEvent::Activity, 0);
        store.apply_event(WS, "1:a", None, AgentEvent::Stop, 1);
        assert_eq!(
            store.surface_record(WS, "1:a").unwrap().kind,
            Some(AgentKind::Codex)
        );
        store.apply_event(WS, "1:a", Some(AgentKind::Hermes), AgentEvent::Activity, 2);
        assert_eq!(
            store.surface_record(WS, "1:a").unwrap().kind,
            Some(AgentKind::Hermes)
        );
    }

    #[test]
    fn surfaces_listing_is_sorted_and_scoped_to_workspace() {
        let mut store = store();
        store.apply_event(WS, "2:b", None, AgentEvent::Stop, 5);
        store.apply_event(WS, "1:a", None, AgentEvent::Activity, 3);
        store.apply_event("other", "9:z", None, AgentEvent::Activity, 0);
        let listed: Vec<&str> = store.surfaces(WS).map(|(id, _)| id).collect();
        assert_eq!(listed, vec!["1:a", "2:b"]);
        let ids: Vec<&str> = store.workspace_ids().collect();
        assert_eq!(ids, vec!["other", WS]);
    }

    #[test]
    fn hook_event_vocabulary_classifies_like_the_cli_shims() {
        // running-class
        for event in [
            "SessionStart",
            "session_start",
            "UserPromptSubmit",
            "prompt-submit",
            "pre_llm_call",
            "PreToolUse",
            "post_tool_call",
        ] {
            assert_eq!(
                AgentEvent::from_hook_event(event),
                Some(AgentEvent::Activity),
                "{event}"
            );
        }
        // needs-input-class
        for event in ["Notification", "notification", "pre_approval_request"] {
            assert_eq!(
                AgentEvent::from_hook_event(event),
                Some(AgentEvent::NeedsInput),
                "{event}"
            );
        }
        // idle-class
        for event in [
            "Stop",
            "stop",
            "SubagentStop",
            "post_llm_call",
            "SessionEnd",
        ] {
            assert_eq!(
                AgentEvent::from_hook_event(event),
                Some(AgentEvent::Stop),
                "{event}"
            );
        }
        // unrecognized → no transition, never guessed
        assert_eq!(AgentEvent::from_hook_event("SomethingElse"), None);
        assert_eq!(AgentEvent::from_hook_event(""), None);
    }

    #[test]
    fn agent_kind_names_round_trip() {
        for (name, kind) in [
            ("claude", AgentKind::Claude),
            ("codex", AgentKind::Codex),
            ("opencode", AgentKind::OpenCode),
            ("gemini", AgentKind::Gemini),
            ("hermes", AgentKind::Hermes),
        ] {
            assert_eq!(AgentKind::from_name(name), Some(kind));
            assert_eq!(kind.as_str(), name);
        }
        assert_eq!(AgentKind::from_name("Claude-Code"), Some(AgentKind::Claude));
        assert_eq!(AgentKind::from_name("not-an-agent"), None);
    }

    #[test]
    fn state_names_are_stable_for_socket_fields() {
        assert_eq!(AgentState::Unknown.as_str(), "unknown");
        assert_eq!(AgentState::Running.as_str(), "running");
        assert_eq!(AgentState::NeedsInput.as_str(), "needs-input");
        assert_eq!(AgentState::Idle.as_str(), "idle");
    }
}
