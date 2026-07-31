# Limux H1 residuals plan — #124 follow-through (A–E)

**Status:** Slice A LANDED on #124 @ `c62713f` (2026-07-31). B–E remain. Still PARTIAL.
**Author:** bari / LIMUX_MGR
**Date:** 2026-07-31
**Trigger:** Operator directed: address all #124 residuals, update docs/handoff, prepare for compaction; resume next session via hcom in plan mode.
**Authoritative plan URI:** `local://limux-h1-residuals-plan.md`
**Durable copy:** `/tmp/limux-wave-briefs/reports/H1-residuals-plan.md`
**Review baseline:** https://github.com/RichelynScott/limux/pull/124#issuecomment-5148324767
**Design:** `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md`
**Scout:** `/tmp/limux-wave-briefs/reports/B.md` (+ `B-impl.md`)

---

## 0. Non-negotiable constraints

1. **No host bounce** of live PID **860496** / install `main-15ccb28ed4a8-matched-20260731` without fresh peer + operator restart ack.
2. **No live H1 disclosure probes** — static-trace + unit/integration tests only.
3. **Never flip `LIMUX_ENTITLEMENT` default away from `Off`** in the same PR that lands enforcement wiring.
4. **Do not merge messaging that "H1 is closed"** until A+B+C land and E is decided (D scoped accordingly). #124 alone = PARTIAL scaffold.
5. **Ephemeral worktree** for commits that touch shared `main`; never branch-switch shared checkout while peers may be live.
6. **Archive-not-delete**; leave untracked `AUTOPILOT_LOG.md` + `docs/research/*` alone.
7. Set `export CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude` on every commit shell.

---

## 1. Current truth (measured 2026-07-31)

