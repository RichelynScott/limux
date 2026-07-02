# Hermes GLM Review

Slot: glm-control-trust
Model: glm-5.2
Provider: ollama-cloud


session_id: 20260630_172920_ffb224
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P0] Extension-side allowlist is a UX guardrail, not a trust boundary. The plan (Control Interface section) defines the method allowlist in the Cursor extension's typed request builders, but `control_bridge.rs:468-728` dispatches every method in the `METHODS` array (lines 22-39, including `surface.send_text`, `surface.send_key`, `pane.create` with `command`) to any authorized peer. `auth.rs:62-67` shows the only gate is `peer.uid == current_uid()`. Any same-user process bypasses the extension entirely and calls the socket directly. The plan must explicitly state: "The extension allowlist is advisory; the server-level trust boundary is same-user Unix credentials, and no per-connection method restriction exists." Otherwise reviewers/operators will misread the v1 non-goals as a security guarantee.

2. [P1] Server does not strip `command` from `pane.create`. `control_bridge.rs:84-99` documents `command` as a host extension field and `parse_create_pane_request` accepts it. The plan says the extension won't send `command`, but the server never rejects it. If the plan's intent is that Cursor-originated pane creation must never carry a command, add a server-side proof (unit test) that the `cursor.pane_create_empty` path or a restricted-identity connection rejects a non-null `command`. The current plan only tests the extension builder, not the server.

3. [P1] `system.capabilities` (bridge line 470-471) advertises the full method surface including `surface.send_text`/`surface.send_key` to every connected client. If the plan wants Cursor to be a narrow client, either filter the capabilities response by connection identity/role, or document that capabilities exposure is intentionally broad and the allowlist is enforced only client-side.

4. [P2] `/tmp/limux.sock` fallback (Socket Resolution step 6) is world-writable territory. The plan lists it without discussing symlink-race risk or socket file permissions. `socket_path` binds with `requires_owner_only_socket` for `LocalUser` mode (auth.rs:45-47), but `/tmp` fallback deserves an explicit note that socket permissions must be 0600 owner-only and that `/tmp` is a last resort, not a safe default.

5. [P2] No per-connection identity or role concept on the server. The plan proposes `cursor.*` namespaced methods but the server has no mechanism to distinguish a Cursor-originated connection from a CLI or agent-team connection. If the plan wants server-enforced narrowing for Cursor, it needs a connection-registration or caller-tag step (`system.identify` with a `caller` field exists at line 473-481 but is informational, not enforcement).

Missing Evidence:
- No test proving a direct-socket same-user caller can send `surface.send_text` while the extension cannot — the gap between client-side allowlist and server-side dispatch is untested.
- No evidence that `workspace.list` currently returns `folder_path`, `openable_path`, or `path_source`; the plan says these need to be added but doesn't show the current response shape.
- No verification that `cursor <folder>` (default launcher) exists on the target WSL/Linux environment or what its failure modes are.

Recommended Plan Changes:
- Add an explicit "Trust Boundary" subsection stating: same-user Unix socket auth is the only server-enforced boundary; the extension allowlist prevents accidental misuse, not malicious use. Any same-user process has full method access.
- Add a server-side unit test proving that the intended v1 method set (workspace.list, workspace.select, pane.create without command, surface.read_text) is handled, and that `command`-bearing pane.create from a restricted path is rejected — not just that the extension doesn't send it.
- Add a note on `/tmp/limux.sock` permissions and symlink risk, or remove it from the fallback chain.
- Document whether `system.capabilities` should be filtered for extension clients or left as-is with a stated rationale.
- Specify whether a future server-side per-connection method allowlist (connection role/tag) is a v1.1 or v2 concern.

Exit status: 0
