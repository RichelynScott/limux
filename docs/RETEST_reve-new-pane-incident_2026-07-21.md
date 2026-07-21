# Retest — reve's new-pane incident (2026-07-19), against main @ `b919ecb`

**Created by:** Claude Code (tutu · LIMUX_MGR · cd1a39d7)
**Date:** 2026-07-21 22:5x UTC
**Purpose:** Retest reve's 2026-07-19 `new-pane` incident (filed against legacy
v0.2.2 `main-1005f58d`) against current main, and record a source-level root-cause
trace for defects 1+2 that reve hypothesised but could not verify black-box.

Source: `LIMU_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md`
(note: my earlier handoff cited this as `LIFO_INBOX/` — wrong path, corrected.)

---

## Verdict summary

| # | reve's defect | Verdict |
|---|---|---|
| 1 | New pane's terminal never initializes; `--command` never runs | **LIKELY STILL OPEN** — concrete mechanism identified, NOT runtime-verified |
| 2 | `pane.create` reports false timeout while succeeding | **LIKELY STILL OPEN** — same root cause as 1; mechanism identified |
| 3 | `send-key` rejects every key name | **MISDIAGNOSED — real defect found, different from the one reported.** Fix landed |
| 4 | `read-screen --help` dumps another agent's pane | ✅ **FIXED** — PR #82 (`08abec1`) |

**I cannot run the GUI, so nothing below is runtime-verified.** Everything is
source-level reasoning, labelled as such. Three confident root causes were
asserted and proven false in this repo on 2026-07-21; I am not adding a fourth.

---

## Defect 4 — FIXED (PR #82)

`read-screen` fell through to `surface.read_text` with no target, and the
server's global-focus fallback returned another agent's pane content. It was
also the only one of read-screen/send/send-key with **no `LIMUX_WORKSPACE_ID`
fallback**, so it defaulted to global focus unconditionally.

PR #82 added an explicit `--help` path and a `LIMUX_WORKSPACE_ID` default
matching `send`/`send-key`. reve's suggested hardening ("reject unknown flags,
or require explicit target") is what shipped.

---

## Defect 3 — reve's diagnosis was wrong; the real bug is worse

reve reported "every key name I tried was rejected" and suggested **documenting
the accepted key vocabulary** in `send-key --help`.

**The vocabulary was never the problem, so that fix would not have helped.**

Trace:
- `send-key` → `handle.send_key(&key)` (`terminal.rs:405`).
- `send_key` parses via `NormalizedShortcut::parse` (`shortcut_config.rs:871`),
  which accepts `<ctrl>`/`<control>`/`<shift>`/`<alt>`/`<option>`/`<meta>`/
  `<super>`/`<cmd>`/`<command>` prefixes plus a key name.
- `normalize_runtime_key` (`shortcut_config.rs:1611`) maps `return` → `enter`,
  `esc` → `escape`, `pageup` → `page_up`, etc.
- `runtime_key_to_gtk_key` then maps `enter` → `Return`, `escape` → `Escape`,
  `tab` → `Tab`, arrows to `Left`/`Right`/`Up`/`Down`, function keys upper-cased.

So **`enter` is a valid key name and maps correctly to GTK's `Return`.** reve's
very first attempt should have worked.

**The actual defect:** `send_key`'s FIRST statement is

```rust
let Some(surface) = *self.surface_cell.borrow() else { return false; };
```

An **unrealized surface returns `false`** — the identical failure value used for
a genuinely unparseable key. The caller (`window.rs:6548`) maps any `false` to
`invalid_params: "unsupported key"`.

`send_text` has the *identical* first check but its caller
(`surface_send_text_response`, `window.rs:561`) correctly reports
`conflict: "terminal surface {id} is not ready for text input"`.

**Same underlying condition, two different diagnoses.** That is exactly why
reve saw `-32009 not ready` from `send` and `-32602 unsupported key` from
`send-key` on the same broken panes, and reasonably concluded the key names
were wrong. He was testing against the defect-1 panes whose terminals never
initialized.

