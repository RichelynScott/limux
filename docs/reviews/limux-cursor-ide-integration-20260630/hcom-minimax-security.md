# Hermes MiniMax Direct Review

Slot: minimax-security
Model: MiniMax-M3
Provider: minimax
Generated: 2026-06-30 17:23:44 EST

```text

session_id: 20260630_172345_954be2
I have enough to render the security/socket-trust review. Findings below.

Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P0] The plan's "narrow by design" claim is only schema-level — not enforced at the Rust bridge. Plan lines 87–119 (`docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`) say "do not expose surface.send_text / send_key / command fields" and call it a "schema and request-builder requirement," but the live socket server in `rust/limux-host-linux/src/control_bridge.rs:823` accepts ANY same-uid connection (`SocketControlMode::LocalUser` default in `rust/limux-control/src/auth.rs:25-32, 62-68`) and `handle_client` dispatches arbitrary `ControlCommand`. So the v1 trust boundary lives only in the Cursor extension's allowlist — if a future bug or sibling extension bypasses the typed builders, the socket still serves full power. Fix: add a server-side `allowlist: &[&str]` mode on the control bridge (e.g. `LIMUX_SOCKET_ALLOWLIST=workspace.list,workspace.select,cursor.*,pane.create_empty,surface.read_text`) that the bridge checks against `method` after peer auth. Cursor-only sockets should bind a separate path or run under a dedicated `SocketControlMode::LimuxOnly` plus method allowlist, not share the existing `LIMUX_SOCKET`.

2. [P0] `/tmp/limux.sock` as a fallback is a privilege-conferring default. Plan lines 73–74 (`...limux-cursor-ide-integration-plan-20260630.md`) keep `/tmp/limux.sock` as a "last compatibility fallback," but `/tmp` is world-writable sticky and `/tmp/limux.sock` mode 0o600 is set after bind (`rust/limux-control/src/socket_path.rs:62-87`). A local non-uid attacker can pre-create `/tmp/limux.sock` as a symlink, force Limux's bind to follow it, then steal the connection — TOCTOU. The CLI itself should never open `/tmp/limux.sock` unless the operator opted in. Fix: drop `/tmp/limux.sock` from the extension's resolver; require explicit `limux.socketPath` or `$XDG_RUNTIME_DIR`/`/run/user/${uid}`. Also reject symlinked parent dirs in `prepare_socket_path` and `O_NOFOLLOW` the bind.

3. [P0] Peer credential check uses `SO_PEERCRED`, which on Linux returns the peer's credentials at `connect()` time — but `pid` is not re-validated against `cmdline`/`exe` and the `is_descendant` check in `auth.rs:108-128` walks `/proc/<pid>/stat` only once at accept. `LIMUX_SOCKET_MODE=limuxOnly` (plan implies same-uid + descendant) does not prevent a non-Cursor same-user process from connecting via the same allowlisted bridge if a future sibling shares mode. The Cursor extension must additionally pass an "extension nonce" (per-session random token minted by the host) and present it in the first request; the bridge refuses requests without the matching token when the connection mode is Cursor-coupled. The plan does not mention per-session binding.

4. [P1] "Multi-socket / stale-runtime" disambiguation (plan lines 81–84) is identity-only (`identify`/build), but no spec is given for what `identify` returns, how `path_source` differs from the resolved path, or what "stale" means. A malicious local user can squat the expected path and forge an identify response with a stale build tag — Cursor would then attach to the wrong Limux. Fix: add a host-minted `host_token` (random 128-bit, persisted only in the GTK host's private state) that `identify` echoes; the extension compares it against a user-confirmed "last known token" before granting write commands (pane create, folder open). Refuse silently on mismatch.

5. [P1] `cursor.workspace_open_folder` and the reverse "Open current Cursor folder in Limux" carry filesystem path strings across the trust boundary. Plan lines 153–161 say "canonicalize and require existing local directory" but do not say the Limux host must also reject paths outside an operator-configured workspace-root allowlist, symlinks, and `..` traversal. Fix: host-side allowlist (`LIMUX_WORKSPACE_ROOTS` or per-workspace `folder_path` set), `realpath()` with `..` checks, reject symlink components, refuse if realpath is not under an allowed root.

6. [P1] The plan does not address log/audit surface. Every Cursor request should be logged with `(peer.pid, peer.uid, method, workspace_id, surface_id, folder_path)` at the bridge level (existing logging at `control_bridge.rs:837` is connection-level only). Without per-method audit, "no surface.send_text from Cursor" cannot be verified post-hoc. Add structured audit log + a `cursor.last_actions` CLI command.

7. [P1] "Default to `cursor <folder>`" (plan line 152) launches an external process from a same-user connection. The host must validate `cursor` resolves under `$PATH` via a configured allowlist, not the system default — a hostile `$PATH` shim can hijack the launch. Fix: configurable `LIMUX_CURSOR_BIN`; reject unset + non-absolute.

8. [P2] `cursor.pane_create_empty` with `command` field forbidden — must be enforced at protocol-deserialization, not only in extension builders. Add a Rust unit test that submits a hand-crafted JSON frame with extra `command` field and asserts the bridge strips/rejects it.

9. [P2] `workspace.list` adding `folder_path`/`openable_path` is a new metadata leak surface. Plan should restrict these to same-user peers already (covered) but also document that they expose absolute paths to ANY same-user Cursor — acceptable here, but call it out so future reviewers don't reintroduce a broader accessor.

10. [P2] Plan does not pin Cursor/VS Code extension API version. Recent VS Code versions changed `Pseudoterminal` semantics; the no-deps `node --test` claim depends on stable API. Add a `engines.vscode` floor.

Missing Evidence:

- Method allowlist/denylist in the actual Rust bridge (no current implementation; plan asserts but does not require).
- Concrete `identify` response schema; what constitutes "stale".
- Path canonicalization & workspace-root policy for `cursor.workspace_open_folder`.
- Per-session binding (host-issued nonce) for the Cursor client.
- Audit log shape for per-method bridge calls.
- `LIMUX_CURSOR_BIN` / launcher allowlist specification.

Recommended Plan Changes:

- Move trust-boundary enforcement from "extension allowlist" to "host bridge allowlist + per-session nonce"; add `LIMUX_SOCKET_ALLOWLIST` env and document in `Architecture > Control Interface`.
- Replace `/tmp/limux.sock` fallback with "explicit operator opt-in only" or drop entirely from the extension resolver; add symlink/parent-dir TOCTOU defenses to `socket_path.rs`.
- Add `cursor.*` method namespace to the protocol so Cursor-coupled requests can be rate-limited and audited as a class.
- Specify `identify` response fields (`host_token`, `build`, `path`, `started_at`) and "stale" thresholds.
- Add `LIMUX_WORKSPACE_ROOTS` config + `realpath` validation + symlink/.. rejection for folder-open paths in both directions.
- Add per-method audit log table at bridge level and a `limux cursor audit` reader.
- Add a `LIMUX_CURSOR_BIN` allowlisted launcher; reject relative/`$PATH` lookups.
- Add Rust-side tests for "extension cannot inject `command`/`send_text`/`send_key`" via raw frame injection — currently only the extension builder is tested.
- Note: the `AGENTS.md`-flagged "live PTY/Ghostty surface owned by Limux" line should be repeated in v2 attach PRD as a hard invariant; v1 must not modify any existing method handler.

```

Exit status: 0
