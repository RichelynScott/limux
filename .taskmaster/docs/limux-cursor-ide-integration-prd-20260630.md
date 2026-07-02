# PRD: Limux Cursor IDE Integration

Date: 2026-06-30
Owner lane: lifo / Limux
Status: final planning PRD for TaskMaster parsing; implementation remains gated
Source plan: `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`
Review evidence: `docs/reviews/limux-cursor-ide-integration-20260630/`

## 1. Introduction

Limux is the operator's primary terminal workspace manager for Codex, Claude,
Hermes, and shell sessions. The operator wants to keep Limux as the owner of
terminal panes and agent runtime state while regaining Cursor's file explorer
and editor workflow.

This PRD defines a v1 Cursor/VS Code-compatible extension plus narrow Limux host
support. Cursor becomes a file/editor cockpit and read-only Limux navigator.
Limux remains the authority for Ghostty surfaces, panes, workspaces, PTYs, and
agent sessions.

The v1 implementation must not become a generic terminal-control bridge. The
current Limux socket authorizes same-user peers broadly, so state-changing
Cursor commands require a server-side restricted method surface before shipping.

## 2. Goals

- Let the operator keep Cursor's file explorer visible while Limux owns terminal
  panes and agent sessions.
- Provide a Cursor side view that lists Limux workspaces, panes, tabs, and
  surfaces from the selected Limux runtime.
- Allow safe workspace focus/presentation and empty-pane creation from Cursor.
- Allow opening a known Limux workspace folder in Cursor and opening the current
  Cursor folder in Limux without copy/pasting paths.
- Add read-only visible viewport snapshots for selected Limux surfaces.
- Keep v1 free of host package-manager execution during normal development and
  testing.
- Make all safety claims executable through Rust tests, Node tests, Xvfb smoke,
  and a v1 acceptance script or equivalent check.

## 3. Non-Goals

- No GTK embedding inside Cursor.
- No replacement of Ghostty rendering with a webview terminal.
- No `surface.send_text`, `surface.send_key`, CLI `send`, raw PTY write, paste,
  keyboard-input, or arbitrary command injection from Cursor in v1.
- No generic raw JSON request console or arbitrary Limux method command from
  Cursor.
- No `pane.create.command`, `workspace.create.command`, or command-bearing
  pane/workspace creation from the Cursor path.
- No native Windows-host Cursor direct transport to WSL Unix sockets in v1.
  Supported v1 topologies are native Linux Cursor and Windows Cursor with the
  Remote-WSL extension host running inside the target distro.
- No true tmux-like attach mode in v1. Attach mode requires a separate PRD.
- No TaskMaster MCP registration, package-resolved extension tooling, `npx`,
  `npm exec`, `vsce`, `yo code`, or package scripts for v1.

## 4. User Stories

### US-001: Discover And Select A Limux Runtime

Description: As an operator, I want Cursor to connect to the intended Limux
runtime so that commands do not target a stale or wrong Limux process.

Acceptance criteria:

- [ ] The extension resolves the socket using the same order as Limux source:
      configured `limux.socketPath`, `LIMUX_SOCKET`, `LIMUX_SOCKET_PATH`,
      `LIMUX_CHANNEL` channel-derived sockets, `${XDG_RUNTIME_DIR}/limux/limux.sock`,
      then existing `/tmp/limux.sock`.
- [ ] Channel-derived socket resolution matches `RuntimeChannel::from_env()`:
      `stable` maps to `${XDG_RUNTIME_DIR}/limux/stable/limux.sock`,
      `preview` maps to `${XDG_RUNTIME_DIR}/limux/preview/default/limux.sock`
      unless `LIMUX_PREVIEW_ID` is set, and `preview:<id>` or `preview/<id>`
      maps to `${XDG_RUNTIME_DIR}/limux/preview/<id>/limux.sock`.
- [ ] If `XDG_RUNTIME_DIR` is unavailable, channel-derived sockets use the
      current Limux compatibility names under `/tmp`: `limux-stable.sock` and
      `limux-preview-<id>.sock`.
