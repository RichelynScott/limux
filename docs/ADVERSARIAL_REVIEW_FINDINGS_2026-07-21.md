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
