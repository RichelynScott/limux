# Hermes MiniMax Direct Review

Slot: minimax-runtime
Model: MiniMax-M3
Provider: minimax
Generated: 2026-06-30 17:23:44 EST

```text

session_id: 20260630_172345_04095f
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P0] Socket-resolution list ignores WSL/Windows-host reality — `limux-cursor-ide-integration-plan-20260630.md` lines 66-75. Resolution stops at `/tmp/limux.sock`, but on WSL the typical real path is `/mnt/wsl/.../<distro>/run/user/<uid>/limux/limux.sock` when accessed from a Windows-host Cursor, or a Windows-side pipe `\\.\pipe\limux` once WSL mirrors it. Without a Windows-host mode the "Cursor inside WSL/Remote first" claim is wishful — most WSL operators run native Windows Cursor. Fix: add an explicit discovery branch that detects `WSL_DISTRO_NAME`/`WSL_INTEROP`, exposes `wslpath -u` translation, and documents `\\.\wsl$\<distro>\...` as the only Windows-host reach path. Either ship that or demote v1 to WSL-Cursor-only with a clear failure message.

2. [P0] No strategy for stale/duplicate sockets — lines 78-84 + "Verify two simultaneous Limux sockets plus one stale socket do not silently attach Cursor to the wrong runtime" (lines 221-222). Identify call is mentioned but no allowlist, no PID/build fingerprint comparison, no automatic selection rule. On WSL after a host restart the `/tmp/limux.sock` fallback is the classic stale-socket footgun. Fix: define a concrete selection algorithm — connect to all candidates, run `identify`, reject sockets whose `build_id`/`pid`/`started_at` does not match the expected live host, present survivors in a quick-pick; ship a regression test that pre-creates a fake socket and asserts it is refused.

3. [P1] Multi-runtime targeting unresolved — `control_bridge.rs` (per AGENTS.md) serves workspace/surface/send/notify. The plan lists `cursor.workspace_present` as a possible new method but never specifies how the extension addresses a specific runtime when both a dev `target/debug/limux` and an installed `limux` are listening. Fix: mandate a runtime identity token (`system.identify` returning `runtime_id`, `pid`, `socket_path`, `build_id`) and require the extension to pin one before any state-changing call; reject cross-runtime `workspace.select` against an unintended runtime.

4. [P1] WSL argv/cwd hazards in the Limux→Cursor launcher — lines 148-160. `cursor <folder>` with safe argv is fine on Linux, but the canonicalized path may be `/mnt/c/...` when the Limux cwd came from a Windows-side shell, and the resulting Windows-Cursor invocation lives in a different mount namespace. Fix: detect `/mnt/[a-z]/` paths and either translate via `wslpath -w` and launch `cursor.exe` or refuse with a clear message; add a unit test that proves argv construction never shells out and never passes Windows-style paths unmodified.

5. [P1] Wayland focus assumption is asserted, not verified — line 101-102 says "verify on a real desktop in addition to Xvfb." No concrete desktop environment, no test matrix, no fallback when `workspace.present { present: true }` cannot actually raise the GTK window (common on Wayland without `xdg-activation`). Fix: require an explicit contract — either adopt `xdg-activation` token passing through the socket, or document that focus is best-effort and ship a smoke that proves failure surfaces in the UI, not silently.

6. [P2] No timeout/heartbeat on identify + tree refresh — lines 78-84, 217-220. WSL Unix sockets can hang forever when the host is mid-restart or the distro is paused (`wsl --shutdown` mid-session). Fix: every extension→host call gets a connect timeout (e.g., 1s) and an overall request timeout (e.g., 3s), with a clear "Limux unreachable" state and manual refresh; add a regression test that points at a black-hole path.

7. [P2] Read-only snapshot scope is underspecified — line 105-107 restricts to visible viewport but `surface.read_text` is not enumerated against the current control-bridge surface in AGENTS.md. Confirm the method exists or add it; either way specify max-bytes, encoding, and refresh cadence before claiming "snapshots are sufficient for v1."

Missing Evidence:
- Live read of `rust/limux-host-linux/src/control_bridge.rs` to confirm `workspace.list` already exposes `folder_path`/`path_source` or that the listed "extend workspace.list" change is real work, not assumed.
- Confirmation that the identify surface returns a build fingerprint stable enough for stale-socket rejection.
- WSL smoke matrix: native-Windows Cursor → WSL Limux (covered?), Cursor-via-WSLg → WSL Limux, Cursor-on-Linux → WSL-Limux, plus dual-runtime host case.
- Concrete proof that `xdg-activation` is or is not available on the operator's compositor.

Recommended Plan Changes:
- Add a "WSL & multi-runtime" subsection mandating a runtime-identity handshake, blackhole/timeout handling, and explicit refusal of stale `/tmp` sockets.
- Tighten "Open in Cursor" launcher with `wslpath` translation rules and a refusal path for non-Linux folders.
- Replace the Wayland sentence with an `xdg-activation` decision (use it or document best-effort + UI surface).
- Make the `node --test` suite cover stale-socket rejection, multi-socket selection, and timeout paths, not only request-builder allowlist.
- Keep the implementation gate closed until at least the GLM/Kimi rerun lands; the MiniMax credential gate is still operator-blocked per synthesis.

```

Exit status: 0
