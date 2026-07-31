# Limux H1 / #124 residuals — authoritative plan (2026-07-31)

**Status:** Slice **A DONE** on #124 @ `c62713f` (verified). Slice **B DONE in code** on OPEN [#128](https://github.com/RichelynScott/limux/pull/128) @ `f55cb2b` (verified; not merged). **Next = Slice C** (GTK). E before D. Still PARTIAL — not H1-closed.
**Authoritative URI:** `local://limux-h1-residuals-plan.md`
**Durable repo copy:** `docs/LIMUX_H1_RESIDUALS_PLAN_2026-07-31.md` (on `main`)
**Wave-brief copy:** `/tmp/limux-wave-briefs/reports/H1-residuals-plan.md`

## Already done (do not repeat)

| Slice | Evidence |
|---|---|
| Hygiene / Wave2 / matched install | doctor green; host PID **860496** on `main-15ccb28ed4a8-matched-20260731` — leave alone |
| Scaffold #124 base | `2c8ad7c` entitlement scaffold, default-off |
| **A** claim-first | `c62713f` — `claim_or_allow_explicit`; gates use claim-first; `entitlement_require_claim_natural_first_workspace_id_binds` |
| **A verify** | `cargo test -p limux-core --lib entitlement_` → **8 passed**; entitlement filter **18 passed** |
| **B** live wire-up | `f55cb2b` on `bari/h1-entitlement-live-wireup-20260731` — `server.rs`/`ffi.rs` `EntitlementConfig::from_env` + per-conn cell + `dispatch_with_entitlement`; Off shim tests-only in `handle_command` |
| **B verify** | `cargo test -p limux-control --test entitlement_wireup` → **2 passed**; #128 MERGEABLE/CLEAN; B contains A tip |
| Docs | #126 plan, #127 A handoff, #129 B docs on `main` @ `eb4d8a0` |

## Non-negotiables

1. No host bounce of PID 860496 without fresh ack.
2. No live H1 disclosure probes.
3. Never flip default off `Off` in the same PR as new enforcement.
4. Do not say “H1 closed” until A+B+C landed, E decided, D scoped, § CLOSED met.
5. Ephemeral worktrees; `CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude`.
6. Never edit tests to force green.

## Remaining execution order

### C — GTK `workspace_index_for_target` (**START HERE after restart**)

**Branch:** stack on #128 tip `f55cb2b` (or merge #128 first then branch from main+A+B).
**Files:** `rust/limux-host-linux/src/window.rs` (`workspace_index_for_target` + call sites); ControlCommand / bridge entitlement threading.
**Change:**
1. Thread `&ConnectionEntitlement` (or owned clone of Arc cell) into GTK control path.
2. After resolve: if `!allows_workspace(id)` → reject.
3. **Active** under `Claimed(W)`: focused must be W else return **`not_found`** (not `PermissionDenied`) — B.md §2.4.
4. §1c management: resolve against **claimed** workspace, not only `current_workspace_idx()`.
5. Add host test for foreign handle under claim (PR #124 PARTIAL §3 #2).

**Verify:**
```bash
rg -n 'workspace_index_for_target|current_workspace_idx' rust/limux-host-linux/src/window.rs
cargo test -p limux-host-linux --lib -- --nocapture   # focused entitlement tests must pass; do not weaken unrelated failures
```

**Risks:** PermissionDenied on Active leaks existence; missed call sites; sharing one cell across GTK commands incorrectly.

### E — Operator-vs-agent signal (**before D / before default flip**)

Decide and record in `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md` + `BARI_HANDOFF.md`:
1. `require-claim` agents + positive operator signal, or
2. Separate operator socket.
Do **not** default-on `UnclaimedAllEntitled` (scout NO-GO). Default stays **Off**.

### D — `workspace.{list,current,select}`

Gate per E. Claimed agent must not enumerate foreign ids; operator path preserved; Off unchanged.

**Verify:** core/host workspace entitlement tests.

## Landing posture

- #124 MERGEABLE/CLEAN @ `c62713f` — may merge as **PARTIAL** (A included); not H1-closed.
- #128 MERGEABLE/CLEAN @ `f55cb2b` — Slice B; merge when operator accepts (still Off).
- Prefer merge #124 then #128 (or merge #128 if it targets main with A commits — confirm stack) before large GTK C, **or** keep C stacked on `f55cb2b` until merges land.
- Activation / default flip = separate operator-acked PR after C+E+D.

## H1 CLOSED criteria (all required)

1. Natural first-claim under RequireClaim without out-of-band `record_claim`. ✅
2. Live accept: from_env + per-conn + dispatch_with_entitlement. ✅ in #128 (pending merge)
3. GTK path entitled; Active → not_found on mismatch. ❌ C
4. Operator-signal written; workspace.list/current/select gated. ❌ E then D
5. Default Off or explicit activation PR.
6. Tests green without cheating.
7. Docs/fast-follow §7 say CLOSED only then.

## Post-restart checklist

1. Read this file (and `docs/LIMUX_H1_RESIDUALS_PLAN_2026-07-31.md`).
2. Confirm shared checkout on `main`; host 860496 untouched; doctor ok.
3. Operator pick: merge #124/#128 now, or stack C on `f55cb2b` first.
4. Execute **Slice C** unless redirected to E.
