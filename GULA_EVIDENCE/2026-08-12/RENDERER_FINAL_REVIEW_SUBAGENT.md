# Renderer source final review

Date: 2026-08-12
Reviewer: `renderer_final_review` Codex subagent
Branch: `gula/renderer-supervisor-v2-20260812`
Fixed point: `ffafacb74e403964205be4ce29440f4eb22dc6ab`
Scope: renderer CLI/host source and `docs/verification/renderer-probe-supervisor-20260812.md` only

## Verdict: PASS

No necessary P0-P3 finding remains within the source-checkpoint contract. The renderer supervisor is fail-closed, current automatic selection remains dormant, probe isolation is preserved, fallback behavior is strict, and every post-spawn path now has an owning cleanup guard.

## Contract review

| Clause | Result | Evidence |
|---|---|---|
| Explicit renderer environment is preserved | PASS | `rust/limux-cli/src/renderer_launch.rs:114-124` selects `PreserveInherited` for explicit controls; `renderer_launch.rs:271-275` checks the declared renderer-key set. |
| Automation remains inactive without child-env removal | PASS | `renderer_launch.rs:284-289` returns `false`; the capability is required at `renderer_launch.rs:114-124`. |
| Probe is isolated from session/socket/Ghostty | PASS | Host dispatch at `rust/limux-host-linux/src/main.rs:529-535` precedes Ghostty/session/socket initialization; probe command removes inherited Limux targeting at `renderer_launch.rs:163-181`; the probe creates only GTK/GSK objects at `rust/limux-host-linux/src/window/renderer_diagnostics.rs:420-498`. |
| Malformed, timeout, failed, oversized, or non-accelerated result falls back | PASS | Strict payload acceptance is at `renderer_launch.rs:126-161`; process results map to `None` at `renderer_launch.rs:225-245`; CLI fallback is at `rust/limux-cli/src/main.rs:729-746`. |
| Every spawned probe child is owned and reaped | PASS | `renderer_launch.rs:184-192` moves the child into `ProbeChild` immediately after spawn, before stdout extraction or reader creation. `renderer_launch.rs:198-217` uses fallible `thread::Builder::spawn`, so failure unwinds through `ProbeChild::Drop` at `renderer_launch.rs:71-82`, which kill/waits and joins any reader. Normal polling, timeout, capture, rejection, and success paths remain under the same owner at `renderer_launch.rs:220-245`. |

## Closed review findings

- Post-spawn polling/wait errors: closed by `ProbeChild` cleanup ownership and Drop.
- Missing executable supervisor tests: closed by real fixed-system-binary regressions.
- Pre-guard reader-thread creation window: closed by moving the child into `ProbeChild` before stdout extraction and using fallible `thread::Builder::spawn`.

## Verification performed on final bytes

- `cargo test -p limux-cli renderer_launch -- --nocapture`: PASS, 10 passed.
- `cargo test -p limux-host-linux renderer_probe -- --nocapture`: PASS, 1 passed.
- `rustfmt --edition 2021 --check rust/limux-cli/src/renderer_launch.rs rust/limux-host-linux/src/window/renderer_diagnostics.rs`: PASS.
- `git diff --check` over tracked changes in the six reviewed paths: PASS; the new files were compiled and directly rustfmt-checked.
- Executable subprocess regressions cover successful JSON, malformed JSON, non-zero exit, oversized stdout, timeout kill/reap, and `/proc/<pid>` absence after each return (`renderer_launch.rs:457-499`).
- Tests use fixed system binaries and create no files or `/tmp` paths.
- Vendored Ghostty remains outside the renderer source diff.

## Non-blocking observations

- Probe stderr remains discarded and ordinary rejection reasons collapse to `None` (`renderer_launch.rs:183-250`). A bounded typed reason could improve field diagnostics, but deleting this observation does not leave the source-checkpoint contract unmet.
- Renderer-control inventories remain duplicated between `renderer_launch.rs:8-18` and `renderer_diagnostics.rs:12-21`; `MESA_GL_VERSION_OVERRIDE` appears only in the CLI list. This is maintainability debt, not a current contract failure.

## Scope boundary

This PASS is for the dormant source checkpoint, not runtime activation or production performance. Keep `child_env_removal_supported()` false until the separate Ghostty terminal-child environment-removal seam lands and is verified. The dirty debug probe evidence remains `PROVENANCE-INCOMPLETE` for comparative or production conclusions. No install, restart, runtime promotion, performance claim, or live daily-driver mutation was reviewed or authorized here.

## Methodology

Applied the two-axis `$code-review` method against the pinned fixed point, `$limux-use-guide` for the runtime boundary, `$karpathy-guidelines` for surgical scope and necessity, and `$systematic-debugging` for lifecycle-path tracing.
