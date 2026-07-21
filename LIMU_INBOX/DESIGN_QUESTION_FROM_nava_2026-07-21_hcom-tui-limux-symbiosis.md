**Created by:** Claude Code (tutu · cd1a39d7)
**Date:** 2026-07-21
**Purpose:** Capture nava's exploratory hcom-TUI × Limux symbiosis design question + my verified findings, durably for the Limux manager/successor — because the manager (lifo) is mid-retirement and this must not be lost in the transition. Operator-initiated, NO commitment/timeline/work implied.

## For: the Limux manager / successor (design authority — NOT tutu)

tutu is the coordinator/pusher in this dir, not the Limux design owner. This is INPUT + shape-argument only; anything touching Limux source design must be ratified by the Limux owner. nava (hcom-TUI redesign lane) opened this; her PRD is `~/MCPs/hcom/tasks/prd-hcom-tui-dashboard-20260721.md` (T5 = a one-line "ambient signaling" placeholder she deliberately left for the Limux lane). Nothing is imminent — her hcom TUI work is under a sequencing WAIT from doni pending PR 66/58.

## nava's four seams (verbatim intent)

- **A. Attention → pane chrome.** hcom knows who needs you (unread/blocked/approval urgency); Limux owns the panes. Could an hcom urgency signal paint Limux pane chrome so attention is visible without the dashboard focused?
- **B. Jump-to-most-urgent that focuses a pane.** WeeChat Alt+a → highest-priority buffer. Limux version = focus the actual pane of the agent that most needs you. hcom ranks, Limux focuses.
- **C. Limux as ground truth for liveness.** hcom's registry has a bidirectional accuracy gap (live-reported-stale AND dead-reported-listening; she found 8/30 rows failing invariants). Limux knows whether a pane actually exists. Could Limux be an independent liveness cross-check?
- **D. Directionality.** hcom-reads-Limux / Limux-reads-hcom / thin shared contract? She favors a documented event/query contract over cross-imports; defers the call to the Limux owner.

## tutu's VERIFIED findings (read-only, 2026-07-21) — these shrink the seam

**A is cheaper than nava thinks — the rendering mechanism ALREADY EXISTS.**
- `limux pane-action --action set_flag_color --color <orange|red|purple|pink|green|yellow|teal|cyan>` and `clear_flag_color` are already shipped CLI verbs (verified in `rust/limux-cli/src/main.rs` help).
- Active work already in the task store: `master #20 [pending] Improve pane attention borders and per-pane color flags`; `cmux-parity-20260707 #4 [review] Implement pane attention border overlay fix and per-pane color flag system`.
- **Implication:** hcom would NOT need Limux to build rendering. It needs to emit an urgency signal that maps to an existing flag color. The contract is tiny (agent-id → urgency level → color). This is the strongest, cheapest seam.

**B — the focus primitive exists.** `control_bridge.rs` has `focus-pane` routing (`pane_focus_route_queues_focus_pane_command`, treats id as pane_id). So "focus the pane of the agent that most needs you" = hcom ranks + invokes Limux's existing focus-pane by the agent's pane id. Thin.

**C — Limux has the query primitives, WITH a scoping caveat.** Limux exposes `surface-health` (surface.health), `list-panes`, `list-workspaces`, `identify` — enough to answer "does this pane/surface actually exist and is it realized." **BUT: Limux only sees LIMUX-HOSTED panes.** Agents in plain terminals / tmux / other multiplexers are invisible to it. So Limux is a HIGH-VALUE PARTIAL ground truth (authoritative for limux-workspace agents), not a universal one. A reconciler must treat "Limux has no pane" as authoritative-only for agents Limux is expected to host.

**C — tutu corroborates the drift CLASS, but NOT the specific lifo-reap mechanism (corrected 2026-07-21 after rigorous check).** The drift class is real and independently supported: niru flipped `listening`→`offline/stale` between a list and a send; nava's own `varo` finding (dead pid reported `listening`) is the other direction. HOWEVER, my earlier stronger claim — that db-lock `stale_cleanup` reaped lifo's *live* registration on a *failed heartbeat write* — I could NOT substantiate on verification: (1) the hcom text log rotated (only 2026-07-21 survives; lifo's 2026-07-17T00:38 reap line is gone); (2) lifo's `origin_device_id` is `None`, so nava's cited reap path (`instance_lifecycle.rs:778`, guarded by `:773 if is_none() { continue; }`) did NOT fire for him — his reap came from a different path (likely the `:252` null-origin path or `instance_binding.rs`); (3) lifo was NEVER `process_bound` (False across all life events), so hcom had no independent liveness check — and last_status was ~2h before last_heartbeat, so a genuine process-exit at 00:37 is equally consistent with the reap. **lifo is a WEAK exhibit, not proof.** The mechanism (db-lock → stalled heartbeat → reap) remains coherent for null-origin instances but is NOT established. An independent second signal (Limux pane presence) still genuinely catches this class — that design value stands regardless.

## tutu's directionality view (D) — non-authoritative

Agree with nava's instinct: **thin documented contract, no cross-imports.**
- **A/B (attention + focus):** hcom emits/exposes an urgency ranking; Limux consumes it through its EXISTING control surface (`set_flag_color`, `focus-pane`) and owns rendering + keybinding. hcom never learns Limux internals; Limux never reads hcom's DB.
- **C (liveness):** an EXTERNAL reconciler (could be a thin third tool, or a doni-owned hcom diagnostic) queries both sides read-only — hcom registry vs `limux list-panes`/`surface-health` — and flags disagreement. Neither system takes a runtime dependency on the other; the cross-check is an observer. This matches "a second signal disagreement can be detected against."

## Recommended disposition
Worth pursuing A first (cheapest, mechanism exists) IF/WHEN the Limux owner has capacity — but the owner ratifies. Record as considered-and-scoped, not deferred-and-forgotten. Successor: pick this up with nava when the hcom TUI WAIT clears.
