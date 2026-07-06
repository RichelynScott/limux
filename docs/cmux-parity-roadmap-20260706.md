# Limux ⇄ cmux Parity Roadmap — WSL2, Stability-First

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 22:15 UTC
**Purpose:** Single prioritized roadmap for getting Limux as close as practical
to cmux (manaflow-ai/cmux) for the operator's real environment — WSL2
Ubuntu/zsh on Windows 11 — sequenced stability-first, decomposable into PRDs
for lifo + subagents to execute.

---

## 1. Goal statement

Limux is the Linux/WSL2 counterpart to cmux: a terminal workspace manager
purpose-built for running many AI coding agents in parallel, with a
metadata-rich sidebar, agent-hook-driven notifications, full CLI/socket
programmability, and (eventually) browser panes. "Parity" here means parity of
**product value on WSL2**, not source-level cloning — cmux is Swift/AppKit and
GPL; Limux translates each candidate into a Rust/GTK4/libadwaita/libghostty
native design (see `docs/research/cmux-upstream/README.md` licensing posture).

The roadmap is stability-first by operator decision (2026-07-06): the P0 wave
makes the runtime trustworthy and closes the verification debt before feature
waves widen the surface.

## 2. Evidence base

| Input | Source |
|---|---|
| cmux current feature surface + deltas since 2026-07-02 | Research pass 2026-07-06 (gh API: releases → v0.64.17, 95 merged PRs 07-02→07-06, README, source layout) |
| Limux feature/gap inventory | origin/main @ `36ea71d` (CLI dispatch, `control_bridge.rs` 18-method table, `limux-core` dispatcher, parity plan, TaskMaster store) |
| Defect + WSLg constraint inventory | FYI.md / handoffs / `docs/terminal-input-regression-20260701.md` / live log + symlink evidence 2026-07-06 |
| Scored candidate backlog | `docs/research/cmux-upstream/items.md` (lifo, 2026-07-02) |
| Lane truth from lifo | hcom 2026-07-06: task #14 done, #15 in review, Cursor lane at PR #15 open (task #6), PRD handoff = repo PRDs + TaskMaster-ready tasks |

## 3. Strategic finding — upstream `mux` changes the parity question

