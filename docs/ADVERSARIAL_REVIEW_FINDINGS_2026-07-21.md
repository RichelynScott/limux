# Adversarial review findings — `31a9431..05006b6` (2026-07-21)

**Created by:** Claude Code (tutu · LIMUX_MGR · cd1a39d7), recording an Opus 4.8 adversarial subagent's report
**Date:** 2026-07-21
**Purpose:** Durable record of the standing adversarial review of the five PRs merged 2026-07-21, which shipped with ZERO external review (Codex bot down fleet-wide). Remediation status tracked per finding.

**Method (the reviewer's, verbatim in substance):** isolated worktree at `05006b6`,
baseline gate green (620 passed). **Five hunk-reversion experiments** run and re-tested —
i.e. it reverted each fix and re-ran the suite to see whether any test noticed. Ghostty
source read as ground truth.

---

## Remediation status

| ID | Severity | Finding | Status |
|---|---|---|---|
| H-1 | HIGH | Cross-lane disclosure fix misses `agent-team`'s real topology | ✅ **FIXED** PR #86 |
| H-2 | HIGH | Every read-screen scoping test passes with the fix deleted | ✅ **FIXED** PR #86 |
| H-3 | HIGH | Scrollbar + #84 wiring entirely untested | ⬜ **OPEN** |
| M-1 | MED | Scrollbar fix's stated invariant is FALSE — config reload mutates it at runtime | ⬜ **OPEN — real residual bug** |
| M-2 | MED | `-uno` turned a loud-wrong attestation into silent "verified clean" | ✅ **FIXED** PR #86 |
| M-3 | MED | `-uno` also hides untracked content inside submodules | ⬜ **OPEN** |
| M-4 | MED | `SocketControlMode` strict mode FAILS OPEN on `limux-only` | ✅ **FIXED** PR #86 |
| M-5 | MED | 100 ms settle timer has a live drop path | ⬜ **OPEN** |
| L-1 | LOW | `pipe-pane --help` silently pipes an empty stream | ✅ **FIXED** PR #86 |
| L-2 | LOW | `read-screen` doesn't reject option values starting with `-` | ⬜ OPEN |
| L-3 | LOW | #83 test comment inverts the security direction of `cmuxOnly` | ⬜ OPEN |
| L-4 | LOW | `hook_session_id_from_transcript` accepts unvalidated paths (SPECULATIVE) | ⬜ OPEN |

---

## THE HEADLINE: 4 of 5 behavioural fixes are test theater

The reviewer reverted each fix and re-ran the suite. **Four of the five survive a full
revert with a green suite.**

| Fix | New tests | Reverted the fix → | Verdict |
|---|---|---|---|
| #82 read-screen `--help` interception | 2 | 122/122 pass | **theater** |
| #82 read-screen workspace scoping | 1 | 122/122 pass | **theater** (fixed in #86) |
| #82 `hook_session_id` ordering | 2 | 120/122, **2 fail** | **load-bearing** |
| #82 scrollbar presentation wiring | 2 | 422/422 pass | **theater** |
| #82 scrollbar user-interaction grace | 0 | — | **uncovered** |
| #84 grid predicate (pure fn) | 6 | **fails on revert** | **load-bearing** |
| #84 deferral wiring + settle timer | 0 | 422/422 pass | **uncovered** |
| #85 `git_tracked_dirty` | 0 | — | **uncovered** |

The pure-logic helpers are tested; **the wiring that actually uses them is not.** A test
that passes with the fix deleted provides zero protection, and given this repo's
documented history of confident-but-false self-verification, that is the single most
important remediation item.

---

## OPEN — details worth acting on

### M-1 (MEDIUM) — the scrollbar fix's invariant is false; a live residual reflow path

The test comment at `terminal.rs` asserts: *"Config is constant for the surface lifetime,
so it cannot oscillate; this is the one case where dropping out of layout is safe."*

**That is false.** `terminal.rs` (inside `GHOSTTY_ACTION_RELOAD_CONFIG`) does
`CURRENT_SCROLLBAR_ENABLED.store(load_scrollbar_enabled(config), Relaxed)`.

**Concrete failure:** user is scrolled back → a Ghostty config reload flips `scrollbar`
to false → next `SetScrollbar` selects `Hidden` → `set_visible(false)` → the Box child
loses its allocation → GLArea widens → ghostty column change → `PageList.resize` →
`resizeCols` → `switch (self.viewport) { .pin => if (self.pinIsActive(...)) self.viewport
= .active }` (`ghostty/src/terminal/PageList.zig:996-1000`) → **viewport yanked to the
bottom. Exactly the bug #82 fixed.**

Mitigating: `apply_scrollbar_presentation` is the **only** remaining `set_visible` caller
on the scrollbar, so config reload is the *one* residual path — but it is live, not
impossible.

**This matters because scroll-yank is the operator's own reported symptom.** Fix
direction: layout participation must not change while the viewport is scrolled back —
defer the reload's layout change until the viewport is at the bottom.

### M-3 (MEDIUM) — `-uno` masks untracked content inside submodules

Built a real superproject+submodule to check:

```
submodule TRACKED file modified  -> porcelain " M sub" | -uno " M sub"  (good)
submodule UNTRACKED file added   -> porcelain " M sub" | -uno ""        (masked)
```

The reviewer *falsified its own strongest instance*: `ghostty/zig-out/` is gitignored, so
`libghostty.so` never registered as dirty anyway. Residual class is still real: an
untracked `.cargo/config.toml` or `rust-toolchain.toml` changes the emitted binary while
now reporting `dirty=false`.

### M-5 (MEDIUM) — settle timer drop path

`clear_pending` now also cancels the settle timer and discards the parked snapshot.
`gl_area.connect_unrealize` calls it, and GTK unrealizes during reparenting on splits —
the same interaction the 100 ms window was sized against. A second split inside that
window drops the final size. Recovery via `connect_realize` → `refresh_surface_display`
pushes the allocation to ghostty but **never updates `coalescer.last_applied`**
(pre-existing), so a later size-allocate equal to the stale value is skipped entirely,
leaving ghostty's grid mismatched with the GLArea. Untested (H-3).

*Ruled out by the reviewer:* no `SourceId` double-remove, no use-after-free.

### L-2 / L-3 / L-4

- **L-2** `read-screen --workspace --scrollback` sends `workspace_id:"--scrollback"` →
  confusing not-found. Its sibling `build_close_surface_request` rejects `-`-prefixed
  values; this does not. Not a disclosure.
- **L-3** The #83 test comment calls `cmuxOnly` a *"hostile inherited legacy alias"* —
  **inverted**; it is the *stricter* mode, and the preview runner overrides operator
  hardening. Mitigated (isolated socket + `XDG_*`), but the comment mis-frames the
  security direction for the next reader. *(I wrote that comment. It should be corrected.)*
- **L-4 (SPECULATIVE, so labelled)** `hook_session_id_from_transcript` returns
  `Path::file_stem()` with no validation and now outranks the environment, so a payload
  carrying `"transcript": "/var/log/agent/session.log"` would collapse every session to
  `"session"` — a worse misattribution than the one #82 fixed. The reviewer **could not
  demonstrate this against a real deployment** (Claude Code payloads always carry
  `session_id`, which still wins) and said so. No traversal risk.

---

## VERIFIED CLEAN — the reviewer tried to break these and failed

Recording these because "we checked and it held" is worth as much as a finding.

- **`surface_grid_axis_can_change` (#84) is correct.** Ground-truthed against ghostty
  (`renderer/size.zig`: `grid = GridSize.init(screen.subPadding(padding), cell)`,
  `GridSize.update` = `@max(1, trunc(screen/cell))`, `subPadding` saturates). Hence
  `slack = padding + r` with `r ∈ [0, cell−1]` — the code's exact bound.
  **Brute-forced ghostty's model over cell∈[1,12] × padding∈[0,39] × current∈[1,199] ×
  next∈[1,199]: zero cases where the predicate said "cannot change" but the grid changed.**
  All underflow paths guarded; the `>` vs `>=` boundary is right.
- **`--surface` cannot cross a workspace.** The server resolves the workspace first, then
  searches only within it. The bypass does not exist. (`--workspace` crosses trivially —
  that is H-1, and it is not access control.)
- **#82's reflow root cause is real** (`PageList.zig:996-1000`). Nuance: the comment's
  *"a column change is a reflow"* is narrower than ghostty — `resizeWithoutReflow` does the
  same viewport reset on **row growth** too (`PageList.zig:2125-2131`, *"This effectively
  'pulls down' scrollback"*). Harmless for the width-only scrollbar case; it *strengthens*
  #84's row-axis deferral.
- **#85 achieves its stated goal** in the live checkout.

---

## Cross-cutting note

There is **no connection→workspace binding anywhere**; socket auth is uid-only
(`LocalUser` default) and workspace/surface values are caller-supplied. A same-uid process
can target any workspace, `--scrollback` included. The H-1 fix removes an *accidental*
default; **it is not an access control**, and neither code comments nor the limux skill
should imply otherwise.

---

# SECOND REVIEW — PR #88 `unsafe` fd logging (`d8e7648`)

**Verdict: NO — NOT SAFE TO INSTALL.** Commissioned because the implementing
agent flagged its own gap (*"unsafe fd code is unit-tested but had no
cross-family adversarial review"*, *"No GUI run"*). It found **no fd-corruption
defect** — the `unsafe` code is correctly paired and leak-free — but the change
**regresses** the thing a crash log exists for.

## H1 (HIGH) — stderr written shortly before exit is silently discarded

`install_bounded_stderr` **detaches** the drain thread: `if let Err(error) =
drain` (~`host_log.rs:891`) moves the `io::Result<JoinHandle>` and drops the `Ok`
variant. Nothing joins or flushes it. `std::process::exit()` kills the thread
wherever it sits in its 25 ms `STDERR_IDLE_TICK` sleep; the 64 KiB pipe buffer
and `pending` die with it.

**Confirmed a REGRESSION, not a pre-existing gap.** Pre-#88 `main.rs` did
`libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO)` — fd 2 *was* the log file,
writes were synchronous, loss was impossible. Verified independently against
`1254072`.

Measured (probe calling the real `install_host_stderr_log()`): 200 `eprintln!`
then `exit(3)`, 20 runs → **3 lossy, one captured 0 of 200 lines**. Panic path,
15 runs → panic message **absent in 2**.

**Failure:** GTK fatal/assert/panic writes the diagnostic, process exits, drain
thread dies mid-sleep → the log ends *before* the crash. Truncated log, no cause.

## H3 (HIGH) — the entire P2 fd-reservation layer is unreachable

Rust std runs `sanitize_standard_fds()` before `main`, so fds 0/1/2 are always
open even when launched `2>&-` or `0<&- 1>&- 2>&-` — probed: all three report
ino 4, mode `020666` (`/dev/null`), and the first `open` lands on **fd 4**. So
`pipe2` can never return fd ≤ 2 in this binary and the documented mechanism
**cannot occur**.

**This is the THIRD wrong mechanism in this one story** — the handoff's original
version, the implementer's correction, and now the premise itself. Each
explained the evidence available at the time.

Also unwired: neutering all four production call sites → **448 passed, 0
failed**. ~90 lines of `unsafe` fd surgery, dead in production, wiring untested.

## H2 — the `O_CLOEXEC` fix is test theater

Removing `O_CLOEXEC` from the installer's pipe → **448 passed, 0 failed**. Both
A1 tests build their *own* pipes and never call `install_bounded_stderr`. The fix
is genuinely correct (proved by dumping a real child's `/proc/self/fd`: without
it the child inherits `4 -> pipe:[...]` read-side) — correct fix, zero defence.

## M1–M2

- **M1** Retained logs are never pruned. With production constants, ≥10 retained
  files → `install failed: retained logs leave no budget for a new bounded active
  log`, permanently. Doctor does not check the retained budget.
- **M2** A dead drain thread now silently **kills** the host rather than hanging
  it: `eprintln!` panics on EPIPE (probe rc=101), and the panic message goes to
  the same dead pipe, so the host dies with no diagnostic. Impossible pre-#88.

## Verified correct (evidence given)

A1 and A2 are **load-bearing** (reverting A1 → 2 failures + a full 15.01 s block,
independently reproducing my own result). **No data race** — writer/state move
wholly into the drain thread; the only shared object is the kernel-synchronised
pipe. **EINTR** handled. **No fd leaks or double-closes** across all five error
paths. `O_CLOEXEC` completeness verified via child `/proc/self/fd`.

**Hypotheses it killed:** cap marker overrunning the cap (headroom correct);
double-close in `reserve_standard_fds` (unreachable *and* paired); fd-test
suite flakiness (30 clean runs).

## One FALSE POSITIVE — worth recording

**L5 claimed `CLAUDE.md` still warns about a failing
`hook_session_id_falls_back_to_transcript_stem`.** It does not — that was fixed
in PR #86, and the file says "**Resolved**" and explains it. The reviewer matched
the test name without reading the surrounding text. **Even a rigorous adversary
produces false positives; verify its claims too.** Its HIGH findings came with
probe data (20-run trials, `/proc/self/fd` dumps, hunk reversions) and held up.
