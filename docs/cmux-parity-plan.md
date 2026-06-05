# cmux-parity plan (revised after architectural discovery)

## Architecture discovery

Limux has **two control servers**:

1. **Standalone `limux-control-server` binary** — uses `limux_core::Dispatcher`
   + `ControlState` and supports the **full** command vocabulary. Used for
   tests and for CLI calls when the GUI isn't running.

2. **Embedded bridge inside `limux-host-linux`** — `control_bridge.rs` only
   routes a narrow subset of methods to the GTK main loop. Supports
   `system.ping`, `system.identify`, `workspace.{current,list,create,
   select,rename,close}`, `pane.list`, `pane.surfaces`, `surface.list`,
   `pane.create` for terminal self-spawn, `surface.send_text`,
   `surface.send_key`, `surface.read_text`, `surface.health`, and
   `notification.create`. It still does **NOT** support browser commands.

When the GUI is running, the CLI targets the bridge via the runtime
socket. `list-panes` / `list-panels`, terminal `new-pane --command ...`,
text injection, key-level injection, `surface-health`, and terminal
`read-screen` now work against the running host.

## Delivery strategy (revised)

### Phase 1 — Env auto-wiring ✅ (shipped in 1295d12)

### Phase 2 — Make the bridge a full proxy (🚧 PARTIAL)

Bridge should route unknown methods to a local `Dispatcher` instance
seeded with live GTK state, OR to dedicated per-method `ControlCommand`
variants that interrogate the live state. The cleanest path:

- Maintain a `Arc<Mutex<ControlState>>` owned by the GTK app, kept in
  sync with live workspace/pane/surface state.
- Bridge falls through unknown methods to `Dispatcher::dispatch` on that
  shared state.
- Specific methods that need GTK side-effects (send_text, create_surface,
  notification.create) remain as `ControlCommand` variants.

The terminal introspection path is now bridged directly against live GTK state.
Remaining proxy work is for deferred browser surface commands and broader
dispatcher parity.

**Shipped so far (in 6b8eb1a and follow-up bridge work):**

- `surface.send_text` and `notification.create` now pass `allow_name=true`
  to `parse_optional_workspace_target` for workspace-name targets. The current
  generated team protocol uses surface IDs because `agent-team` splits peers
  inside one workspace.
- `pane.list`, `pane.surfaces`, and `surface.list` now route on the live
  GTK bridge, so agents can discover peer panes/surfaces in a running
  Limux window.
- `surface.send_key` now routes to the exact terminal surface when provided,
  so agents can send deterministic key-level control such as Ctrl-C.
- `surface.health` and `surface.read_text` now route on the live GTK bridge,
  so agents can inspect peer terminal health and visible screen text.
- `pane.create` now routes through the GTK bridge for terminal panes. From
  inside an agent terminal, `limux new-pane --direction right --command 'claude'`
  uses `LIMUX_WORKSPACE_ID`, `LIMUX_SURFACE_ID`, and `LIMUX_PANE_ID` to split
  the caller's pane, create a new terminal, and launch the command there.

**Still open (priority order):**

- Browser command bridge parity.

### Phase 3 — `limux notify` + GUI toast/sidebar integration ✅
`ControlCommand::CreateNotification` wired through the bridge into
`mark_workspace_unread_with_message` + libadwaita toast.
CLI: `limux notify [--workspace <id|name>] [--subtitle <…>] [--body <…>] <title>`.

### Phase 4 — `limux claude-hook` / `opencode-hook` / `gemini-hook` ✅
Reads hook JSON from stdin, translates the agent-specific event vocabulary
into a `notify` (and, where useful, an inline `send`). Drop-in for
`~/.claude/settings.json` hooks blocks.

### Phase 5 — `limux agent-team` + generated protocol file ✅
`limux agent-team [--agents codex,claude[,opencode,gemini]] [--cwd <path>]
[--protocol-path <path>] [--roster-path <path>] [--ledger-path <path>]
[--force-protocol-overwrite] [--force-roster-overwrite] [--no-launch]
[--no-bootstrap] [--dry-run]`:

- Splits the active workspace into one terminal pane per agent and launches
  each agent CLI unless `--no-launch` is set.
- After the generated protocol file is written, sends each launched peer a
  short bootstrap prompt that tells it to read `LIMUX_AGENTS.md` and the
  authoritative instruction sources listed there. Use `--no-bootstrap` to
  launch panes without that post-launch prompt.