This is the same defect class nava reported in hcom the same day: *the binary
tells the operator something that sends them chasing the wrong problem.*

**Fixed** in this change: `send_key` now returns a three-state outcome
(`Sent` / `SurfaceNotReady` / `UnsupportedKey`) so the control path reports
"not ready for text input" for an unrealized surface — matching `send_text` —
and reserves "unsupported key" for actual parse failures, now including the
offending key and a pointer to the accepted syntax.

---

## Defects 1 + 2 — one mechanism, LIKELY STILL OPEN

reve wrote: *"Defects 1 and 2 may be the same root cause observed from two
angles; I did not verify that."* The source supports that hypothesis with a
concrete mechanism.

`pane.create` with `--command` runs, after the pane is created:

1. `handle.send_text(&command)` — `window.rs:6293`
2. after `PANE_CREATE_COMMAND_SUBMIT_DELAY_MS`, `handle.send_key("enter")` —
   `window.rs:6297`

**Both of these depend on the surface already being realized**, because both
functions return `false` immediately when `surface_cell` is empty.

Two distinct problems fall out:

**(a) The command is silently dropped.** At `window.rs:6293` the return value of
`send_text` is **discarded** — the call is a bare statement. If the surface is
not yet realized, the command text vanishes with no error, no retry, and no log.
That matches reve's observation exactly: pane exists, `read-screen` returns
**empty** (not `not_found`), `--command` never ran.

**(b) `pane.create` reports failure for a pane that exists.** At the other call
site, `window.rs:623`, the `send_key("enter")` result IS checked, and a `false`
becomes `pane_create_command_failure(... "could not submit Enter; the pane
already exists, so inspect current state before retrying")`. So an unrealized
surface turns into a `pane.create` error **while the pane is genuinely there** —
reve's defect 2, including why the error text is so carefully worded about the
pane already existing.

So a single condition — *surface not yet realized when the command-submit
timer fires* — produces both symptoms, from the two call sites that handle the
failure differently.

### Why I am NOT declaring this the root cause

- **Not runtime-verified.** I cannot run the GUI. Whether the surface is
  actually unrealized at `PANE_CREATE_COMMAND_SUBMIT_DELAY_MS` on reve's machine
  is precisely the unproven step, and it is the whole hypothesis.
- reve's report also shows the surface never becoming ready **18+ seconds**
  after creation, and a fixed submit delay does not explain a permanently inert
  surface. Either realization never completes at all (a deeper defect this trace
  does not reach), or something else holds it. A delay-only theory explains the
  *command loss* but not the *persistent* `-32009`.
- The 15s `pane.create` timeout reve saw is longer than any plausible submit
  delay, so the timeout itself needs separate explanation.

**A theory that explains every observation can still be wrong** — that failed
three times in this repo today. This one does *not* explain every observation,
which is itself informative: there is likely a second, deeper realization defect
underneath.

### What would confirm or kill it

1. After restart, run reve's minimal repro (§6 of his report) on current main.
2. If `--command` still fails, check whether `surface-health` reports
   `realized: false` for the new pane, and for how long.
3. If `realized` flips true after the submit delay, (a) is confirmed and the fix
   is to retry/await realization before submitting rather than firing on a fixed
   timer.
4. If `realized` stays false indefinitely, the submit-timing theory is
   **insufficient** and the real defect is in surface realization for
   programmatically-created panes — a different investigation.

### Recommended fix direction (not implemented)

Do not extend the timer — that is a race papered over. Gate command submission
on an actual realization signal, and stop discarding the `send_text` result at
`window.rs:6293` so a failed submit is at minimum observable.

---

## What is NOT claimed

- No GUI verification of anything here.
- Defects 1 and 2 are **not** declared root-caused; a mechanism is identified
  that explains the command loss and the false failure, but not the persistent
  18s+ inert surface.
- Only the defect-3 misdiagnosis fix is implemented and gate-verified.
- reve's environment was legacy channel `main-1005f58d` (v0.2.2); the code paths
  cited above are current main and may differ from what he ran.
