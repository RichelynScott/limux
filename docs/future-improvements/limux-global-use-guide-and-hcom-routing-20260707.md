# Limux Global Use Guide And hcom Routing Brief

Date: 2026-07-07
Status: Routing request
Owner lane: Limux -> CODEX_CLAUDE_CODE global-config managers + hcom manager
Source repo: `/home/riche/MCPs/limux`
Source branch: `lifo/global-use-guide-routing-20260707`
Source base: `origin/main` at `0b69e75`
Requires-Handshake: yes

## Purpose

Limux now has enough operator-facing and agent-facing control surface that it
should not live only in README snippets or terminal memory. Route this to:

- `niru` for Codex global-config pointers and a Codex-visible global skill.
- `kazu` for Claude-side global-config pointers and any Claude-visible mirror.
- `dino` for hcom integration review and hcom-side command/wiring guidance.

The desired output is a concise global-config pointer plus a dedicated,
comprehensive Limux use-guide skill. The global skill should teach agents how
to use Limux safely as the local workspace/control layer while keeping hcom as
the cross-project/session coordination bus.

## Canonical Source Map

Limux-side source material:

- `README.md` - install, build, agent integrations, shortcuts, architecture.
- `AGENTS.md` - contributor guide, crate map, control bridge caveats.
- `docs/limux-hcom-workflow.md` - current Limux + hcom workflow guide.
- `docs/cmux-parity-plan.md` - upstream cmux parity policy and idea feed.
- `docs/verification/post-install-checklist-v1.md` - operator smoke checklist.
- `docs/verification/run-template.md` - verification run record template.
- `docs/verification/wave1-morning-summary-20260707.md` - merged Wave 1 status.
- `docs/decisions/browser-pane-architecture-20260707.md` - PRD-F browser pane
  architecture skeleton; evidence still pending.
- `docs/future-improvements/limux-runtime-channel-contract-20260702.md` -
  stable/preview runtime channel contract.
- `docs/future-improvements/limux-dual-runtime-runbook-20260702.md` -
  operator workflow for side-by-side stable and preview Limux runtimes.

Global-config side target surfaces should be decided by `niru` and `kazu` in
`/home/riche/Proj/CODEX_CLAUDE_CODE`. Do not copy private fleet policy into the
public Limux repo. The Limux repo should remain the command/source canonical;
global config should carry operator workflow, runtime routing, and skill
promotion.

## Command Surface To Cover

Global flags and launch model:

- `limux` with no args launches the GTK app through the installed CLI.
- `limux --version` reports CLI version and build identity.
- `limux --socket <path>` targets an explicit control socket.
- `limux --channel stable|preview[:id]` targets isolated runtime channels.
- `limux --socket-mode runtime|debug`, `--json`, `--pretty`, and
  `--id-format refs|both|uuids` are available for automation.
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
  --lens <name> --summary <text>`
- `limux review spawn --review-id <id> [--launch-mode direct|hcom]`

Control bridge and external integration methods:

- `system.identify`
- `system.capabilities`
- `workspace.current`, `workspace.list`, `workspace.select`,
  `workspace.create`
- `pane.list`, `pane.surfaces`, `pane.create`
- `surface.list`, `surface.read_text`, `surface.send_text`,
  `surface.send_key`, `surface.health`
- `window.list`, `window.current`, `window.present`
- Cursor restricted socket allowlist currently includes `workspace.list`,
  `workspace.select`, `window.present`, `cursor.pane_create_empty`,
  `surface.read_text`, and `cursor.workspace_open_folder`; it rejects terminal
  write methods such as `surface.send_text`, `surface.send_key`, and aliases.

## hcom Integration Questions For dino

Please review whether hcom docs/skills should include a Limux integration note
for:

- `limux agent-team --launch-mode hcom` as the preferred way to keep hcom
  sessions registered while still placing them inside Limux panes.
- Hermes support: Limux has receiver hooks and hcom launch support, but Hermes
  lifecycle plugin installation remains Hermes/hcom-owned.
- hcom session metadata: whether hcom should expose or preserve Limux
  workspace/surface/pane identifiers when launched through Limux.
- hcom routing: when to use `limux send` for same-workspace messages versus
  `hcom send` for cross-project/session messages.
- hcom notifications: whether hcom events should feed Limux sidebar/workspace
  notifications, and which side owns the adapter.
- restart/runtime isolation: stable vs preview Limux channels should not break
  hcom identity, attribution, or current-working-directory metadata.

## Requested Global Skill Shape

Recommended skill name:

- `limux-use-guide`, `limux-operator-guide`, or `limux-workspace-control`.

Recommended sections:

- What Limux is and when to use it.
- Installed CLI vs host binary distinction.
- Stable/preview runtime channel safety.
- Quick diagnostics with `limux doctor`, `target-info`, and socket/channel
  flags.
- Workspace, pane, tab, and surface command cookbook.
- Safe prompt/message injection patterns.
- `read-screen` / `capture-pane` observation patterns.
- Agent-team bootstrap with direct and hcom launch modes.
- Review prepare/spawn workflow and ledger expectations.
- Limux + hcom routing rules.
- Hermes-specific caveats.
- Cursor/restricted socket integration basics.
- Troubleshooting copied from the post-install checklist:
  resources/GSettings/icons, Ghostty resource packaging, mouse drag copy,
  pane resize, notification borders, and stable/preview isolation.
- Verification commands and expected smoke evidence.
- Pointers back to Limux repo docs rather than duplicating every detail.

## Current PR State At Routing Time

Open Limux PRs: none reported by `gh pr list --state open` on 2026-07-07.

Recently merged Wave 1 items on `origin/main`:

- `bd368c4` - PRD-G agent lifecycle state machine.
- `34babf3` - PRD-H cwd inheritance for splits and `pane.create`.
- `8da585b` - Wave 1 wrap summary.
- `0b69e75` - PRD-F F1 sequencing clarification.

Important caveat: PRD-F currently has a decision skeleton, not completed
evidence. Do not present browser-pane architecture as ratified until the F1
measurement/evidence run is complete.

## Acceptance Criteria

- `niru` and `kazu` have durable global-config pointers and either create or
  task-track a Limux global use-guide skill.
- `dino` reviews the hcom/Limux integration boundary and records hcom-side
  follow-up if needed.
- The resulting global skill points to Limux canonical docs instead of copying
  stale command details.
- The guide clearly separates Limux local workspace control from hcom
  cross-session/project routing.
- The guide covers stable/preview runtime isolation so Limux can be developed
  without breaking the user's primary working runtime.
- No secrets, tokens, private fleet roster details, or local-only policy are
  added to the public Limux source.
