**Created by:** Claude Code (tutu · cd1a39d7)
**Date:** 2026-07-21
**Purpose:** HIGH-priority operator-facing bug lead for limu (LIMUX_MGR). Diagnosis captured durably (hcom flaky under db-lock). Operator-reported, daily pain.

## For: limu (LIMUX_MGR)

## Symptom (operator, 2026-07-21)
Running OMP (a Claude agent) inside a Limux pane: the viewport **yanks to the bottom + flashes every ~0.5s**, so the operator cannot scroll up to read. Operator: "I thought this was fixed."

## Diagnosis (tutu, read-only triage — did NOT build/test; that's yours + I'm protecting lifo's uncommitted WIP)

**Prime suspect: PR #59.** Two commits landed under it together:
- `3a182f6 feat(ui): add live workspace status header (#59)`
- `fc23ac2 Add terminal scrollbar support (#59)`

**Mechanism (hypothesis, not proven):**
- `rust/limux-host-linux/src/header_status.rs`: `RESOURCE_SAMPLE_INTERVAL = 1s`, plus an hcom query (`HCOM_QUERY_TIMEOUT = 2s`). The live header refreshes on these.
- `rust/limux-host-linux/src/terminal.rs:1856-1862`: `scrollbar_adjustment.connect_value_changed` emits `scroll_to_row:{row}`. If the adjustment's value/upper is reset on each header refresh or terminal output, it fires → viewport jumps. The flash = full redraw per refresh.
- **"The fix" the operator remembers likely = the existing guard** `scrollbar_adjustment_needs_update` (terminal.rs:1089/1136) + test `scrollbar_adjustment_skips_redundant_updates` (terminal.rs:3871). It's real but is NOT catching this case — either incomplete, regressed, or bypassed by a different code path (header-driven redraw vs output-driven adjustment).

**Aggravating interaction (worth checking):** the header's hcom query is hitting the **active fleet-wide db-lock contention** (see §8 of my succession onboarding doc). Slow/erratic hcom responses → erratic header updates → more redraws → more scroll-yank. So this may present WORSE right now than it will once db-lock eases — but the scrollbar-feedback bug is real independent of db-lock.

## Suggested first checks for you
1. Repro with OMP (or any high-output process) in a pane; watch whether the yank cadence matches the header refresh (1s) vs terminal output.
2. Read `scrollbar_adjustment_needs_update` — does it guard the header-refresh path, or only the output path? Likely gap there.
3. Try gating/disabling the live status header (no disable flag exists today — I searched) to confirm it's the header vs the scrollbar. A temporary env/config gate for the header would ALSO give the operator immediate relief.
4. Check whether the header's hcom query blocks/retries under db-lock and forces redraws.

## Priority
HIGH — operator-facing, daily, blocks reading OMP output in a pane. Comparable to word-wrap #29. Recommend a new task on the reconciled master tag (I did NOT write TaskMaster — you're mid-reconciling the uncommitted state; please add it).

## Note
No clean one-flag workaround exists today. Practical interim: run OMP in a plain terminal (outside a Limux pane) until fixed, OR you ship a header-gate. Operator would strongly prefer the latter.
