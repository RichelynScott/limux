# Limux Dame Manager Handoff

Checkpoint date: 2026-08-12
Owner: `dame`
HCOM session: `019ff794…` (omp, cwd `~/MCPs/limux`)
Scope: all Limux work, per operator succession from `gula` (GULA_SUCCESSOR_PROMPT.md)
Branch: `dame/anti-refill-20260812` (from `origin/main` @ `204a3b6`)

## Immediate Next Action

**CRASH INVESTIGATION (operator priority, 2026-08-12):** WSL hard crash again; Limux suspected. Two Limux-side contributors addressed:

1. **PR #137** (`dame/anti-refill-20260812`, `51710d0` + `4c4940b`): shared Cargo target + disk gate + retention — addresses the build-wave disk churn contributor (1.486 GiB in the incident window per momo's forensics). **PAUSED** — anti-refill task #1 has a resume note; remaining: operator-selected retention threshold, PR merge.
2. **PR #138** (`dame/renderer-tick-overlap-20260812`, `76c0f4f` + `45351ec` + `227fe58`): fixed the 100 ms hidden-tick overlap with the 8 ms visible frame timer (~10 ticks/s), and added aggregate timer-vs-wakeup counters through health output. The timer counter combines 8 ms + 100 ms timer calls; the remaining ~85/s delta is unattributed pending live measurement. The combined-tree wrapper test passed all 497 host tests; this is not live validation.

**Next:** coordinate with momo (corrected durable notice filed in nafo_INBOX) + nafo (hcom inform sent) on the crash; no peer reply has arrived yet. An approved build/install/restart is required before live deltas can be captured. Do not claim renderer-probe activation: #136 remains deliberately inactive because `child_env_removal_supported=false`; the supplied invocation requested no D3D12 policy, and isolated probing success does not activate the live host. After an approved #138 install, sample timer-vs-wakeup deltas. The clock-rotation churn (P1) and C: headroom (P0) remain operator-gated per momo's report.

## What Was Done (anti-refill lane)

