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

## 5. Packaging deletes reach end-user machines as root (SEVERE — separate doc)

Backlog #12 audit result, written up in full at
`docs/LIMUX_PACKAGING_DELETE_AUDIT_2026-07-29.md`. Summary: of 24 delete calls in
`scripts/package.sh`, the builder-scope ones are clean, but the **generated `install.sh`
and the `.deb` `DEBIAN/postinst`** delete files in `/usr/local` **as root on an ordinary
`dpkg -i`** — destroying a user's source-built install — and the legacy-host heuristic
matches essentially any GTK binary named `limux`, which it executes before deleting. Same
defect class as the rest of this cycle (delete where archive was correct), pointed outward.

**Lane: limu** (owns install/packaging). The write-up is authorized; the shipped-behavior
change is **operator-gated** because it alters what lands on user machines.

## 6. Retired-session branch inventory — NO stranded work (closed, no action needed)

Thirteen local branches from retired sessions (`hamo` ×5, `lifo` ×7, `nato` ×1) reported
as ahead-of-upstream or upstream-gone. Full content comparison against `origin/main`:

| Group | Branches | Unpushed content | Verdict |
|---|---|---|---|
| `hamo/*-20260719` | 5 | one local **merge commit of their own already-merged PR** (#74–#78) each | zero unique content |
| `lifo/*` ahead | 4 | real source commits (`split_tree.rs`, `control_bridge.rs`, `layout_state.rs`, verification docs) | squash-merged; added lines present on main, the few misses are **refactored forms** (`drain_children_with_progress` at `split_tree.rs:397`; `READ_ONLY_FALLTHROUGH_METHODS` → `is_read_only_fallthrough` + two named tests) |
| upstream-gone | 4 | 16 / 3 / 2 / 20 commits | **every touched file present on main** (26/3/1/10, zero absent) |

**Conclusion: nothing is stranded.** Deleting these refs would also reclaim ~zero disk (a
branch is a ref; the objects are shared), so there is no reason to delete them — the
inventory exists to answer "is anything lost?", and the answer is no.

### Instrument-error note (worth keeping)

The first pass reported two files as ABSENT-FROM-MAIN (`scripts/xvfb-smoke-test.sh`,
`docs/cmux-parity-roadmap-20260706.md`). Both are **on main**. Cause: `for f in $files`
in **zsh does not word-split** an unquoted parameter, so the loop ran once with the entire
newline-joined blob and `git cat-file -e` failed on the nonsense path. This is the *same*
zsh gotcha already recorded in `docs/LIMUX_ORPHAN_STAGING_MANIFEST_2026-07-29.md`, hit
again by a different session four hours later. Caught only because a second reading
disagreed with the first — a single measurement cannot reveal its own instrument error.
