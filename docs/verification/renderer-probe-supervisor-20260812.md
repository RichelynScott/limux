# Renderer probe and supervisor source checkpoint

Date: 2026-08-12
Owner: Gula / Limux manager lane
Branch: `gula/renderer-supervisor-v2-20260812`
Base: `ffafacb74e403964205be4ce29440f4eb22dc6ab`

## Outcome

Limux now has a source-level, fresh-process renderer-probe path and a fail-closed CLI launch policy. The probe initializes GTK/GSK and one 64 x 64 `GLArea`, emits one bounded JSON diagnostic, and exits without initializing Ghostty, restoring a user session, binding a Limux socket, or launching a terminal child.

Automatic D3D12 selection is deliberately inactive. The policy will not inject renderer variables until the embedded Ghostty C API can remove the exact supervisor-owned keys from terminal children. Current Ghostty overlays environment values but cannot remove inherited keys; enabling automatic injection today would leak `GSK_RENDERER` and `GALLIUM_DRIVER` into every shell.

This checkpoint does not install, restart, or promote a runtime and makes no production-performance claim.

## Source behavior

- `limux-host --renderer-probe` is an isolated GTK/GSK process path.
- The CLI preserves any explicit caller renderer environment.
- Automatic probing requires all of: WSL, `/dev/dxg`, no explicit renderer policy, and an available terminal-child environment-removal capability.
- A successful D3D12 probe must report `GskGLRenderer` or `GskNglRenderer`, no software-fallback evidence, requested `GALLIUM_DRIVER=d3d12`, an open `/dev/dxg`, and a realized positive-size probe surface.
- A rejected, malformed, oversized, failed, or timed-out probe selects the inherited renderer policy.
- Probe stdout is retained only to the existing preview-runner bound of 262,144 bytes; the reader drains the pipe, and the parent reaps or terminates the child at the probe timeout.
- A future accepted automatic selection carries `LIMUX_AUTO_RENDERER_ENV` with the exact injected-key names so the host can remove only supervisor-owned keys from terminal children.

## TDD evidence

RED was observed before implementation:

- `cargo test -p limux-cli renderer_launch -- --nocapture` failed with missing launch-policy, selection, and acceptance symbols.
- `cargo test -p limux-host-linux renderer_probe_flag_is_detected -- --nocapture` failed with missing probe dispatch.
- The positive-size acceptance regression failed when a realized 0 x 0 surface was still accepted.

GREEN after implementation:

- `cargo test -p limux-cli renderer_launch -- --nocapture`: 10 passed, including real subprocess success, malformed JSON, non-zero exit, oversized stdout, and timeout kill/reap paths.
- `cargo test -p limux-host-linux renderer_probe -- --nocapture`: 1 passed.
- `cargo test -p limux-cli -- --nocapture`: 165 unit tests and 5 launcher-route tests passed.
- `cargo test -p limux-host-linux -- --nocapture`: 496 tests passed.
- Clippy passed for both touched packages with only the repository's already-known unrelated lints explicitly allowed:
  - Limux CLI: `clippy::manual_contains`, `clippy::bool_assert_comparison`.
  - Linux host: `clippy::if_same_then_else` in `layout_state.rs`.

The unqualified `cargo clippy ... -D warnings` remains red on those pre-existing, out-of-scope diagnostics. No new renderer diagnostic appeared.

The repository-wide `cargo fmt --check` also remains red on pre-existing formatting drift outside the renderer hunks. The new renderer module and renderer-diagnostics file pass a direct `rustfmt --check`; unrelated formatting was not rewritten in this surgical branch.

## Matched source-only probe

Artifact: `GULA_EVIDENCE/2026-08-12/renderer-probe-matched-r1/summary.json`

The same newly built debug host was executed twice with identical isolated, repository-owned XDG trees and the same probe surface:

| Attempt | Requested policy | Renderer | Software evidence | GPU device use | Surface | Exit |
|---|---|---|---|---|---|---|
| D3D12 | `GSK_RENDERER=gl`, `GALLIUM_DRIVER=d3d12` | `GskGLRenderer` | false | `/dev/dxg` open | realized, 64 x 64 | 0 |
| Desktop GL | `GSK_RENDERER=gl`, automatic Mesa driver | `GskGLRenderer` | true | no `/dev/dxg` | realized, 64 x 64 | 0 |

No `limux.sock`, `session.json`, or surviving probe process existed after either attempt. This proves that the probe distinguishes the two renderer outcomes on this WSL machine without creating terminal children. It does not measure memory or CPU and does not prove that the D3D12 backend is production-safe for the restored daily-driver workload.

## Measurement provenance

- Resolved executable: `/home/riche/MCPs/limux/target/debug/limux`
- Version output: `limux-host 0.2.3 (ffafacb74e40-dirty, debug) install-id=none channel=stable`
- Executable SHA-256 at capture: `c88e50170431f9119a4537eca00709d51569c677e3c6df255f1f4c07ef0be72f`
- Summary SHA-256: `5c1e6efb68041835a6caf3aac9dcc9b50fbde5b2cf8c3da6f8aabae234ca8197`
- Capture timestamp: `2026-08-12T10:22:01-04:00`
- Workload: one hidden probe application window and one 64 x 64 GTK `GLArea`; no Ghostty app or surface, terminal process, Limux socket, or restored session.

Because the debug binary reports a dirty source tree, the evidence is `PROVENANCE-INCOMPLETE` for comparative or production conclusions. It is adequate only as a local mechanism check against the exact working-tree bytes under review.

## Remaining hard gate

Activation is blocked on an embedded Ghostty owner/upstream change. The smallest compatible API shape is a per-surface removal tombstone, such as `ghostty_env_var_s.value == NULL`, applied to the child `EnvMap` after the process environment is copied and before `termio.Exec` starts the command. Limux can then pass tombstones only for keys named by `LIMUX_AUTO_RENDERER_ENV`.

Rejected substitutions:

- empty overrides leave `KEY=` present;
- an `env -u` command wrapper changes shell selection, integration, restored-command, and wait semantics;
- process-global `unsetenv` after GTK initialization is not sound once worker threads may read the Unix environment;
- unconditional scrubbing would erase explicit user renderer policy.

The detailed source proof is `GULA_EVIDENCE/2026-08-12/RENDERER_OWNED_ENV_SEAM_SUBAGENT.md`.