- **Measured baseline (2026-08-12T20:29Z):** `target/` = 4.27 GiB allocated (3.72 GiB debug: deps 1.93 + incremental 1.61 + build 0.18; 0.54 GiB release: deps 0.50 + build 0.04). Disk `/` 33% used (310 GiB of 1007 GiB). Tool: `du -sk` (allocated KiB), source SHA `204a3b6`, branch clean.
- **`scripts/cargo-env.sh`** — shared Cargo target resolver (mirrors zori's `hcom-cargo.sh`): resolves the owning repo root via `git rev-parse --git-common-dir`, always absolute, `--print-target` / `--env` / exec-cargo modes, refuses outside a git checkout / bare repos / unknown flags. Empirically proven: a build from a linked worktree with `--manifest-path` to the main checkout lands in the shared `target/`.
- **Wired into:** `scripts/check.sh` (fmt/clippy/test via cargo-env), `scripts/package.sh` (release build), `scripts/xvfb-smoke-test.sh` (both builds), `scripts/limux-dev` (hint text). `install-user-local.sh` needs no change (reads `$repo_root/target/$profile`, same path).
- **`scripts/disk-gate.sh`** — build-wave discipline: `--report` prints allocation (report-only, exit 0); `--max-target-gib <N>` / `LIMUX_TARGET_MAX_GIB` fails closed BEFORE a build when allocation exceeds the OPERATOR-SELECTED limit. No invented thresholds (advisory-compliant). Compares raw KiB, not rounded GiB display.
- **`scripts/target-retention.sh`** — non-destructive retention report: profile → deps/incremental/build breakdown, `--report-file` optional, never deletes/moves/modifies (archive-not-delete compliant).
- **`scripts/tests/cargo-env-retention.sh`** — 18 assertions: shared-target from main checkout + linked worktree, absolute-path guarantee, `--env` form, refuse paths (no args, unknown flags, outside git), disk-gate report/limit-pass/limit-block/limit-0-no-target/invalid-value, retention report-only/no-mutation/report-file/refuse paths. Fixture-isolated (real scripts invoked from fixture cwd; no real target/ touched).
- **AGENTS.md** — documented the shared-target mandate, cargo-env usage, disk-gate, and retention report under Quality Gate.

## Verification

- `scripts/tests/cargo-env-retention.sh` — PASS.
- `scripts/cargo-env.sh check -p limux-cli` — PASS (4.27s warm).
- `scripts/cargo-env.sh test --workspace` — PASS: 165 CLI + 5 launcher-route + 40 + 2 + 5 + 56 + 496 host + 9 = 778 tests, 0 failures (matches gula's documented baseline).
- `bash -n` on all 7 modified/new scripts — PASS.
- **Combined-tree integration evidence:** a temporary merged #137 + #138 validation tree passed `scripts/cargo-env.sh test -p limux-host-linux` with 497 tests and 0 failures. This does not mean the focused #137 branch contains the #138 source changes; no live install/restart occurred.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` — FAIL on PRE-EXISTING drift in `rust/limux-cli/src/main.rs` (101 fmt diffs; clippy: `contains()` vs `iter().any()`, `assert_eq!` literal bool) and `rust/limux-host-linux/src/window.rs:6478` fmt. Zero Rust files modified by this branch (`git diff origin/main...HEAD -- rust/` empty). Matches gula's documented "repository-wide formatting/clippy still has unrelated pre-existing drift."

## Coordination

- hcom ack sent to `gula` (stored; live wake attempted) — succession absorbed, gula identity NOT reclaimed.
- hcom inform sent to `zori` (stored to INBOX; hook-only delivery) — shared-Cargo design mirrors hcom-cargo.sh; asked for constraints.
- corrected notice sent to `momo` via `nafo_INBOX`; no peer reply has arrived yet. The notice says the remaining delta is unattributed, counters are aggregate timer-vs-wakeup, wrapper verification passed, and no live install/restart has occurred.
- TaskMaster: tag `limux-anti-refill-20260812` created; task #1 (tag) created via `task-master-ai-reviewed` (manual title/description, no AI prompt). Task #7 (renderer) remains blocked/untouched.

## Preserved State (do not touch)

- 130 untracked entries preserved: 125 `GULA_EVIDENCE/2026-08-12/`, 2 `docs/research/` (LIMUX_CONTROL_SCOUT, T3CODE_MOBILE_SCOUT), `WATCHDOG.yml`, `LIMU_INBOX/ALERT_FROM_bora_2026-07-31_omp-shell-keeps-dying.md`, `AUTOPILOT_LOG.md`. All peer-owned; exact-owner review required before any disposition.
- `GULA_HANDOFF.md`, `GULA_SUCCESSOR_PROMPT.md` — predecessor-owned, read-only records.
- Live runtime UNCHANGED: no install, restart, promotion, renderer activation, or artifact deletion. The installed daily-driver resource drain remains unresolved (renderer task #7 blocked on Ghostty env-removal seam).

## Traps (carried from HANDOFF.md §8)

- Shared checkout: never `git checkout`/branch-switch while a peer is live; branch-in-place is the default.
- Push immediately after every commit; subagents die and lose unpushed work.
- Never hand-edit `.taskmaster/tasks/tasks.json`; use `task-master-reviewed` / `task-master-ai-reviewed`.
- `papa-git`: `export CLAUDE_SESSION_NAME=DAME_LIMUX_MGR CLAUDE_AGENT=claude` in every committing bash call (lowercase refused).
- Vendored `ghostty/` is READ-ONLY; work through the C API.
- Clippy `-D warnings` is a hard gate; pre-existing drift is documented, not mine to fix silently.
- The Codex PR bot is not reviewing (fleet-wide close-out); weigh 0-review merges.
