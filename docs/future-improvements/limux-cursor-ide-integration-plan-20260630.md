# Limux Cursor IDE Integration Plan

Date: 2026-06-30
Author/runtime: lifo / Codex
Status: draft revised after native, GLM, and MiniMax adversarial review; Kimi
broad-review lane parked because Hermes core-dumps after a successful response
Implementation gate: do not implement until current Limux PR cleanup is merged
and the operator explicitly opens the Cursor integration implementation lane.

## Ready-To-Implement Gate

This is not implementation-ready until these gates are true:

1. Current Limux cleanup PRs are merged and the working branch is based on the
   updated mainline.
2. A v1 method registry is pinned exactly as below, with Rust and extension
   request builders sharing the same method list.
3. The server-side trust boundary is resolved. Extension-side request builders
   are not sufficient because the current Limux socket authorizes any same-user
   peer in `LocalUser` mode.
4. Socket discovery, stale-socket handling, and multi-runtime selection are
   specified and tested against fake live/stale sockets.
5. The acceptance checks are executable, not prose-only: extension `node
   --test`, Rust contract tests, `./scripts/check.sh`, and the Xvfb smoke
   harness must cover the v1 safety claims.
6. Kimi remains parked unless the Hermes post-response core-dump is isolated;
   GLM + direct MiniMax review artifacts are sufficient for this planning pass
   only because the Kimi failure is a Hermes runtime instability.

## Purpose

The operator wants the Limux terminal/runtime workflow while regaining the
Cursor IDE file explorer and editor surface. This plan treats Cursor as the file
and editor cockpit, and Limux as the terminal, pane, workspace, and agent
runtime. It does not try to embed the GTK Limux window inside Cursor for v1.

## Recommendation

Build v1 as a Cursor/VS Code-compatible extension bridge plus one small Limux
host action:

1. A no-dependency extension under `integrations/cursor-limux/`.
2. A Cursor side view that lists Limux workspaces, panes, tabs, and surfaces
   from the running Limux control socket.
3. Cursor commands for focusing Limux workspaces, creating empty panes, opening
   visible viewport snapshots, and opening the current Cursor folder in Limux.
4. A Limux context-menu action to open a workspace folder in Cursor.

This gives the operator the file explorer/editor on the left without forcing a
risky terminal emulation rewrite as the first step.

## Non-Goals For V1

- Do not embed the GTK window in Cursor.
- Do not replace Ghostty terminal rendering with a webview terminal.
- Do not expose arbitrary `surface.send_text`, `surface.send_key`, CLI `send`,
  `pane.create.command`, `workspace.create.command`, or any raw request
  passthrough from Cursor in v1.
- Do not add a generic command-palette entry that accepts arbitrary Limux method
  names or request JSON.
- Do not depend on host `npm`, `npx`, package scripts, or package-resolution
  during normal v1 operation, development, or tests.
- Do not support native Windows-host Cursor talking directly to WSL Limux in
  v1. Supported v1 topologies are native Linux Cursor and Windows Cursor with
  the Remote-WSL extension host running inside the target WSL distro. Native
  Windows Cursor without Remote-WSL must fail closed with a clear message.

## Architecture

### Extension Location

Create a plain JavaScript extension:

```text
integrations/cursor-limux/
  package.json
  extension.js
  README.md
  media/
```

The first version should use only Node built-ins and the Cursor/VS Code
extension API. `package.json` should have no dependencies, no devDependencies,
and no package-manager-backed scripts. Package manager execution is not required
to run the extension from source with `--extensionDevelopmentPath`.

### Socket Resolution

The extension must mirror the Limux resolver instead of inventing a parallel
fallback list. Current Limux source resolves runtime sockets in this order:

1. Cursor setting: `limux.socketPath`.
2. Environment: `LIMUX_SOCKET`.
3. Environment: `LIMUX_SOCKET_PATH`.
4. `${XDG_RUNTIME_DIR}/limux/limux.sock` when `XDG_RUNTIME_DIR` is set.
5. `/tmp/limux.sock` only as the existing Limux compatibility fallback.

Do not add `/run/user/${uid}/limux/limux.sock` as a separate Cursor fallback
unless Limux's own resolver adds it first. On normal Linux desktops,
`XDG_RUNTIME_DIR` is already `/run/user/${uid}`.

