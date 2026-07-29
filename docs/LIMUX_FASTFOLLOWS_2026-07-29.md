# Fast-follows from the 2026-07-29 PR cycle (repo issues are disabled)

**Created by:** Claude Code (`fire` · limux lane · session `37f828e0` · Claude Fable 5)
**Date:** 2026-07-29 12:05 EST
**Purpose:** Durable home for the non-blocking findings from the Codex bot reviews of
PRs #102/#103/#104 and the gate runs, since GitHub issues are disabled on this repo.
All were triaged fast-follow (P2, none blocking) during the merge cycle.

## 1. prune-reviewed-runtimes: bound `--keep` count before Bash arithmetic

Bot P2 on #102 (merged `59876fa`). `--keep-reviewed`/`--keep` validates digits-only, but
a value beyond Bash's signed 64-bit range wraps negative in the retention comparison — an
absurdly large keep-count retains ZERO candidates and archives every unprotected install,
the opposite of the request. Fix: reject values above a sane cap (e.g. >6 digits) at
validation time. **Lane: limu.**

## 2. prune-reviewed-runtimes: interpreter-startup TOCTOU in the active-process scan

Bot P2 on #102. If an old installed launcher has just started concurrently with an
upgrade, `/proc/<pid>/exe` points at `/usr/bin/bash` until the launcher execs
`libexec/limux-cli`. The installer repoints the lane symlink before the prune scan, so in
that window the old root has neither launcher-link nor active-process protection and can
be archived out from under a starting process. Consider a second settle-then-rescan pass,
or also matching `/proc/<pid>/cmdline` against launcher paths. **Lane: limu.**

## 3. agent-hook debug log rotation: serialize across concurrent hook processes

Bot P2 on #103 (merged `457638a`), rotation near `rust/limux-cli/src/main.rs:1866`. Two
hook subprocesses can both stat a full active log before either rotates; the loser can hit
NotFound and skip its append, or rename the recreated active file over `.1` and discard a
retained generation. Bounded consequence (a lost debug-log generation) — hence
fast-follow. Fix: flock the rotation critical section. **Lane: fire.**

## 4. limux-ghostty-sys: shared-target cache poisoning via compile-time CARGO_MANIFEST_DIR

Found during the 2026-07-29 gate runs (two deterministic exit-101s). `build.rs` reads
`env!("CARGO_MANIFEST_DIR")` at COMPILE time, baking the absolute checkout path into the
build-script binary. With a shared cargo target dir, a build from an ephemeral worktree
leaves a build-script binary pointing at a path that dies with the worktree — and cargo
can later select that poisoned binary from a different checkout. Symptom: `canonicalize()`
panics "libghostty not found" while `ghostty/zig-out/lib` is fully present.

Evidence: `strings` on `target/debug/build/limux-ghostty-sys-66a6f387*/build-script-build`
contained `/tmp/limux-limu-reviewed-retention-20260729/...` (limu's removed worktree);
`cargo clean -p limux-ghostty-sys` + rebuild went green (685 tests, exit 0). This
partially reframes the historical "flaky exit-101 under load" pattern.

Fix: read `std::env::var("CARGO_MANIFEST_DIR")` at build-script RUNTIME (cargo sets it for
build-script execution too) — one line, makes the binary relocation-proof. **Lane: any.**
