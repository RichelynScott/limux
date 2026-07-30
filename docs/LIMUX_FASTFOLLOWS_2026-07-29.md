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

## 7. H1 residual — explicit foreign `workspace_id` still bypasses the scoping (CRITICAL, open)

Landed in #107 (`05836c4`): bare-surface-id disclosure is closed on `surface.read_text`,
`debug.terminal.read_text`, `surface.trigger_flash`, `surface.send_text`,
`surface.send_key`, `surface.clear_history`. **Not closed:** a caller who supplies an
explicit `workspace_id` naming a foreign lane still resolves, because
`resolve_surface_target_scoped`'s first branch only asks "does this surface belong to this
workspace" — it never consults the scope. Rated CRITICAL by adversarial review.

This is the honest ceiling of a dispatcher-side fix: the dispatcher has no notion of which
workspace the **caller** is entitled to. Closing it is design-note **option (b)** —
per-connection entitlement, where an agent presents its own `LIMUX_WORKSPACE_ID` as a claim
at connect and the server rejects reads outside it. It **cannot key on uid**, because the
operator (the one legitimate cross-workspace reader) shares the agents' uid. The operator's
interactive connection needs a distinguishable "unclaimed = all-entitled" path.

**Also unverified:** `surface.focus/close/move/reorder/drag_to_split/refresh` resolve via
`find_surface_in_current_window` / `current_workspace_idx()` rather than
`find_workspace_for_surface`, and were not traced end-to-end. Likely reachable through the
same explicit-`workspace_id` bypass.

### The lesson worth keeping from this one

The first fix shipped with a stated justification — "management operations do not return
content" — that was **false on inspection**: `SurfaceState::info` serializes `text`, so
`surface.trigger_flash` returned the victim's whole scrollback for a flash-counter bump,
a *cleaner* exfiltration primitive than the read method the fix was aimed at. It took an
adversarial reviewer instructed to **refute** to surface that; the author's own tests all
passed, because they tested the thing the author already believed. A security fix's stated
rationale needs the same adversarial treatment as its code.

## 8. A WSLg display reset exits the host with a bare status 1 and no actionable message

Live incident 2026-07-30 18:11:59 (thread `limux-runtime-crash-20260730`, diagnosed
independently by `limu` and `fire`, converging). WSLg reset the display
(`Gdk-Message: Error reading events from display: Connection reset by peer`);
`xdg-desktop-portal-gtk` failed on broken pipe and Weston re-established monitors at
18:12:02. `limux-host` exited **1**. There is **no panic, no segfault, and no code
regression** — #106/#107/#108 are not implicated.

**The defect is the diagnostic, not the exit.** A GTK app generally cannot survive losing
its display, so exiting is defensible; exiting with a bare `1` is not. The operator sees
"the launcher exits, status 1" and cannot distinguish an environmental display reset from
a Limux fault — which is exactly why this consumed two agent lanes. Emitting something
like `limux: display connection lost (compositor reset) — relaunch limux` before exit
turns a multi-lane investigation into a one-line read.

Verified NOT causes (each checked at source, not assumed): stable runtime tree intact
(`bin`/`lib`/`libexec`/`share`, `libghostty.so` 28.5 MB, correctly linked, `--help` exits
0); `session.json` (60254 B) and `runtime-incarnation.json` both valid JSON — no
disk-pressure truncation; X display healthy after the event (`xdpyinfo :0` exit 0).
Post-incident reproduction: `timeout 30 limux` returns **124**, i.e. it stayed up the full
30 s — captured unpiped, so the exit code is the launcher's and not a pipeline's.

### Where the fix must go (the codebase already measured this)

**A diagnostic sited after `app.run()` is dead code on this path.** `rust/limux-host-linux/src/main.rs:555-561`
already records the finding, for a sibling problem:

> No flush call here on purpose. Exiting kills the bounded-log drain thread and discards
> the pipe buffer, but a call sited after `app.run()` does not reliably cover that:
> **measured headless, GTK terminates the process from inside `app.run()`, which never
> returns.** The flush is registered with `atexit` inside `install_bounded_stderr` instead.