Failure to connect should show a clear, non-spammy error and leave the tree view
in a disconnected state with a refresh action.

Ambient `/tmp/limux.sock` is risky because `/tmp` is world-writable. Cursor
should use it only when it is the exact resolved Limux fallback and the
connected host passes identity checks. A follow-up hardening task should reject
symlinked parents and stale pre-created socket paths before expanding Cursor
support.

On connect, the extension must call `system.identify` and display the socket
path, PID, start time, build identity, and runtime identity. Every request must
use short connect and response timeouts. On `ECONNREFUSED`, timeout, or a socket
file that cannot be identified, label the path as stale/unreachable and do not
retry silently.

If multiple candidate sockets are configured or discovered, the extension must
probe all candidates, run `system.identify`, reject stale or ambiguous entries,
and present a quick-pick of live runtimes. State-changing commands must pin the
chosen runtime identity and refuse to run if a later identify response changes.

For native Windows Cursor without Remote-WSL, disable the extension because the
extension host cannot use the WSL Unix socket. Windows-host transport through
`\\wsl$`, a Windows named pipe, or a helper process is a separate v2+ transport
problem.

### Control Interface

Use the existing Limux Unix socket JSON request/response protocol where it has
coverage, but do not expose that socket as a generic JSON client inside Cursor.
The extension must use typed request builders and a strict allowlist. Add
narrowly scoped bridge methods only where the existing surface is too broad.

Important trust-boundary correction: the current Limux bridge is broader than
the Cursor extension. In default `LocalUser` mode, any same-user process that
can reach the Unix socket can call the full bridge surface, including terminal
send methods. Therefore the v1 Cursor allowlist is a client safety guardrail,
not a server security boundary, unless a server-side Cursor-restricted bridge
or per-connection method allowlist is added.

Before shipping state-changing Cursor commands, add one of these server-side
contracts:

- a separate Cursor-restricted socket path whose method set is server-enforced;
  or
- a per-connection caller role/nonce plus server-side method allowlist.

The allowlist must be checked by the Rust bridge after peer authentication and
before dispatch. `system.capabilities` should reflect the restricted method set
for Cursor connections.

V1 methods needed:

- `workspace.list` or existing equivalent used by `limux --json list-workspaces`,
  extended to expose `folder_path`, `openable_path`, and `path_source`.
  Precedence should be explicit: workspace `folder_path`, then last known
  terminal `cwd` only when it is canonicalized, exists, is a directory, and
  passes the configured workspace-root policy.
- `workspace.select` is selection only.
- `window.present` is the separate best-effort raise/focus request. It must
  return whether presentation succeeded, failed, or is unsupported. Wayland
  focus limitations must be visible to the user.
- `cursor.pane_create_empty` creates a terminal pane with no command field.
  The Rust route must reject any supplied command or unknown payload field for
  this Cursor method.
- `surface.read_text` returns the current visible viewport only. It must have a
  fixed/bounded response shape and reject scrollback parameters such as `count`
  or `offset` until the terminal side explicitly supports them.
- `cursor.workspace_open_folder` or another allowlisted method for "open current
  Cursor folder in Limux"; it must not accept arbitrary shell commands.

Avoid v1 Cursor commands that inject arbitrary text into terminals. This means
no `surface.send_text`, no `surface.send_key`, no terminal `send` aliases, and
no `command` fields on workspace or pane creation from the extension. This must
be enforced in both the extension request builders and the server-side
Cursor-restricted bridge.

Same-user Unix socket authentication remains the host-level boundary for other
local tools. The v1 extension boundary is narrower by design: it must not turn
Cursor into a broad terminal-injection client even though the underlying Limux
socket can serve more powerful local callers.

### Cursor UI

Add one Activity Bar container or side-bar view named `Limux`.

The extension should use the standard VS Code/Cursor extension API:

- `contributes.viewsContainers.activitybar` or a sidebar view contribution.
- `contributes.views` for the Limux workspace tree.
- `activationEvents` such as `onView:limux-workspaces` so the extension lazily
  activates when the view opens.
- `vscode.TreeDataProvider<T>` with stable item IDs, context values, and
  `onDidChangeTreeData` refresh.
- Node `net.connect` to the Unix socket with the existing Limux framing.

Tree shape:

```text
Workspace
  Pane
    Tab / Surface
```

Each item should expose only commands that are safe for that scope:

- Refresh.
- Focus workspace in Limux.
- Open workspace folder in Cursor.
- Create pane in workspace.
- Open read-only surface snapshot.
- Open current Cursor folder in Limux.

The tree should not try to mirror every live character-cell terminal update.
Snapshots are sufficient for v1.

### Limux Host UI

Add `Open in Cursor` to workspace row/context-menu actions when a workspace
folder or safe cwd is known.

Launcher behavior:

- Require a configured absolute executable path, for example
  `limux.cursorExecutable` or `LIMUX_CURSOR_BIN`, before launching Cursor from
  Limux. A PATH lookup for `cursor` may be added later only with an explicit
  user confirmation and tests.
- Use safe argv construction, not shell interpolation.
- Canonicalize the path and require an existing local directory.
- Reject symlink escapes, missing paths, files, non-local paths, and folders
  outside the configured workspace-root policy or current workspace set.
- In WSL, decide explicitly whether `/mnt/<drive>/...` paths are allowed. If
  they are not in the same extension-host environment, reject them with a clear
  message instead of launching Cursor against the wrong mount namespace.
- On launch failure, report through existing Limux notification/log surfaces.

For the Cursor-side "open current folder in Limux" command, accept only a
`file:` workspace folder whose `fsPath` is a local Linux path in the same
environment as the Limux socket. Reject remote, virtual, untitled, or missing
folders. For multi-root Cursor workspaces, use a quick-pick instead of guessing.

## V2: Attach Mode

If v1 proves useful, evaluate a true tmux-like attach mode. Do not implement it
as a simple extension of the request/response bridge. Mark this section
`REWORK BEFORE PRD` until a protocol spike is complete.

The first v2 milestone should be a read-only attach spike:

1. Define attach IDs, initial snapshot semantics, incremental output frames,
   heartbeat, detach cleanup, host shutdown behavior, reconnect, replay/resync,
   maximum clients, and slow-reader/backpressure behavior.
2. Prove whether Ghostty output can be observed without stealing ownership from
   the GTK surface.
3. Keep the live PTY/Ghostty surface owned by Limux.

Bidirectional input and resize must wait for a separate protocol contract:

- Resize authority: view-only attaches do not resize the PTY. An active attach
  must explicitly acquire resize authority, all resize writes must reuse the
  existing Limux coalescing/de-duplication path, and conflicting GTK/Cursor
  geometries must produce a visible state instead of competing SIGWINCH streams.
- Input taxonomy: printable keys, control keys, paste payloads, IME/preedit,
  selection copy, standard clipboard paste, primary-selection behavior, and
  alternate-screen behavior must be separate protocol concepts. Do not add a
  raw PTY-write shortcut.

This is more powerful and much riskier than v1. It touches terminal state,
copy/paste, resizing, keyboard modes, and agent TUI behavior. It should be a
separate PRD and review lane.

## Tests And Verification

Rust side:

- Unit tests for `workspace.select` selection-only request handling.
- Unit tests for `window.present` return states.
- Unit tests for safe Cursor launcher argv construction.
- Unit tests for `workspace.list` `folder_path`, `openable_path`, and
  `path_source` metadata.
- Unit tests proving extension-safe host methods cannot carry terminal text,
  key, or command payloads.
- Unit tests proving the server-side Cursor-restricted method allowlist rejects
  `surface.send_text`, `surface.send_key`, raw `pane.create.command`, and
  unknown methods.
- Unit tests for socket-path hardening and stale socket detection behavior.
- Existing `./scripts/check.sh`.
- Xvfb smoke where practical for host focus/menu behavior.

Extension side:

- Source-level tests for socket path resolution and protocol framing.
- Source-level tests with Node built-in `node --test`; do not require `npm test`,
  `npx`, `vsce`, `yo code`, or `@vscode/test-electron` in v1.
- Declare the minimum Node version for direct source tests; target Node 18.17+
  unless the extension host proves a higher floor is needed.
- Tests proving request builders cannot emit `surface.send_text`,
  `surface.send_key`, `pane.create.command`, `workspace.create.command`, or raw
  arbitrary method calls.
- Wire-level tests proving the actual framing/client path rejects forbidden
  methods, not only the public builders.
- TreeDataProvider tests for refresh, stable IDs, context values, and command
  registration.
- Tests for local-folder eligibility: `file:` only, existing local directory,
  multi-root quick-pick, and rejection of remote/virtual/untitled folders.
