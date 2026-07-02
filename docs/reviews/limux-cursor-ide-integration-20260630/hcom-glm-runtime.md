# Hermes GLM Review

Slot: glm-runtime
Model: glm-5.2
Provider: ollama-cloud

--- response ---

session_id: 20260630_172558_29bfb6
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P1] Socket resolution diverges from actual Limux resolver. The plan (lines 67-74) adds `/run/user/${uid}/limux/limux.sock` as step 5, but `rust/limux-control/src/socket_path.rs:91-101` has no such fallback — it goes `XDG_RUNTIME_DIR` → `/tmp/limux.sock` directly. In WSL2 without systemd, `XDG_RUNTIME_DIR` is unset and Limux uses `/tmp/limux.sock`. The extension's step 5 could find a stale socket from a prior systemd-enabled session while Limux itself is listening on `/tmp/limux.sock`. Fix: drop step 5, or add it to Limux's own resolver first. The extension must mirror Limux's resolution exactly.

2. [P1] No stale-socket detection contract. Plan lines 81-83 say "warn or refuse to attach until the user chooses" but never define what "stale" means. In WSL2, when the host crashes or WSL restarts, the socket file persists on disk. A `connect()` returns ECONNREFUSED — indistinguishable from "host not running." `socket_path.rs:118` (`remove_existing_socket`) handles stale cleanup server-side, but the extension has no client-side detection spec. Fix: specify that on ECONNREFUSED, the extension must `fs.stat` the path; if it's a socket file but connect fails, label it stale, offer cleanup or re-resolve, and do not silently retry in a loop.

3. [P1] WSL extension-host topology is underspecified. Plan line 44 excludes Windows-host Cursor directly, but doesn't state that when Cursor runs via Remote-WSL, the extension host executes inside WSL — where `XDG_RUNTIME_DIR`, `/run/user/${uid}`, and the Unix socket all exist. When Cursor runs natively on Windows (no Remote-WSL), the extension cannot reach the Unix socket at all. Fix: add an explicit activation guard — if `process.platform` is not Linux and no WSL Remote tunnel is detected, disable the extension with a clear message. The plan should name the two supported topologies: (a) native Linux Cursor, (b) Windows Cursor + WSL Remote extension.

4. [P2] Multi-socket discovery mechanism is missing. Plan line 222 tests "two simultaneous Limux sockets plus one stale socket" but the resolution order (lines 67-74) yields a single path. There is no discovery step that scans `XDG_RUNTIME_DIR/limux/`, `/tmp/`, or configured paths for multiple sockets. Fix: if multi-runtime selection is a v1 test requirement, add a discovery phase to the socket resolution spec — enumerate candidate paths, probe each, and present a quick-pick when more than one live socket is found.

5. [P2] `workspace.list` metadata fields confirmed absent. `workspace_row` at `window.rs:290-304` returns `cwd` only — no `folder_path`, `openable_path`, or `path_source`. The plan correctly identifies this as a required addition (lines 96-98). No gap in the plan, but the Rust-side work item should be called out as a prerequisite for both "Open in Cursor" and "Open current folder in Limux" commands.

Missing Evidence:
- No evidence that Cursor's extension host runs inside WSL under Remote-WSL with access to Unix sockets — the plan asserts it but doesn't cite VS Code Remote-WSL docs.
- No evidence that `cursor <folder>` works inside WSL when Cursor is running on Windows via Remote-WSL (it may need `cursor.cmd` or a Windows-side path).
- V2 attach mode has zero protocol spike output; the "REWORK BEFORE PRD" tag is appropriate but no spike timeline or owner is defined.

Recommended Plan Changes:
- Remove `/run/user/${uid}/limux/limux.sock` from the extension resolution order or add it to Limux's resolver first — the extension must not check paths Limux never uses.
- Add a concrete stale-socket detection contract: ECONNREFUSED + socket file present = stale; specify cleanup behavior and max retry.
- Add an activation-environment guard section naming the two supported topologies and the hard-fail case (Windows-native without Remote-WSL).
- If multi-socket selection is a v1 acceptance test, add a discovery/quick-pick spec to the socket resolution section.
- Note `cursor <folder>` command availability inside WSL Remote-WSL as an open question requiring manual verification before implementation.

--- end response ---

Exit status: 0
