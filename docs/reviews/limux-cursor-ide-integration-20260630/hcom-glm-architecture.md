# Hermes GLM Review

Slot: glm-architecture
Model: glm-5.2
Provider: ollama-cloud

--- response ---

session_id: 20260630_172558_5911c8
I now have enough verified evidence. Here is my review.

---

Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P1] Socket resolution step 5 is fabricated against the source.
   Plan §"Socket Resolution" lists six steps including
   `${XDG_RUNTIME_DIR}/limux/limux.sock` (step 4) and
   `/run/user/${uid}/limux/limux.sock` (step 5) as distinct fallbacks.
   `rust/limux-control/src/socket_path.rs:91-101` defines only ONE
   default: `XDG_RUNTIME_DIR/limux/limux.sock`, else `/tmp/limux.sock`
   (the `RUNTIME_SUBDIR` is `"limux"`, name `"limux.sock"`). There is no
   `/run/user/${uid}` branch — that path only appears if
   `XDG_RUNTIME_DIR` itself is set to `/run/user/${uid}`, which is one
   step, not two. The plan's step 5 will never match and gives a false
   impression of richer discovery.
   Fix: collapse to five steps: setting → `LIMUX_SOCKET` →
   `LIMUX_SOCKET_PATH` → `XDG_RUNTIME_DIR/limux/limux.sock` →
   `/tmp/limux.sock`. Drop the synthetic `/run/user/${uid}` entry.

2. [P1] `workspace.list` does not emit `folder_path`, `openable_path`, or
   `path_source` today; the plan depends on fields that do not exist.
   The plan §"Control Interface" requires `workspace.list` to "expose
   `folder_path`, `openable_path`, and `path_source`." Verified:
   `search_files` for `openable_path|path_source` across
   `rust/limux-host-linux/src` and `rust/limux-core/src` returns ZERO
   matches. `folder_path` exists as a host-side struct field
   (`layout_state.rs:67`, `window.rs:76`) but is not confirmed to be
   serialized into the `workspace.list` JSON response that the CLI
   `list-workspaces` path emits (`control_bridge.rs:487-489` →
   `ControlCommand::ListWorkspaces`). The plan acknowledges this is
   needed ("V1 methods needed") but does not scope the Rust-side
   response schema change as an implementation prerequisite.
   Fix: add an explicit pre-implementation task: "Extend the
   `ListWorkspaces` response to include `folder_path`,
   `openable_path`, and `path_source` (workspace folder → terminal cwd
   → none), with unit tests in both `limux-host-linux` and
   `limux-core`." Without this, the Cursor extension cannot populate the
   tree or drive "Open in Cursor."

3. [P1] `workspace.select` has no `present` parameter and no
   `window.present` method; the plan floats two options without
   committing.
   Plan §"Control Interface" says either `workspace.select { present:
   true }` with tests OR a new `window.present` /
   `cursor.workspace_present` method. Verified:
   `control_bridge.rs:147-150` defines `SelectWorkspace { target, reply
   }` — no `present` field. `workspace.select` handler
   (`control_bridge.rs:587-593`) accepts only a target. `window.present`
   exists only as a GTK-internal call (`window.rs:1924`, `window.rs:2952`,
   `settings_editor.rs:48`), not as a control-bridge method. The plan
   must pick one path and scope it, because "focus workspace in Limux"
   is a v1 tree command that cannot ship without it.
   Fix: recommend `workspace.select { present: true }` as the lower-risk
   extension (adds a boolean to the existing route, reuses the existing
   `window.present()` GTK path already called on workspace switch).
   Add it to the Rust-side prerequisite task list with a unit test for
   both `present: true` and `present` absent.

4. [P2] `system.identify` exists and is allowlisted; the plan under-specifies
   its use for stale-runtime detection.
   `control_bridge.rs:22,473` confirms `system.identify` is a live bridge
   method. Plan §"Socket Resolution" says "call the existing
   identify/health surface or a new `system.identify` equivalent" — the
   "or new equivalent" is unnecessary hedging; the method exists. The
   stale/ambiguous-socket detection logic is described qualitatively but
   no concrete field contract (build id, socket path, pid, start time)
   is specified for the extension to compare against.
   Fix: drop "or a new equivalent"; define the exact `system.identify`
   response fields the extension reads for staleness checks.

Missing Evidence:
- No evidence the `ListWorkspaces` JSON response currently serializes
  `folder_path` (only that the struct field exists in host state). The
  plan's v1 tree and "Open in Cursor" both depend on it being present
  in the wire response.
- No evidence of a `window.present` or `present` field anywhere in the
  control bridge or core dispatcher. The plan's two-option language
  papers over a real gap.
- The plan's external-review gate (GLM/MiniMax/Kimi wave) is
  documented as unrun in `MANAGER_SYNTHESIS.md:234-236`. This review is a
  single-lens pass and does not satisfy that gate.

Recommended Plan Changes:
- Replace the six-step socket resolution with the actual five-step
  source order; add a unit test in the extension that mirrors
  `socket_path.rs` tests.
- Add a Rust-side prerequisite section ("Host-side changes before
  extension v1") listing: (a) extend `workspace.list` response with
  `folder_path`/`openable_path`/`path_source` + tests, (b) add
  `present: true` to `workspace.select` + tests, (c) confirm
  `pane.create` command-forbidding is enforced at the bridge parser
  (already verified at `control_bridge.rs:438` via
  `optional_terminal_text(..., "pane.create command")` — cite this).
- Commit to `workspace.select { present: true }` as the v1 focus path;
  drop the `window.present` alternative to avoid ambiguity.
- Remove "or a new `system.identify` equivalent" — the method is live;
  specify which response fields drive the stale-runtime check.
- Note that `surface.read_text` aliases (`read-screen`, `capture-pane`)
  already exist (`control_bridge.rs:550`); the extension's typed
  request builder should use `surface.read_text` as canonical and the
  plan should state the alias set is intentionally not exposed to the
  extension UI.

--- end response ---

Exit status: 0