- Negative launcher tests for paths with spaces, leading dashes, shell
  metacharacters, broken symlinks, symlink escapes, files, and missing dirs.
- Manual smoke with Cursor or VS Code:
  `cursor --extensionDevelopmentPath integrations/cursor-limux`.
- Verify the tree matches `limux --json list-workspaces`.
- Verify refresh, focus workspace, create pane, open folder in Cursor, and
  read-only snapshot behavior.
- Verify two simultaneous Limux sockets plus one stale socket do not silently
  attach Cursor to the wrong runtime.

V2 attach mode must not ship without a broader test matrix:

- Fake alternate-screen TUI and normal-screen shell.
- Copy/paste round trip and bracketed-paste behavior.
- Selection copy and primary-selection behavior where supported.
- Read-screen consistency before, during, and after attach.
- Attach, detach, reconnect, host shutdown, and two attached clients.
- Cursor resize storm, GTK resize storm, simultaneous GTK and Cursor views, and
  post-detach size restoration.
- Dead socket, stale runtime, and multi-runtime socket selection.

Operator workflow acceptance:

- User can keep Cursor's file explorer visible while Limux owns terminal panes.
- User can select a Limux workspace from Cursor and have the correct Limux
  workspace come forward.
- User can open the current Cursor folder in Limux without copy/pasting paths.
- No arbitrary text injection command ships in v1.

These acceptance checks should be executable before the implementation PR is
considered mergeable. Add a lightweight `v1-acceptance` smoke or script that
verifies the tree matches `limux --json list-workspaces`, the forbidden methods
are not exposed through the Cursor path, folder round-trip works, and stale or
duplicate sockets do not attach silently.

## TaskMaster Status

This repo currently has `.taskmaster/docs/*.md` notes but no live
`.taskmaster/config.json`, `.taskmaster/state.json`, or
`.taskmaster/tasks/tasks.json`. Per the Codex TaskMaster policy, this draft
does not invent task IDs or manually create task storage.

When the implementation lane opens, bootstrap or repair TaskMaster through the
reviewed wrapper/runbook first, then import this plan as the PRD/task source.

## Source Notes

- Existing Limux future-options note:
  `docs/future-improvements/limux-cursor-integration-options-after-pr-greenlight.md`.
- Hermes CLI docs used for the adversarial review command shape:
  `https://hermesagent.org.cn/en/docs/user-guide/cli`.
- Cursor extensions are VS Code-compatible for the APIs this plan relies on:
  Tree View, commands, extension activation, and `Pseudoterminal`.

## Review Gate Status

The operator requested a buffered Hermes adversarial-review wave using:

- 3 to 5 GLM 5.2 reviewers from Ollama Cloud.
- 3 to 5 MiniMax M3 reviewers.
- 3 Kimi K2.7 Code reviewers from Ollama Cloud.

The review brief is saved at:

```text
docs/reviews/limux-cursor-ide-integration-20260630/REVIEW_BRIEF.md
```

Final status on 2026-06-30:

- GLM through Ollama Cloud succeeded for architecture, runtime, Cursor API, and
  trust-boundary lenses.
- Direct MiniMax M3 succeeded for security, runtime, test, and sequencing
  lenses.
- Kimi `kimi-k2.7-code` reached Ollama Cloud and returned the expected smoke
  response, but Hermes then dumped core during/after CLI cleanup. Rumi advised
  keeping Kimi out of broad reviewer waves until that runtime instability is
  isolated. `kimi-k2.7` is not a valid fallback and returned 404.

Review artifacts are saved in:

```text
docs/reviews/limux-cursor-ide-integration-20260630/HERMES_MODEL_SMOKE_STATUS.txt
docs/reviews/limux-cursor-ide-integration-20260630/MANAGER_SYNTHESIS.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-*.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-*.md
```

Native Codex subagents completed a local fallback adversarial review on
2026-06-30. Their verdict was no P0s, but `ACCEPT_WITH_FIXES` / `REWORK BEFORE
IMPLEMENTATION`. External GLM/MiniMax reviewers returned `PASS_WITH_CHANGES`
with P0/P1 issues; those findings are incorporated above. The Kimi lane remains
a Hermes diagnostic follow-up, not a reason to treat the GLM/MiniMax findings
as unreviewed.