- [ ] The extension does not add `/run/user/${uid}/limux/limux.sock` as a
      separate fallback unless Limux source adds it first.
- [ ] Every candidate socket is probed with a timeout and `system.identify`.
- [ ] Stale socket files, refused connections, and timeout paths surface a clear
      disconnected state and do not silently retry forever.
- [ ] Multiple live runtimes produce a user choice with runtime identity shown.
- [ ] State-changing commands pin the selected runtime identity and fail if the
      identity changes.
- [ ] Node tests cover live, stale, duplicate, and ambiguous socket candidates.

### US-002: View Limux Workspaces In Cursor

Description: As an operator, I want a Cursor side view of Limux workspaces,
panes, tabs, and surfaces so that I can navigate Limux from the same IDE where I
edit files.

Acceptance criteria:

- [ ] The extension contributes a Limux activity/sidebar view and lazy
      activation event.
- [ ] The tree uses `vscode.TreeDataProvider`, stable item IDs, context values,
      and `onDidChangeTreeData` refresh.
- [ ] The tree shape is `Workspace -> Pane -> Tab / Surface`.
- [ ] The extension uses Node `net.connect` to the Unix socket with existing
      Limux framing; it does not shell out for normal refresh.
- [ ] Refresh failures leave the tree in a clear disconnected state.
- [ ] A manual smoke with `cursor --extensionDevelopmentPath
      integrations/cursor-limux` can display the tree against a running Limux.

### US-003: Enforce A Cursor-Restricted Server Surface

Description: As a maintainer, I need Limux to enforce the restricted Cursor
method set server-side so that extension bugs cannot bypass the v1 safety
boundary.

Acceptance criteria:

- [ ] Cursor state-changing commands use a server-enforced restricted method
      set through either a dedicated restricted socket or a per-connection
      caller role/nonce plus allowlist.
- [ ] The restricted method set is exactly pinned and shared by Rust and
      extension request builders.
- [ ] `system.capabilities` reflects the restricted method set for Cursor
      connections.
- [ ] Rust tests prove the Cursor-restricted path rejects `surface.send_text`,
      `surface.send_key`, raw `pane.create.command`, unknown methods, and
      unexpected payload fields.
- [ ] The unrestricted existing socket behavior remains compatible for existing
      Limux CLI/agent use unless the operator separately approves a breaking
      security hardening change.

### US-004: Focus And Present A Limux Workspace

Description: As an operator, I want to select a Limux workspace from Cursor and
bring the correct Limux workspace forward when the desktop allows it.

Acceptance criteria:

- [ ] `workspace.select` remains selection-only.
- [ ] `window.present` is the separate best-effort raise/focus method.
- [ ] `window.present` returns `succeeded`, `failed`, or `unsupported`.
- [ ] Wayland limitations are documented and surfaced to the user instead of
      silently failing.
- [ ] Rust and extension tests cover success, failure, and unsupported states.
- [ ] Xvfb smoke covers the available presentation behavior; a real Wayland
      manual smoke is required before claiming Wayland focus support.

### US-005: Create An Empty Limux Pane From Cursor

Description: As an operator, I want to create an empty Limux terminal pane from
Cursor so that I can set up workspace layout without injecting commands.

Acceptance criteria:

- [ ] The pinned method is `cursor.pane_create_empty`.
- [ ] No `command`, text, key, paste, shell, or raw PTY payload is accepted.
- [ ] Rust rejects any command field or unexpected payload field on this method.
- [ ] Extension request builders cannot emit command-bearing pane creation.
- [ ] Tests submit hand-crafted forbidden JSON frames and prove rejection.

### US-006: Open Workspace Folders Across Limux And Cursor

Description: As an operator, I want Limux and Cursor to open known workspace
folders in each other so that I do not need to copy/paste filesystem paths.

Acceptance criteria:

- [ ] `workspace.list` exposes `folder_path`, `openable_path`, and `path_source`
      only after safe path determination.
- [ ] Safe path determination requires canonicalized, existing local
      directories and configured workspace-root or current-workspace policy.