- Bridge passes `allow_name=true` to `parse_optional_workspace_target` for
  `surface.send_text` and `notification.create`; the generated team protocol
  still addresses peers by surface ID because agents share one workspace.
- Writes `LIMUX_AGENTS.md` in the shared cwd by default, or the explicit
  `--protocol-path`, documenting:
    - the generated-file marker,
    - detected instruction sources (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`)
      with path, modified time, and deterministic content hash metadata,
    - the peers table (agent → pane → surface → launch cmd),
    - the `<agent-msg from="…" to="…" id="…" reply-to="…" ts="…">` envelope,
    - the exact `limux send` invocation for sending and replying,
    - the `limux notify` escalation path for human input,
    - the `LIMUX_*` env contract every spawned terminal inherits,
    - the optional `LIMUX_AGENTS.local.md` durable policy sidecar,
    - the durable `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md`
      coordination files,
    - editable Policies section (timeouts, size limits, destructive-action gating).
  Existing repo `AGENTS.md` files are not written by default. Existing unmarked
  protocol files are not overwritten unless `--force-protocol-overwrite` is
  explicitly passed. Symlink protocol paths are refused.
- Seeds `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` when missing, or
  the explicit `--roster-path` / `--ledger-path` targets. Existing roster and
  ledger files are preserved by default. `--force-roster-overwrite` replaces
  only marked Limux roster files; the review ledger remains create-if-missing
  only. Symlink, non-regular, and overlapping output paths are refused.
- `--dry-run` does not contact a running host, but still writes the generated
  protocol and seeds missing roster/ledger files. Use temporary explicit output
  paths for preview-only runs.

**Shipped in `cec067f`:**

- Default protocol output changed from `AGENTS.md` to `LIMUX_AGENTS.md`.
- `--protocol-path <path>` allows an explicit protocol file location.
- Regression coverage preserves existing repo `AGENTS.md` files and verifies
  the default sidecar path.

**Shipped after `cec067f`: Phase 5A — zero-friction protocol discovery**

- Added a generated-file marker to `LIMUX_AGENTS.md`.
- Added an `Instruction Sources` section that detects repo instruction files
  such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` and points agents to read
  them directly.
- Does not copy, merge, or reinterpret repo instruction files by default. Repo
  instruction files remain authoritative; the Limux sidecar only adds runtime
  topology and messaging protocol.
- Added no-overwrite semantics for existing unmarked `LIMUX_AGENTS.md`, with
  `--force-protocol-overwrite` if replacement is required.
- Documented a durable local extension point,
  `LIMUX_AGENTS.local.md` for team-specific policy that survives regeneration.

**Shipped after Phase 5A: GTK `surface.send_text` failure reporting**

- The live GTK bridge now checks `TerminalHandle::send_text` and returns a
  conflict error when a resolved terminal surface is not ready for text input
  instead of reporting `ok: true`.
- The existing `pane.create --command` path already retries until the new
  pane's terminal surface is writable before returning success.

**Shipped before Phase 5B: typed-PTY control-character guard**

- Text typed into terminal panes through `surface.send_text`, `paste-buffer`,
  `respawn-pane`, `pane.create --command`, and `workspace.create --command`
  now rejects terminal control characters other than tab, LF, and CR.
- The guard is enforced in the CLI for fast feedback and again in the live GTK
  bridge / standalone core dispatcher for direct socket callers.
- `surface.send_key` remains the explicit route for control keys such as
  Ctrl-C, and OSC/output parsing remains separate from typed input.
- Accepted/deferred display-spoofing risks for the Phase 5B threat model:
  bare CR remains allowed, and Unicode format / zero-width characters are not
  caught by the control-character guard. Revisit those before untrusted
  generated prompt text flows through automatic bootstrap.

**Shipped after typed-PTY guard: Phase 5B — automatic bootstrap**

- `agent-team` now keeps `pane.create.command` to the bare launcher binary
  (`codex`, `claude`, etc.) and does not embed prompt text in launch shell
  commands.
- Generated bootstrap prompts are single-line text with escaped dynamic values
  and no CR, tab, LF, bidi formatting, or zero-width display-spoofing
  characters.
- Bootstrap starts only after `LIMUX_AGENTS.md` is written. The prompt is sent
  through `surface.send_text`, then submitted with `surface.send_key enter`,
  matching the documented manual `limux send` + `limux send-key enter` pattern.
- The live GTK host now submits `pane.create --command` by typing the validated
  command text and sending an explicit Enter key, which avoids bracketed-paste
  shells leaving launch commands sitting at the prompt.
- The Xvfb smoke harness shadows `codex` and `claude` with fake binaries and
  verifies both fake agents receive the post-write bootstrap prompt with
  `LIMUX_*` env and zero extra argv.

**Shipped after Phase 5B: Phase 5C — durable roster and review ledger**

- `agent-team` now seeds `LIMUX_TEAM_ROSTER.md` with project, agent, owner,
  hcom, related-team, routing, privacy, and durable-file placeholders. Live
  workspace/pane/surface IDs stay in the regenerated `LIMUX_AGENTS.md` runtime
  protocol so the durable roster does not become stale routing data.
- `agent-team` now seeds `LIMUX_REVIEW_LEDGER.md` with an append-oriented entry
  template for reviewer findings, consensus decisions, accepted risks, and
  cross-team notifications.
- Generated `LIMUX_AGENTS.md` points peers to both durable files, and bootstrap
  prompts tell launched peers to read the roster and ledger before starting.
- Existing roster and ledger files are preserved by default. The roster has an
  explicit `--force-roster-overwrite` reset path for marked Limux roster files;
  the ledger is never overwritten by `agent-team`.
- CLI tests cover dry-run creation, existing roster/ledger preservation, forced
  roster replacement, unmarked roster force refusal, symlink refusal,
  overlapping output path refusal, and live bootstrap ordering. The Xvfb smoke
  harness now proves fake agents see protocol, roster, and ledger files before
  receiving the bootstrap prompt.

**Shipped after Phase 5C: Phase 5D1 — reviewer workflow scaffold**

- Added `limux review prepare --artifact <path-or-ref> --reviewer
  <codex|claude|gemini|opencode|manual> --lens
  <security|correctness|maintainability|ux|release> --summary <text>`.
- The scaffold creates a durable `reviews/<review-id>.md` request file,
  appends a pending entry to `LIMUX_REVIEW_LEDGER.md`, and prints the exact
  reviewer prompt without contacting the host or launching reviewer panes.
- `--dry-run` plans the request, ledger entry, and prompt without writing
  files. `--review-id`, `--reviews-dir`, and `--ledger-path` allow deterministic
  paths for coordinated reviews.
- Existing request files, leaf symlink review directories, leaf symlink
  ledgers, non-regular targets, overlapping request/ledger paths, and control
  characters in generated prompt fields are refused. Use trusted output
  directories; parent path components are not recursively audited for symlinks.
- CLI tests cover request creation, append-only ledger behavior, dry-run,
  existing request refusal, symlink refusal, non-regular ledger refusal, invalid
  choices, overlapping request/ledger paths, dispatch, missing required
  arguments, and control-character rejection.

**Shipped after Phase 5D1: Phase 5D2 — reviewer spawn/evidence pointer**

- Added `limux review spawn --review-id <id>` to continue from an existing
  generated `review prepare` request.
- `review spawn` reads the request file, refuses `manual` reviewers, creates
  one reviewer terminal pane through the live `pane.create` path, sends the
  prepared prompt through `surface.send_text` plus explicit Enter, writes a
  `reviews/<review-id>.evidence.md` pointer file, and updates the matching
  pending ledger entry to `in-progress`.
- `--dry-run` validates the request, ledger, evidence path, reviewer, and
  direction without host contact. `--no-launch` creates a pane without typing
  the reviewer command or prompt, matching the existing launch-safety pattern.
- The evidence file records the request path, reviewer pane/surface, prompt
  status, and suggested `limux read-screen --surface ... --scrollback --lines
  120` capture command. It intentionally does not dump raw terminal scrollback.
- CLI tests cover dry-run host avoidance, live fake-socket pane creation,
  prompt send/Enter submission, evidence pointer creation, and targeted ledger
  update while preserving unrelated ledger content.

### Phase 6 — (deferred) `limux progress`, `limux log`, `limux markdown`
Nice polish, not blockers.

## Why phase 2 first

Without a real bridge, every subsequent feature ends up routing around
the same hole: the GUI owns the ground truth about surfaces/panes but
the CLI can't query it. Fixing this once, properly, makes phases 3–5
small.