So the message cannot be emitted where a reader would naturally put it. Two viable seams:
a **`GdkDisplay::closed` handler** (fires with `is_error=true` while GTK is still up — the
preferred seam, since it can name the cause), or the existing **`atexit`** pattern (already
proven here, but blind to *why* the process is exiting). Prefer the former; fall back to
the latter only to guarantee the line is flushed.

**Regression-test feasibility (honest read, per the revert-the-call-site rule in
`CLAUDE.md`):** a unit test over a message-formatting helper would be **decorative** — it
would prove the helper works while nothing proves it is reached, which is exactly the shape
that rule exists to reject. The load-bearing test is integration: bring the host up under
the existing `scripts/xvfb-smoke-test.sh` harness, kill the display, and assert the
actionable line appears in the log. That is legitimate under the escape hatch because it is
a **timeout ceiling, not a timing assertion** — it returns the moment the line lands, and
only elapses when the message genuinely never comes, which *is* the bug (same shape as
`sink_failure_does_not_block_stderr_writers_while_read_end_stays_open`). If the seam turns
out to have no injection point without a runtime refactor, **file the gap with mutation
evidence instead of forcing a flaky gate.**

Secondary: `limux doctor` reports `[warn] 4 stale sockets` (e.g. `limux-85224.sock`).
**These are self-inflicted diagnostic debris, not incident evidence** — doctor reported
`[ok] no stale Limux sockets found` *before* the reproduction above and `[warn] 4` after,
so the bounded test launches created them. Caught by `limu`, who declined to remove them
without the normal no-loss gate. Worth noting as behavior: a host that is killed (here, by
`timeout`) leaves its control sockets behind rather than cleaning up on signal.
**Lane: limu** (owns runtime/install).

### Instrument-error note (second instance in two days — same shape as item 6)

While diagnosing, `fire` reported "no host process running" from
`ps -eo pid,lstart,etime,stat,cmd | grep -i limux-host | grep -v grep`, which returned
empty, and used that to "correct" `limux doctor`'s accurate `found 1 running host` as a
false positive counting its own test launch. `pgrep -a limux-host` finds PID 65425
immediately. **The doctor was right; the ps pipeline was the broken instrument, and the
broken instrument was used to overturn the good one.** Prefer `pgrep -a` here. This is
the second measurement error in this doc's lifetime caught only because two readings
disagreed — a single measurement still cannot reveal its own instrument error, and being
the one who wrote that sentence in item 6 did not prevent committing it again.

## 9. Successor rebind after an unclean restore has no supported control path (open)

Surfaced by `limu` during the item-8 incident recovery (thread
`limux-runtime-crash-20260730`). After the unclean restore, the live hcom successor
(`limu`) **can** update the canonical Codex hook store to its real pane (264), but the
running Limux surface stays **suspended under the predecessor identity** (`lifo`), because
hot reconciliation requires the *same* session ID. The installed surface exposes **no
supported live rebind or unsuspend control method**, so a successor that legitimately
inherits a pane cannot claim it.

The failure mode is quiet and therefore worth writing down: the hook store and the surface
disagree, each is internally consistent, and nothing reports the divergence — the pane
simply stays attributed to a session that is gone.

**This needs an explicit successor-rebind design, not a workaround.** Hand-editing
`session.json` is the obvious temptation and is the wrong move: it is live operator state
(currently ~60 KB covering the real workspaces), there is no schema-checked write path for
external editors, and a malformed write costs the operator every restored pane. `limu`
correctly declined to do it. Any fix belongs in the control API — a rebind verb with the
same authorization treatment the surface-scoping work in item 7 established, since claiming
another session's pane is precisely a cross-lane authority question.

**Lane: limu** (owns runtime/session-restore), with the control-surface authorization model
overlapping item 7's per-connection entitlement design — worth solving together rather than
twice.
