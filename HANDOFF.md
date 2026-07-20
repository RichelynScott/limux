# Limux — Canonical Session Handoff (single entry point)

> **READ THIS FIRST. This is the session-agnostic current-state doc for the
> Limux repo.** Any session (lifo, hamo, tutu, or a brand-new one) opening this
> directory should be able to know everything going on from this file alone.
> Per-session detail files (`LIFO_HANDOFF.md`, `HAMO_HANDOFF.md`, etc.) are
> retained below as an index — but the CURRENT TRUTH lives in this top section,
> not in any one session's file.

**Consolidated by:** Claude Code (tutu · cd1a39d7) — operator-directed
**Last verified:** 2026-07-19 ~1:15 PM EST (runtime state independently checked, not relayed)
**Why this file was restructured:** the repo had 5 competing per-session handoff
files (`LIFO_HANDOFF`, `HAMO_HANDOFF` [was only in `/tmp`], `NATO_HANDOFF`,
`HALO_HANDOFF`, `LIFO_CL_MGR_HANDOFF`) plus a thin pointer, so you had to already
know who ran last to know which was authoritative. Operator directed a single
consolidated surface on 2026-07-19.

---

## 1. CURRENT RUNTIME STATE (verified 2026-07-19)

**Stable Limux v0.2.3 is the active daily runtime.** Force-restart completed
2026-07-19 ~05:15 EDT (hamo, operator-authorized). Independently re-verified by
tutu 2026-07-19 ~13:15 EST.

| Field | Value | Verified |
|---|---|---|
| Active runtime | **v0.2.3 stable** | ✅ |
| Host process | PID `37671` → `.../stable/main-1a26bda0-v0.2.3-20260719/libexec/limux-host` | ✅ `ps` |
| Build | `1a26bda0bd1c`, release, no dirty marker, `channel=stable` | ✅ |
| Stable socket | `/run/user/1000/limux/stable/limux.sock` | ✅ exists |
| doctor | `ok=true, exit_code=0` — launchers/processes/socket/stale_sockets/ghostty_resources all `ok` | ✅ **via `limux-stable-cli`** |
| Workspaces | 28 legacy workspaces migrated hash-identically into stable session | per hamo handoff |
| Legacy v0.2.2 host (PID 29087) | **stopped**; legacy socket `/run/user/1000/limux/limux.sock` gone | ✅ |
| origin/main | `fec619e4b9863b16a33f0b5dbd5d3e244b26e1ff` | ✅ |
| Release merges | PR #73 (`1a26bda0…`), PR #74 closeout (`7c760f0c…`) | per hamo handoff |

### ⚠️ KNOWN ISSUE #1 — plain `limux` on PATH is still LEGACY v0.2.2 (cosmetic-but-confusing)

If you run plain `limux doctor` you will see:
`[fail] socket: connected host build SHA differs from CLI build SHA`
and `limux --version` reports `0.2.2 (1005f58d92a1) channel=legacy`.

**This is NOT a runtime failure.** It is a launcher-not-repointed condition:

- `~/.local/bin/limux` and `~/.local/bin/limux-cli` still symlink to the OLD
  `main-1005f58d-pane-timeout-clean-20260716` (legacy v0.2.2) install.
- That legacy CLI defaults to the legacy socket, which is now dead → the SHA
  mismatch / fail.
- The **healthy v0.2.3 runtime is reachable via `limux-stable` /
  `limux-stable-cli`** (symlinks created 2026-07-19, → the stable v0.2.3 install).

**To use the live runtime today:** use `limux-stable` / `limux-stable-cli`, or
pass the stable socket explicitly.

**The fix (NOT yet applied — needs an owner/operator decision):** repoint
`~/.local/bin/limux` + `~/.local/bin/limux-cli` at the stable install. hamo
deliberately kept legacy as rollback provenance, so this was left as an explicit
decision, not auto-done. Until it's repointed, operator muscle-memory `limux`
targets the dead legacy path.

### ⚠️ KNOWN ISSUE #2 — TaskMaster tag discrepancy (unresolved)

`HAMO_HANDOFF.md` states "active tag restored to `limux-resource-crash-20260716`",
but that tag **does not exist** in `.taskmaster/tasks/tasks.json` (present tags:
`master`, `cmux-parity-20260707`, `product-hygiene`) and `.taskmaster/state.json`
says `currentTag: master` (last switched 2026-07-10). A successor should
reconcile this before relying on either claim.

---

## 2. OPEN LANES (what's still in flight — none block the completed v0.2.3 release)

