# PRD-C: Post-Install Verification Loop — Checklist + Status Write-Back

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:15 UTC
**Purpose:** Close the verification-loop gap: every fix wave ends "operator must
restart and retest" and the confirmation is frequently never recorded (task #14
`done` and #15 `review` both lack any recorded operator sign-off). Establish a
repeatable 10-minute checklist + mandatory write-back, and use its first run to
clear the current needs-verification backlog on a fresh current-main install.

- **Priority:** P0 (Wave 0 — roadmap W0.3)
- **Dependencies:** PRD-A (`doctor` identifies the build under test), PRD-B
  (fresh install ships valid resources). Can be authored in parallel; executed after both.
- **Effort:** S (mostly docs/process + small harness additions)
- **Execution model:** lifo prepares; **operator executes the live checklist**
  (~10 min, `[USER]` step); lifo records results
- **Channel targeting:** preview channel for the fresh install; promote symlinks
  to it only after the checklist passes

## Problem Statement

Defect-inventory finding §E9: six fixes are merged but unverified live —
keyboard/modifier (#14, marked `done` with no operator confirmation recorded
anywhere), window controls (#15, `review`), resize-storm coalescing fixes,
sidebar resize handle restore, runtime-channel isolation (merged after the
operator's current install was built — never exercised), and the stuck-click/
copy-paste watch item. The operator's live install (`resize-live-sync-ae26e0a`,
2026-07-01) predates all of `origin/main`'s newest fixes. There is no written
checklist, so verification happens ad-hoc and results evaporate.

## Goals

1. A written, versioned, ~10-minute operator checklist that covers the known
   defect classes and takes ≤1 command to set up.
2. Mandatory write-back: every checklist run produces an FYI.md entry and
   TaskMaster status updates via `task-master-reviewed`.
3. First execution clears the current backlog: #14 confirmed (or reopened),
   #15 → `done` (or reopened), resize/sidebar/channel items verdicts recorded.

## User Stories

### US-1: As the operator, I can verify a fresh build in ~10 minutes without thinking
- [ ] `docs/verification/post-install-checklist-v1.md` exists with numbered
      steps, each: exact action → expected result → PASS/FAIL checkbox.
- [ ] Checklist covers, at minimum: plain typing + modifier chords + paste
      (Ctrl+V / Ctrl+Shift+V) in a fresh pane (#14 class); window controls
      (minimize/maximize/close) + edge hitbox (#15 class); drag-resize soak
      ≥30 s while a live agent TUI (claude or codex) is running (resize-storm
      class); sidebar resize handle + collapse/restore; close-and-relaunch
      session restore of a multi-workspace layout; `limux notify` toast +
      sidebar dot; `limux doctor` reports clean/no-drift.
- [ ] Setup section is copy-paste: install preview-channel build from current
      main, launch, run checklist (exact commands included).
- [ ] Checklist header records: build SHA + install-id (from
      `limux --version`), date, operator verdict per item.

### US-2: As the fleet, results are durable — never verbal-only again
- [ ] A filled checklist is saved under `docs/verification/runs/<date>-<install-id>.md`
      (results file, committed).
- [ ] An FYI.md entry is appended per run (What/Why/How/Impact + link to the
      run file), by the session that ran the loop.
- [ ] TaskMaster statuses are updated via `task-master-reviewed set-status`
      for every task the run confirms or reopens (never hand-edit
      `tasks.json`).
- [ ] The checklist doc states the rule: a fix PR that changes
      operator-visible behavior is not `done` until a checklist run (full or
      the relevant subset) records its verdict.

### US-3: As lifo, the first run clears the current backlog
- [ ] `[USER]` Operator executes checklist v1 on a fresh preview install
      built from current `origin/main`.
- [ ] #14: typing/modifier/paste verdict recorded → task stays `done` or is
      reopened with the captured evidence (`LIMUX_DEBUG_KEYS=1` log if FAIL).
- [ ] #15: window controls verdict recorded → `review` → `done` or reopened.
- [ ] Resize-soak, sidebar-handle, and session-restore verdicts recorded.
- [ ] Runtime-channel isolation smoke: stable + preview run simultaneously
      without socket interference (`scripts/tests/runtime-isolation-smoke.sh`
      as the scripted part; operator confirms both windows behave).
- [ ] If all PASS: symlinks promoted to the new install; if any FAIL: verdict
      + evidence recorded, promotion blocked, new TaskMaster task filed.

## Functional Requirements

1. New files: `docs/verification/post-install-checklist-v1.md`,
   `docs/verification/runs/` (with `.gitkeep`), template
   `docs/verification/run-template.md`.
2. Optional small harness assist (only if cheap): `limux doctor` gains a
   `--checklist-header` flag printing the build/install/date block for
   pasting into a run file. (Skip if PRD-A lands without room — manual copy
   is acceptable for v1.)
3. No mandatory GUI automation: this is deliberately an OPERATOR loop — the
   classes it covers (live WSLg input, window chrome, drag feel) are exactly
   the ones headless harnesses miss. The Xvfb smoke suite remains the
   automated complement, unchanged.
4. Checklist versioning: v1 frozen after first run; edits create v2 (runs
   reference the version they executed).

## Non-Goals

- No automated GUI-driver test rig (out of scope; Xvfb harness already covers
  scriptable paths).
- No changes to fix-PR workflow tooling/hooks — the write-back rule is
  documented process, not enforcement code, in v1.
- No stable-channel mutation until the checklist passes.

## Technical Considerations

- Runtime-channel isolation (task #19) shipped `--channel stable|preview[:id]`
  with separate sockets/app-IDs/state — the checklist's install step must use
  preview explicitly so the operator's daily driver is never displaced by an
  unverified build (contract doc: `limux-runtime-channel-contract-20260702.md`).
- Keep checklist items symptom-anchored to the defect inventory so a FAIL maps
  directly to a known class (typing→#14/resource-shape, `?` glyphs→font-env,
  bracketed paste→shell-mode — the symptom-split from `LIFO_HANDOFF.md`).

## Success Metrics

- 100% of the §B needs-verification backlog has a recorded verdict after the
  first run.
- Every subsequent operator-visible fix merged to main gets a recorded
  checklist verdict before its TaskMaster task is closed.

## Testing Instructions

```bash
./scripts/check.sh                                  # unchanged — docs-mostly PRD
bash scripts/tests/runtime-isolation-smoke.sh       # scripted part of the run
# then the [USER] live checklist per docs/verification/post-install-checklist-v1.md
```

## Rollback Plan

Docs/process only — revert commits. No state migrations.

## Open Questions

1. Should run files live in-repo (proposed) or only as FYI entries? Proposed:
   in-repo — greppable history beats journal prose.
2. Does the operator want a reminder surface (e.g. `limux doctor` warning when
   the running build has no recorded checklist run)? Deferred to v2.