| Item | Value |
|---|---|
| Shared `origin/main` | `98106c1` (Wave2 docs sync #125; product #121–#123 merged) |
| PR #124 branch | `bari/h1-option-b-entitlement-default-off-20260731` @ `2c8ad7c` |
| PR state | OPEN, MERGEABLE, CLEAN (no CI rollup) |
| Live host | PID **860496**, install `main-15ccb28ed4a8-matched-20260731` (pre-#122/#123 symbols) |
| Doctor | all `[ok]` including `stale_sockets` |
| #124 content | `limux-core` entitlement scaffold only; default-off |
| Live wire-up | **Absent** — `handle_command` hardcodes `EntitlementMode::Off`; `EntitlementConfig::from_env()` unused by accept/dispatch |
| GTK path | **Untouched** — `workspace_index_for_target` (`window.rs:1077`) + ~20 call sites |
| Discovery | `workspace.list` / `current` / `select` ungated |
| Operator signal | **Undecided** (B.md §2.5) |

### Known logic bug on #124 (blocking for RequireClaim)

Gate order in `gate_workspace_id_param` and `resolve_surface_target_scoped`:

1. `allows_workspace(id)` — under `RequireClaim` + unclaimed → **false**
2. then `record_claim(id)` — never reached on first natural request

Test `entitlement_mode_flag_pins_fail_open_vs_fail_closed` papers over via out-of-band `record_claim` before re-dispatch.

---

## 2. Goal

Close H1 option (b) as a **real** entitlement boundary in ordered slices A→E, keeping default-off until operator-signal (E) is chosen. Produce mergeable PRs with honest PARTIAL/COMPLETE labels, updated handoffs, and TaskMaster notes — without bouncing the live host or running disclosure probes.

---

## 3. Slice map (execute in order)

### Slice A — Fix RequireClaim first-claim (on #124)

**Why first:** Unblocks treating #124 as mergeable PARTIAL scaffold; small, core-only.

**Change set**
- Add a helper (name bikeshed OK), e.g. `claim_or_allow_explicit(workspace_id) -> Result<(), CommandError>` that:
  - short-circuits when `!is_enforcing()`
  - when unclaimed + enforcing + explicit `workspace_id`: **`record_claim` first**, then allow
  - when already claimed: allow iff `claimed == workspace_id`, else `permission_denied`
- Use it from:
  - `gate_workspace_id_param`
  - `resolve_surface_target_scoped` explicit-`workspace_id` arms (both surface+workspace and workspace-only)
- Do **not** auto-claim from bare surface ids / focused fallback / §1c management helpers (`check_focused_workspace_against_claim` stays check-only).

**Tests**
- NEW: `entitlement_require_claim_natural_first_workspace_id_binds` — fresh `ConnectionEntitlement` in `RequireClaim`, **no** prior `record_claim`, dispatch with explicit foreign-then-own workspace_id path; first own id succeeds and sticks; subsequent foreign fails.
- Keep existing `entitlement_*` + #107 tests green.
- Fix or rewrite the papered-over portion of `entitlement_mode_flag_pins_fail_open_vs_fail_closed` so RequireClaim success is proven via natural claim, not only out-of-band claim.

**Verify:** `cargo test -p limux-core --lib`
**PR action:** Push to #124 branch; update PR comment that first-claim bug is fixed; still label PARTIAL (B/C/D/E remain). Merge #124 only after A is green **and** operator accepts PARTIAL merge (optional rebase onto current `main`).

**Out of scope for A:** server wire-up, GTK, workspace.list gating, default flip.

---

### Slice B — Live server wire-up (`from_env` + per-connection cell)

**Depends on:** A merged (or stacked on A tip). Prefer separate PR: `bari/h1-entitlement-live-wireup-YYYYMMDD`.

**Change set**
- At accept / connection start in `limux-control` (and any host-side control server path that constructs the dispatcher), build:
  - `EntitlementConfig::from_env()` once per process (or per connection if config can change — prefer once per process + clone cell)
  - `ConnectionEntitlement::new(config)` **per connection** (sticky claim must not be shared across peers)
- Thread entitlement into `Dispatcher::dispatch` → `dispatch_with_entitlement` (stop hardcoding `Off` in `handle_command` shim for live path; keep Off-shim only for tests that intentionally ignore entitlement).
- Module docs in `entitlement.rs` must match reality after this lands.

**Tests**
- Unit/integration: with env `LIMUX_ENTITLEMENT=require-claim`, unclaimed connection without workspace_id is denied on content path; with explicit workspace_id natural-claims (relies on A).
- With unset env → Off → pre-patch behavior.
- Prefer env-guarded tests (`Mutex`/serial or `temp_env` pattern already used in `from_env_falls_back_to_off_when_unset`).

**Verify:** `cargo test -p limux-core --lib` + `cargo test -p limux-control --lib` (and any host control bridge tests touched).
**Host bounce:** NOT required to land code; flipping env on a live install is operator-gated and still default Off.

**Out of scope for B:** GTK `window.rs`, discovery gating, default flip.

---

### Slice C — GTK / production path entitlement

**Depends on:** A+B (so live accept path can supply a cell). Separate PR: `bari/h1-entitlement-gtk-bridge-YYYYMMDD`.

**Change set (B.md §2.4)**
- Add `entitlement: ConnectionEntitlement` (or `Arc`-shared handle) onto the live `ControlCommand` / bridge request context.
- Thread through `workspace_index_for_target` (`window.rs:1077`):
  - `Handle` / `Name` / `Index`: reject when `!allows_workspace(resolved_id)` (after A’s claim-or-allow semantics for explicit ids)
  - `Active` under `Claimed(W)`: require focused == W; on mismatch return **`not_found`**, not `PermissionDenied` (anti-oracle)
- Prefer a single sanctioned helper (`EntitledWorkspaceResolver` or similar) so missed call sites cannot silently bypass; migrate ~20 call sites.
- §1c management set (focus/close/move/reorder/drag/refresh): resolve against **claimed** workspace when claimed, not `current_workspace_idx()` alone; closes `workspace.select(W')` → `surface.focus(<id in W')` exfil chain.

**Tests**
- Port/add scout §3 items that belong on GTK path, especially `workspace_index_for_target_rejects_foreign_handle_under_claim`.
- Active-under-foreign-claim → `not_found`.
- Do not weaken #107 / existing bridge tests.

**Verify:** focused `limux-host-linux` tests for window/control_bridge; avoid full `./scripts/check.sh` unless needed; **no host bounce**.

**Out of scope for C:** default flip; final discovery policy if still blocked on E (may stub behind same Off flag).

---

### Slice D — `workspace.list` / `current` / `select` policy

**Depends on:** E decision (below). May implement **behind Off** with mode branches once E is chosen; otherwise document-only.

**Recommended policy (proposal for operator ratify in Slice E)**

| Mode / state | `workspace.list` | `workspace.current` | `workspace.select` |
|---|---|---|---|
| `Off` | all (today) | today | today |
| `UnclaimedAllEntitled` + Unclaimed | all | today | allowed (operator path) |
| `UnclaimedAllEntitled` + Claimed(W) | **only W** (or all if operator-signal says all-entitled only while unclaimed — prefer only W once claimed) | must match W / else deny | select away from W → deny (or require re-claim path — deny sticky) |
| `RequireClaim` + Unclaimed | deny or empty + permission_denied | deny | deny unless select carries claimable id then claim |
| `RequireClaim` + Claimed(W) | only W | W only | deny foreign select |

**Note:** Scoping list under Claimed(W) removes the discovery primitive that makes foreign targeting trivial; operator multi-workspace view requires the operator-signal path (E).

**Until E is ratified:** keep ungated; update design/fast-follow notes that D is blocked on E.

---

### Slice E — Operator-vs-agent signal decision (operator gate)

**This is not a silent code choice.** Present at plan-mode start; do not invent.

**Options (from B.md §2.5)**
1. **Positive operator signal (RECOMMENDED):** Unclaimed is NOT all-entitled. Operator asserts all-entitled via `LIMUX_OPERATOR=1` (or `connection.claim_all`). Misconfigured agents fail closed. Pairs naturally with eventual default `require-claim`.
2. **Separate operator socket:** second 0600 socket with no entitlement; agent socket enforces claims. Cleaner trust split; more surface/docs/install complexity.
3. **Keep `UnclaimedAllEntitled` as the only non-Off mode** — **NO-GO as default-on**; acceptable only as explicit opt-in mode string, never as process default.

**Plan-mode ask (mandatory before implementing D or flipping defaults):**
- Pick 1 or 2 (or defer D and keep Off).
- Confirm: default remains `Off` until a later dedicated PR after A–C (+ D if chosen) are on `main` and soak.

**Docs after decision:** update design doc § decision log, fast-follow §7, HANDOFF/BARI with chosen signal and "default still Off".

---

## 4. Suggested PR sequence

```
A  fix first-claim on #124          → merge PARTIAL scaffold
B  live from_env + per-conn cell    → merge (still default Off)
C  GTK workspace_index entitlement  → merge (still default Off)
E  operator decides signal          → docs commit (may be before D code)
D  discovery/select gating          → merge per E
F  (FUTURE, separate) default flip  → ONLY after soak + explicit operator GO
```

Do **not** combine A+C or B+default-flip.

---

## 5. Docs / handoff / TaskMaster updates (this prep packet + each slice)

### This prep packet (before session close)
- Author this plan (`local://` + `/tmp/limux-wave-briefs/reports/H1-residuals-plan.md`).
- Update `HANDOFF.md` + `BARI_HANDOFF.md` on a docs branch from `main`: point to plan URI, state #124 residuals A–E as next packet, note review comment URL, leave live host alone.
- Optional: short PR comment on #124 linking this plan (no merge).

### Per-slice
- HANDOFF/BARI: what landed, what remains, SHAs, PARTIAL vs CLOSED language.
- Fast-follow §7 / design doc: decision log entries for E; CLOSED banners only when A–D complete per policy.
- TaskMaster: do **not** mark H1 done until C lands and E decided; use `task-master-reviewed` only (never hand-edit tasks.json).

---

## 6. Verification matrix

| Slice | Command / check |
|---|---|
| A | `cargo test -p limux-core --lib` — natural first-claim test PASS; #107 PASS |
| B | core + control lib tests; env Off vs require-claim |
| C | host-linux window/bridge tests; Active→not_found assertion |
| D | bridge tests for list/current/select under modes |
| E | docs-only; operator ack recorded in HANDOFF |
| Always | `limux doctor` read-only OK; host PID unchanged; no disclosure probes |

---

## 7. Resume checklist for next session (hcom / plan mode)

1. Read `local://limux-h1-residuals-plan.md` (authoritative). If missing, recover from `/tmp/limux-wave-briefs/reports/H1-residuals-plan.md` or git docs commit.
2. Read `HANDOFF.md` then `BARI_HANDOFF.md`.
3. Confirm live host still PID 860496 (or re-measure; do not bounce).
4. Ask operator only for: (i) proceed Slice A now? (ii) E signal choice if reaching D/default.
5. Execute Slice A on #124 via ephemeral worktree / PR branch checkout discipline.
6. Stop after A verify unless operator expands scope mid-session.

---

## 8. Explicit non-goals (this campaign)

- Matched reinstall / host restart for #122/#123.
- OMP cmux-parity 7.3 sidebar visibility.
- Live GUI verify of scroll/#84.
- Successor-rebind entitlement coupling (note overlap only).
- Hygiene re-archive of sockets/tmp.

---

## 9. Acceptance for "H1 CLOSED"

All must be true:
- [x] Slice A pushed on #124 (`c62713f`); merge pending operator PARTIAL ack
- [ ] Slice B merged (env actually honored; default still Off)
- [ ] Slice C merged (GTK path entitled; §1c claimed resolution)
- [ ] Slice E decided and recorded
- [ ] Slice D implemented per E (or explicitly waived in writing)
- [ ] Default still Off unless a later dedicated flip PR + soak GO
- [ ] HANDOFF/BARI/fast-follow language says CLOSED with residuals listed honestly

Until then: **PARTIAL** only.