| Lane | State | Where |
|---|---|---|
| **reve new-pane incident** | `limux new-pane` panes created but terminal never initializes (`--command` never runs). Filed against **legacy 0.2.2**; a **v0.2.3 retest is requested**. | `LIFO_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md` (untracked) |
| **TaskMaster `master` tag** | 25 tasks; open: #7,#8,#10,#11,#12 (Cursor-ext surface), #13 in-progress (V2 boundary doc), #16,#17,#20 (UI), #23,#24 in-progress (surface-pane identity, sandbox relaunch), #25 (byte-safe send) | `.taskmaster/` |
| **TaskMaster `cmux-parity-20260707`** | 10 tasks; several in-progress (#3,#5,#6,#7 — post-install checklist, live-bridge parity, WebKitGTK spike, agent lifecycle) | `.taskmaster/` |
| **TaskMaster `product-hygiene`** | #1 in-progress (version bump/rendering) | `.taskmaster/` |
| **dino hcom-lineage-recovery** | Suspended `unclean_restore`; worktree `/tmp/hcom-lineage-recovery-20260719` **must not be removed**. Unrelated to Limux release. | dino's lane, not Limux |

---

## 3. DIRTY WORKING TREE (current, 2026-07-19)

Branch: `lifo/limux-first-hcom-tracking-20260715` (clean vs its remote). Untracked:

- `LIFO_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md` — reve's incident (see Open Lanes)
- `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` — **Lifo-owned**; do not stage/modify/remove without lifo/operator authorization
- `HAMO_HANDOFF.md` — rescued into repo by tutu from `/tmp` on 2026-07-19 (was reboot-fragile); safe to stage

---

## 4. PER-SESSION DETAIL FILES (index — the "who did what" record)

These are retained for depth. **Do not treat any single one as current truth —
Section 1 above is current truth.** Do not overwrite a peer's file; each is
owned by its named session.

| File | Owner / era | Contains |
|---|---|---|
| `HAMO_HANDOFF.md` | hamo / 2026-07-19 | v0.2.3 release + force-restart detail, evidence paths, critical rules (rescued from `/tmp`) |
| `LIFO_HANDOFF.md` | lifo / through 2026-07-16 | PR #70/#71 housekeeping, pre-restart v0.2.2 state, prior next-actions (now superseded by Section 1) |
| `LIFO_CL_MGR_HANDOFF.md` | lifo (CL mgr) | earlier lifo lane |
| `NATO_HANDOFF.md` | nato | earlier lane |
| `HALO_HANDOFF.md` | halo / 2026-06 | Limux-improvement re-anchor, June crash triage |
| `FYI.md` | append-only journal | decision history (large — condense later under approved cleanup) |

---

## 5. KEY RUNTIME PATHS & EVIDENCE ARTIFACTS

| Path | Purpose |
|---|---|
| `~/.local/limux-reviewed/stable/main-1a26bda0-v0.2.3-20260719/` | **active** stable v0.2.3 install (`bin/limux-stable`, `bin/limux-stable-cli`, `libexec/limux-host`) |
| `~/.local/limux-reviewed/main-1005f58d-pane-timeout-clean-20260716/` | legacy v0.2.2 install (rollback provenance; `~/.local/bin/limux` still points here) |
| `/run/user/1000/limux/stable/limux.sock` | active stable socket |
| `/run/user/1000/limux-archive/force-restart-20260719T0515/` | archived non-connectable socket residuals (kept outside runtime scan) |
| `/tmp/limux-release-0.2.3-20260719/` | ephemeral release worktree (retained for no-loss closeout; do not delete without no-loss gate) |
| `/tmp/limux-release-0.2.3-evidence-20260719/`, `/tmp/limux-installed-stable-smoke.*/` | retained release/boundary-review evidence incl. `doctor.json` |

---

## 6. CRITICAL BEHAVIOR RULES (carried forward)

- **v0.2.3 release unit is COMPLETE** — do NOT redo the release, merge, or stable
  install. New Limux work starts from current `origin/main` + repo lane preflight.
- Stable is the active daily runtime by explicit operator authorization. Keep
  legacy install/launcher as rollback provenance; do not relaunch legacy
  concurrently or overwrite stable state without a new runtime decision.
- Do NOT broaden HCOM/OMP behavior from the release closeout — the release only
  integrated reviewed Limux product bytes.
- Preserve peer-owned dirt; use exact-path staging; archive, never delete.
- Do not delete the retained `/tmp` worktree/evidence or dino's
  `/tmp/hcom-lineage-recovery-20260719` worktree without the documented no-loss
  gate + owner authorization.
- Keep vendored `ghostty/` read-only; clippy is a hard gate (`-D warnings`).
- Verify runtime claims against the **stable** CLI/socket, not the legacy PATH
  `limux` (see Known Issue #1), before reporting doctor state.

---

## 7. HISTORICAL (Halo, June 2026 — retained, not current)

The original June-2026 Halo handoff content (Limux-improvement re-anchor away
from the VM/isolation goal, the 2026-06-20 crash triage, PR #1 / G0 stability
closeout, and crash-evidence commands) is preserved in `HALO_HANDOFF.md` and in
git history of this file prior to the 2026-07-19 consolidation. It is historical
context, not the active resume path. The active resume path is Sections 1–6 above.