cmux landed a **Rust, Linux-capable backend called `mux`** on 2026-07-06
(cmux #7180 decoupled multiplexer; #7346 platform module + Linux CI, explicitly
"phase 1 of Windows/Linux support"; #7347 API/CLI/bindings contract spec;
#7325 CDP browser panes rendered via kitty graphics). Upstream is building the
Linux core itself.

Implications for Limux:

1. **Do not panic-pivot.** mux is phase-1 scaffolding; Limux is a working GTK
   product today. The daily-driver value stays with Limux.
2. **Align contracts, not code.** Limux already speaks newline-framed JSON over
   a Unix socket. A cheap research spike (W1.5) should diff Limux's control
   vocabulary against mux's contract spec and steer new API surface toward
   convergence, so agent scripts stay portable and a future
   mux-backend option stays open instead of permanently diverging.
3. **Steal the browser architecture decision.** mux chose CDP-driving an
   existing Chrome over an embedded engine. On WSL2 (WSLg, software GL) that
   trade is even more attractive than on macOS — see W1.2.

## 4. Wave 0 (P0) — "Trust the runtime"

Rationale: the defect inventory's worst finding is not a bug but a **class** —
installed-vs-running-vs-source drift caused ≥4 false bug reports and two wrong
root-cause hypotheses (every build logs `version=0.1.19`; the operator today
runs `resize-live-sync-ae26e0a` while main is a ~6,300-line superset). Second
worst: the one packaging relaxation (`ghostty/src` as runtime resources) caused
the severest user-facing regression (typing corruption), and current installs
ship **no** Ghostty resources at all. Feature work on top of an untrusted
runtime re-creates the June cycle.

| # | Item | What / acceptance sketch | Evidence |
|---|---|---|---|
| W0.1 | **Build identity + `limux doctor`** | Real build id (git SHA + install-id + channel) embedded at build time; printed by `--version` and on every host-log start line. New `limux doctor` compares symlink target, running-host binary path, repo main, channel, socket liveness; flags stale/mismatched builds in one command. Acceptance: doctor detects a deliberately stale install in a test harness; log start-lines carry SHA. | Defect inventory §E1: version string never bumps; 4+ drift incidents |
| W0.2 | **Ghostty resource/terminfo packaging correctness** | Build compiled terminfo + shell-integration into installs; installer self-verifies resource shape and fails loudly (no resources ≠ silent pass); CI asserts manifest invariants. Acceptance: fresh install manifest shows valid resources; installer aborts on source-only shape; regression test for the `ghostty/src` class. | `docs/terminal-input-regression-20260701.md`; current manifest "resources: not found" |
| W0.3 | **Fresh-main install + operator verification loop** | Install current main via the (new, unexercised) preview channel; run a written 10-minute operator checklist: typing/#14 symptoms, window controls/#15, drag-resize soak with a live agent TUI, sidebar handle, session restore. Results recorded to FYI + TaskMaster statuses (#14 confirm, #15 → done). Acceptance: every "needs-verification" item in defect inventory §B has a recorded verdict. | Defect inventory §B (6 unverified fixes), §E9 (verification-loop gap) |
| W0.4 | **Log hygiene + crash evidence** | Classify/annotate benign WSLg noise (EGL/Zink/popup-remap) at log level; tag browser-subprocess lines by origin; keep `LIMUX_DEBUG_KEYS` permanently; panic hook prints build id. Acceptance: a fresh log start is machine-classifiable into {benign-env, limux-warning, limux-error}. | Defect inventory §C/§D |
| W0.5 | **Pane attention border + per-pane color flags (TaskMaster #20)** | Fix z-order so attention border renders above content; deterministic per-pane color flags preserving unread semantics. The one live UI defect promoted into P0. | tasks #20; `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md`; research db cmux-013 |

Explicit W0 non-duplication rule: PR #6 (`c0a294c`) already stabilized
workspaces/input/resizing/window chrome — W0 verifies it live (W0.3), it does
not re-plan it.

## 5. Wave 1 (P1) — parity core

| # | Item | What / acceptance sketch | Evidence |
|---|---|---|---|
| W1.1 | **Live-bridge parity core** (architectural unlock) | Implement the designed-but-unbuilt bridge parity layer: GTK host owns a live-synced `ControlState`; `control_bridge` routes its 18 GTK-side-effect methods as today, falls through **read-only** methods to `limux_core::Dispatcher`, and adds explicit GTK mutation routes for live UI-changing verbs. Closes the 18-vs-100+ method gap without mutating only the mirror: tmux-compat verbs (`swap-pane`, `resize-pane`, `break/join-pane`, window nav), `workspace.reorder/move_to_window`, `surface.split/focus/close/move`, `notification.list/clear`, `tab.action` all work against the running GUI through the correct read-vs-mutation route. Acceptance: Xvfb smoke drives a representative method from each family live; method-coverage table generated from code. | Parity plan Phase 2 (open since inception); limux inventory §B |
| W1.2 | **Browser panes, live** (design decision inside PRD) | Bring browser capability to the running GUI. Two candidate architectures, decided in the PRD after a short spike: (a) WebKitGTK embedded panes wired through the bridge (existing standalone `browser.*` model), or (b) mux-style CDP-driving an existing Chrome/Chromium with the pane as a rendered view. WSL2 lens: WebKitGTK under WSLg = heavy, software-GL, subprocess log noise (already observed); CDP reuses the user's real browser + auth. Ships with server-side domain allowlist + audit events (security posture from research db cmux-001). Depends on W1.1. | Research db cmux-001 (high); cmux #7325 precedent; parity plan "browser command bridge parity" |
| W1.3 | **Agent lifecycle sidebar** | Sidebar rows show agent state per workspace/pane: running / needs-input / idle / unknown, fed by the existing hook pipeline (claude/codex/gemini/opencode/hermes hooks already land events). Includes sidebar scalability: no full list-model rebuilds per title/output update. Acceptance: state transitions visible in Xvfb harness with fake agents; sidebar update cost measured before/after under 20 simulated workspaces. | Research db cmux-002 + cmux-007 (both high); cmux README sidebar identity |
| W1.4 | **Session restore correctness pack** | Persist + restore split order/ratios/pane identity; preserve cwd when splitting or opening tabs from panes hosting resumed agents; recently-closed/focus history (stretch: scrollback restore). WSL2 framing: survive host restarts (the operator's normal life event) with layout intact. Acceptance: scripted kill/restart harness restores a 3-workspace mixed-split layout byte-identically; cwd preserved on split from a deep dir. | Research db cmux-004/008/009 (high); cmux #4130/#6146 precedent |
| W1.5 | **mux contract-alignment spike** (research, no code) | Read mux's contract spec (cmux #7347) + JSON-lines socket shape; produce a decision doc: where Limux vocabulary already matches, where new W1 surface should converge, whether a future mux-backend option is worth keeping open. Timeboxed; output feeds W1.1/W1.2 PRD review, not a separate build. | cmux research §C1/§E1 |

## 6. Wave 2 (P2) — fidelity + daily-driver polish

- **W2.1 Render sizing / fractional scale correctness** — port upstream-Limux
  physical-pixel fixes manually (research db limux-up-001; upstream PRs #83/#100).
- **W2.2 IME / dead-key input correctness** — harvest upstream tests + Wayland
  IMContext behavior (limux-up-002; upstream #90).
- **W2.3 Shortcut contract review** — Ctrl+W closes tab not pane/workspace;
  audit the reserved-binding guard set (limux-up-003).
- **W2.4 Occluded-surface throttling** — don't burn CPU/GPU rendering hidden
  panes streaming agent output (cmux-006; matters more under WSLg software GL).
- **W2.5 Workspace groups + saved split layouts** — organization at agent scale
  (cmux v0.64.11 groups; cmux #7414 layouts).
- **W2.6 Command palette + `limux.json` custom commands** — project-scoped
  actions from config, palette-launched (cmux README/#4043).
- **W2.7 Sidebar git/PR metadata** — branch, dirty state, linked PR
  status/number per workspace row, polled with `GIT_OPTIONAL_LOCKS=0`
  (cmux README/#5907; research db cmux-010/011).
- **W2.8 Per-terminal font size/scaling in settings** (limux-up-007).

## 7. Wave 3 (P3) — reach features (translate-when-stable)

- **W3.1 Agent session lifecycle: resume / fork / hibernation** — resume
  agent sessions after restart, fork conversations, hibernate idle agent panes
  (RAM matters on WSL2). cmux #4198/#6803/v0.64.11.
- **W3.2 SSH remote workspaces + detachable PTY daemon** — cmuxd-style
  reconnect-surviving remote sessions (cmux v0.64.11/#7250/#7463).
- **W3.3 Notification panel + unread-jump + per-category gating** — beyond
  current toast/dot model (cmux #7129, ⌘⇧U analog).
- **W3.4 Extensions / custom sidebars** — only if a real internal need appears
  (cmux #5382).

**Non-goals (explicit):** iOS companion, Cloud VMs / Pro billing, Vault cloud
sync, freeform canvas (upstream calls it not production-ready), source/asset
copying from cmux (GPL — translate designs only).

## 8. Committed parallel lane — Cursor IDE integration (unchanged)

Lifo continues TaskMaster #6–#13 (server-side restricted method surface → tree
provider → select/present → empty pane → safe folder-open → read-only
snapshots → acceptance gates → v2-boundary docs). PR #15 open at time of
writing. The roadmap treats this lane as committed capacity, not a competitor;
its server-side `cursor.*` restricted surface work is a natural neighbor of
W1.1 bridge work — sequencing note in §9.

## 9. Sequencing + parallelization

```
now ──► W0.1 doctor/build-id ─┐
        W0.2 ghostty packaging ├─► W0.3 fresh install + operator verify ─► W0.4/W0.5
        (parallel, small)     ┘
                                          │
Cursor lane (lifo, continuous) ───────────┼──────────────►
                                          ▼
        W1.1 bridge parity ─► W1.2 browser panes
        W1.3 agent sidebar (independent — parallel to W1.1)
        W1.4 restore pack   (independent — parallel to W1.1)
        W1.5 mux spike      (anytime, feeds W1 PRD reviews)
```

- W0.1 + W0.2 are small, independent, and unblock W0.3 (you can't verify
  what you can't identify). W0 total is deliberately thin — days, not weeks.
- W1.1 is the single ordering constraint in W1: browser (W1.2) depends on it;
  W1.3/W1.4 don't. Coordinate W1.1 with lifo's Cursor task #6 — both touch the
  bridge/method-registry surface; land Cursor's restricted surface first or
  co-design the method-registry shape once.
- Each W-item maps to one PRD + one TaskMaster import, executed by lifo +
  subagents per the agreed handoff shape (repo PRD docs + TaskMaster-ready
  tasks with executable acceptance).

## 10. Proposed PRD list (Wave-0 + Wave-1)

| PRD | Covers | Size |
|---|---|---|
| PRD-A `runtime-trust` | W0.1 doctor/build-id + W0.4 log hygiene | M |
| PRD-B `ghostty-packaging` | W0.2 resources/terminfo + installer self-verify | S/M |
| PRD-C `verify-loop` | W0.3 operator verification checklist + status write-back (mostly process + a little tooling) | S |
| PRD-D `pane-attention` | W0.5 attention border + color flags (#20) — includes the tab.action live-bridge route that doesn't exist yet | M |
| PRD-E `bridge-parity` | W1.1 ControlState fall-through | M/L |
| PRD-F `browser-live` | W1.2 browser panes + allowlist/audit (contains the WebKitGTK-vs-CDP decision) | L |
| PRD-G `agent-sidebar` | W1.3 lifecycle states + sidebar scalability | M |
| PRD-H `restore-pack` | W1.4 session restore correctness | M |
| — | W1.5 mux spike: not a PRD — a timeboxed research task with a decision-doc deliverable | XS |

Wave-2/3 PRDs are cut only after Wave-1 lands (re-check cmux upstream then —
it moves ~95 PRs per 4 days; the research db refresh is cheap and scripted).

## 11. Verification discipline (applies to every PRD)

- Executable acceptance only: `./scripts/check.sh`, targeted `cargo test`,
  Xvfb smoke additions, and — where operator-visible — a written live
  checklist whose results get recorded to FYI/TaskMaster (closes the
  §E9 verification-loop gap that let #14/#15 sit unconfirmed).
- Every PRD names its channel targeting (stable vs preview) and its
  build-id expectations once W0.1 lands.
- WSLg noise classification from W0.4 is the triage baseline: EGL/Zink/popup
  warnings are benign-env unless correlated with a reproducible failure.