- [ ] The Limux-to-Cursor launcher requires a configured absolute executable
      path such as `limux.cursorExecutable` or `LIMUX_CURSOR_BIN`; PATH lookup is
      not the default.
- [ ] Launcher argv construction never uses shell interpolation.
- [ ] Tests cover spaces, leading dashes, shell metacharacters, missing dirs,
      files, broken symlinks, and symlink escapes.
- [ ] Cursor-to-Limux folder opening accepts only local `file:` workspace
      folders in the same Linux/Remote-WSL extension-host environment.
- [ ] Native Windows paths, virtual folders, remote folders, untitled folders,
      and ambiguous multi-root cases are rejected or quick-picked explicitly.

### US-007: Read Visible Surface Snapshots

Description: As an operator, I want a read-only snapshot of a Limux surface in
Cursor so that I can inspect what a pane is showing without attaching to the
live PTY.

Acceptance criteria:

- [ ] The pinned method is `surface.read_text`.
- [ ] The v1 contract returns only the current visible viewport.
- [ ] The response has bounded size, explicit encoding, and no scrollback
      semantics.
- [ ] Requests with `count`, `offset`, scrollback, paste, key, or text payload
      fields are rejected.
- [ ] Tests cover bounded output and forbidden parameter rejection.

### US-008: Ship Executable Acceptance Gates

Description: As a maintainer, I want executable checks for the v1 acceptance
claims so that future changes cannot accidentally reopen the trust boundary.

Acceptance criteria:

- [ ] Extension source tests run with built-in `node --test` and no npm/npx.
- [ ] Extension tests cover socket resolution, framing, method allowlist,
      TreeDataProvider behavior, local-folder eligibility, and activation smoke
      helpers.
- [ ] Rust tests cover restricted method dispatch, launcher argv/path safety,
      workspace metadata, present states, and visible snapshot bounds.
- [ ] `./scripts/check.sh` remains green.
- [ ] `./scripts/xvfb-smoke-test.sh` or a documented sibling smoke covers host
      integration where practical.
- [ ] A v1 acceptance script or equivalent check verifies tree parity with
      `limux --json list-workspaces`, no forbidden Cursor methods, folder
      round-trip, and stale/duplicate socket behavior.

### US-009: Keep V2 Attach Mode Separate

Description: As a maintainer, I want attach mode tracked separately so that v1
does not accidentally absorb risky terminal ownership, resize, copy/paste, and
input work.

Acceptance criteria:

- [ ] The v1 implementation does not add attach, input forwarding, raw PTY
      writes, bidirectional resize authority, or stream replay.
- [ ] The v2 section remains marked `REWORK BEFORE PRD`.
- [ ] A future v2 PRD must define attach lifecycle, output framing, heartbeat,
      detach cleanup, host shutdown, replay/resync, backpressure, resize
      authority, input taxonomy, and multi-client behavior before code starts.

## 5. Functional Requirements

- FR-001: Add a Cursor/VS Code-compatible extension at
  `integrations/cursor-limux/` using only Node built-ins and the extension API.
- FR-002: Extension package metadata must have no dependencies,
  devDependencies, or package-manager-backed scripts for v1.
- FR-003: Extension activation must be lazy and tied to the Limux view.
- FR-004: Extension socket resolution must mirror Limux source exactly.
- FR-005: The extension must use Node Unix socket I/O and Limux framing, not
  shell command wrappers, for normal runtime communication.
- FR-006: Runtime identity must be displayed and pinned before state-changing
  calls.
- FR-007: Add or adapt a server-side Cursor-restricted method surface before
  shipping any Cursor state-changing command.
- FR-008: Pinned v1 methods are `workspace.list`, `workspace.select`,
  `window.present`, `cursor.pane_create_empty`, `surface.read_text`, and
  `cursor.workspace_open_folder`.
- FR-009: The Cursor path must not expose or register terminal text/key/paste
  injection methods.
- FR-010: `workspace.list` must expose openable path metadata with explicit
  source and safety status.
- FR-011: Limux-to-Cursor launcher must require a configured absolute
  executable path and safe argv construction.
