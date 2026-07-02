# Hermes MiniMax Direct Review

Slot: minimax-sequencing
Model: MiniMax-M3
Provider: minimax
Generated: 2026-06-30 17:25:57 EST

```text

session_id: 20260630_172558_bd75dc
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P0] Plan §"Cursor UI" / §"Control Interface" — no canonical method-name registry. The plan lists `workspace.select { present: true }` vs. `window.present` vs. `cursor.workspace_present` as parallel options and leaves `cursor.pane_create_empty` vs. `pane.create` undecided. Before drafting code, pin each v1 method to exactly one name with a one-line rationale; otherwise the Rust allowlist and the JS request builders will drift apart.

2. [P0] Manager Synthesis §"Rerun Gate" — the external adversarial-review requirement (GLM/MiniMax/Kimi wave) is still open, but the plan §"Review Gate Status" downplays this to a footnote. Make the gate explicit and blocking: a "Ready to Implement" precondition line that requires the rerun output to be synthesized into this directory, and a date stamp for the next attempt.

3. [P1] Plan §"Socket Resolution" — ordering mixes user intent (setting, env) with system fallbacks (`/tmp/limux.sock`) without distinguishing "explicit" from "ambient." Define what happens when `LIMUX_SOCKET` points at a stale runtime (parent died, pid reused). Item 4–6 currently allow silent attach to a wrong host. Bind socket identity to the `system.identify` token and require the user to acknowledge a mismatch before the tree populates.

4. [P1] Plan §"V2: Attach Mode" + Manager §"Native Review Synthesis" — the `REWORK BEFORE PRD` marker is correct, but v2 still has no formal exit criteria for when the PRD is ready. Add a short "v2 PRD-ready checklist" (attach lifecycle doc, resize-authority spec, input taxonomy table, multi-client matrix) with named owners, so v2 does not drift into v1 implementation by accident.

5. [P1] Plan §"Architecture / Extension Location" — `integrations/cursor-limux/` lives outside the Cargo workspace and outside any test harness in `./scripts/check.sh`. Add an explicit integration in `scripts/check.sh` (or a sibling) that runs `node --test` on the extension sources, so CI catches regressions even when no host Cursor is available.

6. [P1] Plan §"Limux Host UI / Launcher behavior" — `cursor <folder>` is named as default without documenting how the host resolves the `cursor` binary (PATH search, configured override, error path). Spell out resolution order and the failure-mode notification; today `which cursor` in a WSL/Remote shell may be empty.

7. [P2] Plan §"Cursor folder launch" — accepts only `file:` folders in the same Linux env, but does not state what "same environment" means in WSL (distro path vs. `/mnt/c` Windows path vs. WSL UNC). Add a one-paragraph environment check rule so the rejection logic is testable.

8. [P2] Plan §"Tests And Verification" — no negative tests for the launcher's argv construction (e.g. folder path containing spaces, `--`, or a leading dash). Add at least three.

Missing Evidence:

- Concrete list of which existing Limux methods already cover `workspace.list` folder metadata and `surface.read_text`. Plan assumes coverage; brief evidence (`rust/limux-control/src/*` or core dispatcher types) was not attached.
- A diff or pointer showing `system.identify` / `health` currently returns host/build identity at the fidelity the plan assumes; Manager mentions a "new `system.identify` equivalent" as if it may not exist.
- Confirmation that `Limux Pane IDs` (u32) and `surface_id` (`pane_id:tab_id`) survive a JSON round-trip into a Cursor TreeView without coercion rules.
- WSL/Remote live-test result: tree renders, refresh works, and focus reaches the right workspace under Wayland at least once.

Recommended Plan Changes:

- Add §"V1 Method Registry" locking names 1:1 with Rust handlers.
- Add §"Implementation Preconditions" with the rerun-gate synthesis as an explicit blocker and a date.
- Promote socket identity binding to the top of the Security section; require user confirmation on mismatch.
- Add a v2 PRD-ready checklist with owners.
- Wire extension tests into `./scripts/check.sh` via `node --test`.
- Specify `cursor` binary resolution and argv negative tests.
- Add a WSL environment-path rule for folder eligibility.

```

Exit status: 0
