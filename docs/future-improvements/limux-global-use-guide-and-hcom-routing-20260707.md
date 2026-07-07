# Limux Use Guide And hcom Routing Brief

Date: 2026-07-07
Status: In-repo staged skill and docs update
Source branch: `lifo/global-use-guide-routing-20260707`
Public classification: mechanics-only

## Purpose

Limux has enough operator-facing and agent-facing control surface that its
day-to-day workflow should be documented in the repo rather than left in
terminal memory. This brief records the in-repo staging shape for a reusable
Limux use guide and the public-safe Limux/hcom boundary.

Global Codex/Claude promotion is a separate owner-gated global-config task.
Promoted global skill mirrors should point back to this repo as the canonical
command/source reference.

## Canonical Source Map

Use these repo sources for Limux command and workflow behavior:

- `README.md` - user-facing install, diagnostics, runtime channels, agent
  integrations, shortcuts, and architecture.
- `AGENTS.md` - contributor guide, crate map, CLI/runtime caveats, bridge
  status, verification notes.
- `skills/limux-use-guide/` - staged global-skill candidate.
- `docs/cmux-parity-plan.md` - upstream cmux parity policy and idea feed.
- `docs/verification/post-install-checklist-v1.md` - operator smoke checklist.
- `docs/verification/run-template.md` - verification run record template.
- `docs/verification/wave1-morning-summary-20260707.md` - merged Wave 1 status.
- `docs/future-improvements/limux-runtime-channel-contract-20260702.md` -
  stable/preview runtime channel contract.
- `docs/future-improvements/limux-dual-runtime-runbook-20260702.md` -
  operator workflow for side-by-side stable and preview Limux runtimes.
- `rust/limux-cli/src/main.rs` - CLI argument handling and help text.
- `rust/limux-host-linux/src/control_registry.rs` - live bridge method
  registry.
- `rust/limux-host-linux/src/state_mirror.rs` - current read-only mirror
  fallthrough.

Do not copy private fleet policy, local operator paths, runtime IDs, secrets, or
manager rosters into public Limux documentation.

## Command Surface To Cover

Global flags and launch model:

- `limux` with no args launches the GTK app through the installed CLI.
- `limux --version` reports CLI version and build identity.
- `limux --socket <path>` targets an explicit control socket.
- `limux --channel stable|preview[:id]` targets isolated runtime channels.
- `limux --json` and `--id-format refs|both|uuids` are automation helpers.
- `limux target-info` / `limux socket-info` prints the resolved socket/channel
  without connecting.
- `limux doctor [--json] [--log-triage --lines <n>]` is the first diagnostic
  for stale launchers, stale resources, socket drift, and build mismatch.

Workspace and pane operations:

- `limux identify --json`
- `limux list-workspaces`
- `limux list-panels --workspace <id|ref>`
- `limux list-panes --workspace <id|ref>`
- `limux surface-health --workspace <id|ref>`
- `limux new-workspace --cwd <path> [--command <text>]`
- `limux close-workspace --workspace <id|ref>`
- `limux sidebar-state --workspace <id|ref>`
- `limux new-surface --workspace <id|ref>`
- `limux new-pane --direction <left|right|up|down> --command <text>`
- `limux rename-workspace`, `limux rename-window`, `limux rename-tab`
- `limux tab-action --action <name>`
- `limux pane-action --action set_flag_color --color
  <orange|red|purple|pink|green|yellow|teal|cyan>`
- `limux pane-action --action clear_flag_color`

Terminal text and observation:

- `limux send --surface <surface-ref> <text>`
- `limux send-key --surface <surface-ref> <key>`
- `limux read-screen --surface <surface-ref> [--scrollback] [--lines <n>]`
- `limux capture-pane` as an alias of `read-screen`.
- Text send paths intentionally reject terminal control characters except tab,
  LF, and CR. Use `send-key` for intentional control keys.
- Keep `limux send` payloads short. For long prompts, write a file and send a
  path.

Agent and review operations:

- `limux notify --subtitle <text> --body <text> <title>`
- `limux hooks setup [agent]`
- `limux hooks uninstall [agent]`
- `limux hooks <agent> <event>`
- `limux claude-hook`, `limux opencode-hook`, `limux gemini-hook`
- `limux hermes-hook` / `limux hooks hermes <event>` as receiver paths only;
  Hermes-side lifecycle plugin installation remains external.
- `limux agent-team --agents codex,claude[,hermes,opencode,gemini] --cwd "$PWD"`
- `limux agent-team --launch-mode hcom` launches peers as
  `hcom <agent> --run-here` inside Limux panes.
- `limux agent-team --no-bootstrap`, `--no-launch`, and `--dry-run` are
  important safety/control flags.
- `limux review prepare --artifact <path-or-ref> --reviewer <agent|manual>
  --lens <security|correctness|maintainability|ux|release> --summary <text>`
- `limux review spawn --review-id <id> [--launch-mode direct|hcom]`

## Limux And hcom Boundary

- Limux socket: local GUI control bus for panes, workspaces, terminal text,
  read-screen, notifications, and runtime diagnostics.
- hcom: cross-agent/session messaging bus for named agents, durable messages,
  transcripts, resume/fork, and cross-project coordination.
- `limux notify` is for user-visible Limux attention such as toasts/sidebar
  badges.
- `hcom send` is for agent-to-agent messages and should not be described as a
  pane notification by itself.
- Agents launched through Limux inherit `LIMUX_WORKSPACE_ID`,
  `LIMUX_SURFACE_ID`, `LIMUX_PANE_ID`, `LIMUX_TAB_ID`, and `LIMUX_SOCKET`.
  hcom workers launched inside Limux should preserve or reference those values
  when they need to call back into the GUI control bus.

## Runtime Isolation

Use user-local stable/preview lanes for testing Limux changes without replacing
the daily-driver runtime:

- `--channel legacy` creates `limux` / `limux-cli`.
- `--channel stable` creates `limux-stable` / `limux-stable-cli`.
- `--channel preview` creates `limux-preview` / `limux-preview-cli`.
- `--channel preview:<id>` creates named preview launchers.

The installer writes `install-info.json` beside the installed executable.
Verification should include `--version`, `target-info`, `doctor --json`,
Ghostty resource validation, and the checklist/run-template workflow in
`docs/verification/`.

## Current Bridge Caveats

PRD-E live bridge parity is partial:

- The GTK bridge has a native method registry for current live methods.
- Read-only state-mirror fallthrough currently includes `window.list` and
  `window.current`.
- Remaining read-only parity, mutation routing, registry expansion, browser
  pane behavior, and kill-switch behavior remain open work until the relevant
  PRD tasks are closed.

PRD-F browser-pane architecture should not be presented as ratified until its
measurement/evidence task is complete.

PRD-G hook/sidebar lifecycle work is in progress. It is valid to document the
intended hook-to-sidebar lifecycle state flow, but not to claim complete parity
for every agent family until the task closes.

## Acceptance Criteria

- `README.md` and `AGENTS.md` cover the verified CLI/runtime surface.
- `skills/limux-use-guide/` contains a public-safe staged skill and README.
- The guide clearly separates Limux local workspace control from hcom
  cross-session/project routing.
- Stable/preview runtime isolation, Ghostty resource validation, and
  verification checklist workflow are documented.
- No secrets, tokens, private fleet roster details, local operator-only policy,
  or machine-specific runtime state are added to the public Limux source.