- FR-012: Cursor-to-Limux folder opening must accept only local Linux/Remote-WSL
  `file:` folders.
- FR-013: Visible snapshots must be read-only, bounded, and viewport-only.
- FR-014: Wayland presentation limitations must be visible to users and tests.
- FR-015: v1 acceptance must be covered by executable tests and smoke checks.

## 6. Technical Considerations

- Existing source evidence:
  - `rust/limux-control/src/socket_path.rs` currently resolves explicit/env
    paths, channel-derived stable/preview sockets from `LIMUX_CHANNEL`, then
    `${XDG_RUNTIME_DIR}/limux/limux.sock` or `/tmp/limux.sock`.
  - `rust/limux-control/src/auth.rs` defaults to same-user `LocalUser`.
  - `rust/limux-host-linux/src/control_bridge.rs` advertises and dispatches a
    broad method set including `surface.send_text`, `surface.send_key`, and
    command-bearing `pane.create`.
- The extension allowlist cannot be described as a security boundary unless
  the Rust bridge enforces it.
- `/tmp/limux.sock` remains compatibility behavior but must be treated as risky
  in the Cursor path until socket hardening is done.
- Extension tests should run from source with system Node 18.17+ and no package
  installation.
- Any future package-based Cursor test tooling is a separate supply-chain gate.

## 7. Testing Requirements

- Rust:
  - `cargo fmt --check`
  - `cargo check -p limux-host-linux`
  - `cargo check -p limux-cli`
  - focused unit tests for new host/control behavior
  - `./scripts/check.sh`
  - `./scripts/xvfb-smoke-test.sh` or documented equivalent
- Extension:
  - `node --test` for pure JS tests
  - no npm/npx/package scripts in v1
  - tests for socket discovery, stale detection, framing, allowlist rejection,
    TreeDataProvider behavior, and path eligibility
- Manual smoke:
  - `cursor --extensionDevelopmentPath integrations/cursor-limux`
  - set or resolve `LIMUX_SOCKET`
  - verify tree, refresh, runtime identity, focus/present behavior, empty pane,
    folder open, and visible snapshot

## 8. Rollback Plan

- Keep v1 behind explicit extension installation/development-path use until
  accepted.
- Do not change existing CLI/agent socket behavior unless a server-side
  restricted Cursor path is added as a separate path or role.
- If extension activation or runtime selection misbehaves, disable the Cursor
  extension without affecting Limux host operation.
- If host launcher behavior misbehaves, disable `Open in Cursor` unless an
  absolute executable path is configured.
- If server-side restriction introduces regressions, keep the existing full
  socket path for CLI/agent compatibility and disable the Cursor-restricted
  state-changing commands.

## 9. Success Metrics

- The operator can keep Cursor's file explorer visible while using Limux for
  terminal/agent panes.
- The operator can identify and select the correct Limux runtime from Cursor.
- The operator can open known workspace folders across Limux and Cursor without
  copy/pasting paths.
- The v1 Cursor path exposes zero terminal text/key/paste injection commands.
- Stale and duplicate sockets are detected instead of silently attaching to the
  wrong runtime.
- Full acceptance suite passes before merge.

## 10. Open Questions

- Should the Cursor-restricted method surface use a separate socket path or a
  caller role/nonce on the existing socket?
- What should be the operator's default configured Cursor executable path on
  this workstation?
- Should `/tmp/limux.sock` remain usable for Cursor after stricter socket
  hardening, or should Cursor require explicit path/XDG runtime only?
- Which real Wayland desktop session will be used for the manual presentation
  smoke?
- Should a future native Windows transport be a named pipe bridge, helper
  process, or Remote-WSL-only policy?

## 11. Implementation Gate

Implementation must not start until:

- The current Limux PR cleanup is merged and the implementation branch is based
  on updated mainline.
- TaskMaster parsing succeeds or the TaskMaster blocker is routed to the
  TaskMaster managers.
- The operator explicitly opens the Cursor integration implementation lane.
- The v1 server-side trust-boundary decision is made.
