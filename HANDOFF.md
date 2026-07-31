# Limux — Directory State (session-agnostic)

**Updated:** 2026-07-31 by `bari` (LIMUX_MGR) — shared `main` after Merge #112
(`83cb928` lineage; hygiene + #110/#111/#112 continuity). Open PR: **#109**
(`bari/yield-abc` @ `7d9bfb4`) — unknown-flag reject, byte-safe `send`,
display-loss diagnostic.
TaskMaster reconciled; planned stale sockets + `/tmp` limux test debris archived
(not deleted). Product backlog lives in `BARI_HANDOFF.md` and
`docs/LIMUX_FASTFOLLOWS_2026-07-29.md` (not only `TUTU_HANDOFF.md`).
Doctor now fully green (`stale_sockets` `[ok]`); historical leave-alone `52458`
paths are **gone** (do not recreate/re-archive).

**Scope:** `/home/riche/MCPs/limux`
**Purpose:** ONE file that tells ANY session — not just the current manager —
what is going on in this directory. Read this first. Per-session detail lives in
the per-session handoffs listed in §7.

> Supersedes the halo/Codex handoff of 2026-06-20. Halo is retired under the
> fleet-wide Codex close-out. The original is preserved in git history —
> `git show f3c95a5:HANDOFF.md` — which is its durable record; a convenience
> copy also sits at `archive/HANDOFF_halo_2026-06-20.superseded.md`; `archive/` is
> tracked (un-ignored by `61a0f36`), so that copy is durable in git too — though git
> history remains the stale-proof record (its truth does not depend on ignore config).

---

## 1. STATUS — host healthy; hygiene closed; yields open as PR #109

**Open product PR:** [#109](https://github.com/RichelynScott/limux/pull/109) on
`bari/yield-abc` tip `7d9bfb4` (TaskMaster #25/#33/#34). Shared `main` stays
docs/hygiene-only until merge — product symbols live on the PR branch.


**Host is UP.** Live control sockets remain
`/run/user/1000/limux/stable/limux.sock` (+ `.cursor`); host PID still live.
`limux doctor` (re-checked 2026-07-31): launchers / processes / socket /
`stale_sockets` / ghostty all `[ok]` — doctor exit 0.

**Socket archive (plan step 1, historical):** the planned doctor-stale set of
**four** sockets (`limux.sock`, `limux.cursor.sock`, `stable/limux-85224.sock`,
`stable/limux-85224.cursor.sock`) was archived same-tmpfs under
`/run/user/1000/limux-socket-archive/20260731T013021Z/` (unix sockets cannot
`mv` across filesystems; home-side manifests under
`~/.local/state/limux/socket-archive/`). After that archive, doctor briefly warned
on `stable/limux-52458{.cursor,}.sock`. Plan line 44 required **stop and record;
do not broaden**. An unauthorized broaden archived those two briefly; they were
**restored** (`RESTORE.txt` under `…/20260731T014108Z/`). As of the later
continuity re-check those `52458` paths are **absent**, live sockets are only the
two stable listeners above, and `stale_sockets` is `[ok]`.

**Still not live-verified in the GUI:** the OMP scroll fix (#82 + #106
`ScrollMods=0`) and #84's resize behaviour have not been *observed* working —
only gate-verified and source-traced. Coordinate with the operator before poking
live panes.

**Open product work** is listed in `BARI_HANDOFF.md`: in-flight yields on
PR #109 (#25/#33/#34), plus still-open H1 entitlement option (b),
successor-rebind, prune fast-follows, live GUI verify, and OMP plan-review
sidebar visibility as cmux-parity `7.3`.

---

## 2. WHAT IS INSTALLED / RUNNING

| | Value |
|---|---|
| **Installed + running** | `limux-cli 0.2.3 (c757056d2539, release)` — clean, no `-dirty`; install-id `main-c757056d2539-adv-remediated-20260721`, channel `stable` |
| **Contains** | The reviewed PRs through the 2026-07-21 adversarial remediation (#81–#87 family on that install) |
| **NOT in the running build** | Everything merged after `c757056`, including #88/#90 (bounded-logging), #105 (hook-log flock + ghostty `CARGO_MANIFEST_DIR` runtime read), #106 (wheel ScrollMods), #107 (H1 surface scoping), #108 (packaging rename-not-delete). **~50+ commits behind main.** Reinstall is operator-gated — documenting the lag is in scope; reinstall/restart is not part of this packet. |
| **Previous launchers** | archived via `mv` (not `rm`) at `~/.local/limux-reviewed/archive/20260721T223224Z/` |

---

## 3. WHAT IS MERGED ON MAIN

main after Merge #112 (`83cb928` = docs SHA sync; prior #111/`990b198`,
#110/`e07dd2c`; hygiene via `5649457` / `bc99b45` / `b018ff9`). Continuity
commits intentionally cite the merge lineage rather than their own tip SHA.
Last known full Rust gate green from fire's closeout lane (`./scripts/check.sh`
exit 0). Docs-only continuity does not re-run the Rust gate.

> ## PR #92 (named session profiles) — merged, reverted, re-landed (history)
>
> **Past tense:** #92 merged, hit a launcher-reachability P1, was **reverted via
> PR #96**, then **re-landed as PR #99** (orthogonal channel/profile model +
> generated-launcher test + channel-as-path sanitizer). Do not treat the old
> "BEING REVERTED" framing as current work. See git history around `4e625bf` /
> `e157c90` / #99 for the full review record if needed.

Notable merges after the installed `c757056` tip (non-exhaustive; tip is
`ef4d376`):

| PR / commit | Content |
|---|---|
| #81–#87 | Stranded audit, H1/scrollbar/resize/dirty-marker, adversarial remediation |
| #88 / #90 | Bounded host logging + #88 regression fix |
| #89 | Three-state build-provenance test |
| #99 | Session profiles v2 re-land (post-#96 revert of #92) |
| #102–#104 | Retention / hygiene / FYI consolidation (2026-07-29 space-crisis cycle) |
| #105 `a520e4d` | Agent-hook log rotation flock + ghostty build-script relocation-proof |
| #106 `51d9e97` | Discrete wheel `ScrollMods=0` |
| #107 `05836c4` | Surface content reads scoped to focused workspace (H1 partial) |
| #108 `2cceb95` | Packaging: stop deleting `/usr/local` as root; rename-not-delete |
| fast-follows doc | `docs/LIMUX_FASTFOLLOWS_2026-07-29.md` — §3–§5 CLOSED; §1–§2, §7–§9 OPEN |

### Mechanism notes still worth keeping (abbrev.)

1. **H1 — cross-lane disclosure.** Partial close in #107; **residual CRITICAL**
   remains for explicit foreign `workspace_id` (fast-follow §7 / design note).
2. **OMP scrollbar peg/flash.** Layout-sibling width flip; #82 layout-neutral +
   #106 ScrollMods. Live GUI observe still outstanding.
3. **#84 sub-cell resize deferral.** Gate-verified; live observe outstanding.

---

## 4. OPEN WORK

| Item | State |
|---|---|
| **Unknown CLI flags** | OPEN — reject before socket contact (`TUTU_HANDOFF` item-2 / `BARI_HANDOFF` #1) |
| **CLAUDE.md checklist card lines** | OPEN — limux-local piece (`BARI_HANDOFF` #2) |
| **H1 residual CRITICAL** | OPEN — explicit foreign `workspace_id`; operator-gated option (b) (fast-follow §7) |
| **Display-loss exit diagnostic** | OPEN — bare status 1 on WSLg display reset (fast-follow §8) |
| **Successor-rebind control path** | OPEN — unclean restore leaves suspended predecessor identity (fast-follow §9) |
| **Prune `--keep` cap + TOCTOU** | OPEN — limu lane (fast-follow §1–§2) |
| **Live GUI verify** | OPEN — OMP scroll + #84 resize still never operator-observed |
| **OMP plan-review / ask sidebar visibility** | OPEN — cmux-parity **7.3** after native PRD-G wiring; decision `LIMU_INBOX/RESPONSE_FROM_limu_2026-07-30_omp-ask-waiting-abc-decision.md` |
| **Installed runtime lag** | DOCUMENTED — still on `c757056`; reinstall operator-gated |
| **26 GB archived legacy log** | Still on disk under archive-not-delete; delete is operator call |
| **Standing adversarial residuals** | M-1/M-3/M-5/L-2/L-3/L-4 still open per `docs/ADVERSARIAL_REVIEW_FINDINGS_2026-07-21.md` |
| **✅ Hygiene 2026-07-31** | CLOSED for docs/TaskMaster/`/tmp` debris + **planned** 4-socket archive; doctor still warns on recorded `limux-52458.*` (plan stop — not broadened) |

---

## 5. PR #68 — merge done, three fixes NOT implemented

A prior agent completed the *merge* onto main. The three fixes below were
verified by direct execution in an earlier session but are **not written**:

- **P2 — stderr fd hijack (data integrity).** With fd 2 closed, `pipe()` returns
  `(2,3)` — the read end lands on fd 2. `dup2` then destroys the drain thread's
  reader; the thread exits, drops its `File`, and closes fd 2 entirely. The next
  `open()` claims fd 2, so later stderr writes **silently corrupt an unrelated
  file**. Fix: `reserve_standard_fds()` + `relocate_above_stderr()` +
  `pipe2(O_CLOEXEC)`.
- **A1 — GUI hang (highest severity).** If the log sink fails, the drain loop
  `break`s and nobody drains the pipe. Once the 64KiB buffer fills, a write from
  the GTK main thread blocks **forever** = full GUI freeze. Fix: never stop
  draining while the write end is open; discard on sink failure, mark degraded.
- **A2 — silent permanent log death.** At the byte cap `write_bounded` returns
  `Ok(false)` silently and rotation is startup-only, so the log dies permanently
  and silently. Fix: make the cap observable.

### ✅ IMPLEMENTED AND MERGED — `d8e7648` (PR #88)

All three landed. Gate green, **655 passed / 1 ignored**.

**The stated P2 mechanism above was WRONG, and the implementer caught it.** The
pipe read end is *not* the first victim: `prepare_host_logging()` opens the
managed **log file** before any pipe exists, so with fd 2 closed the *log file*
takes it and `dup2` clobbers the sink. The first fix attempt — following the
mechanism as written above — was a **silent no-op**, caught only empirically
(a probe `write` returned 15 bytes while the log stayed 0 bytes). Reservation
now runs before that open. *This was the fourth wrong root cause of the day; it
came from this handoff and was passed down unverified.*

**A1's hang is coupled to P2.** In an isolated pipe, stopping the drain yields
`EPIPE`, not a hang (Rust ignores `SIGPIPE`). The freeze requires another holder
of the read end — which existed only because the pipe lacked `O_CLOEXEC` and
children inherit it.

**Tests are load-bearing, independently verified.** I reverted the A1 fix myself:
both `sink_failure_*` tests fail and the run blocks the full **15.00s** — a real
writer hang, not a assertion tweak. Restored cleanly, 0.05s.

⚠️ **NOT INSTALLED into the operator's build.** This is `unsafe` fd manipulation
in a live GUI app with no adversarial review and no GUI run. An adversarial pass
is running; install only after it clears. `install_survives_a_closed_stderr_and_keeps_logging`
is `#[ignore]`d (it hijacks process-wide stderr) — run deliberately with
`cargo test -p limux-host-linux install_survives -- --ignored --test-threads=1`.

---

## 6. nava's hcom-TUI × Limux design question — owner findings

Full input: `LIMU_INBOX/DESIGN_QUESTION_FROM_nava_2026-07-21_*.md`.

**CORRECTION to an earlier claim in that file** (verified 2026-07-21): it stated
"the focus primitive exists", citing `control_bridge.rs`. That is only half
true. `pane.focus` and `surface.focus` exist at the **protocol** layer, but
there is **no CLI verb** exposing them — `limux --help` has no focus command. So
nava's seam B (hcom ranks, Limux focuses) is **not** as thin as she was told: it
needs either a new CLI verb or a direct socket client. Seam A is unaffected.

**Still true and verified:** `limux pane-action --action set_flag_color --color
<...>` and `clear_flag_color` ARE shipped CLI verbs, so seam A (attention → pane
chrome) needs no new rendering work from Limux.

**Ratified shape — thin contract, no cross-imports; the agent self-reports the
mapping.** Limux already injects `LIMUX_SURFACE_ID` and `LIMUX_WORKSPACE_ID`
into pane env (verified — the CLI reads them for workspace scoping), so an agent
inside a Limux pane already knows its own ids and can register them with hcom.
hcom then ranks urgency and shells the public CLI. Neither system imports the
other, and neither takes a runtime dependency on the other. It also degrades
correctly: an agent with no `LIMUX_SURFACE_ID` is simply not Limux-hosted —
which structurally resolves seam C's scoping caveat, because Limux's silence
about such an agent is then not evidence of death.

---

## 7. PER-SESSION HANDOFFS AND OWNERSHIP

| File | Whose | Contains |
|---|---|---|
| `BARI_HANDOFF.md` | **bari (current LIMUX_MGR)** | live resume surface — open backlog + hygiene closeout |
| `TUTU_HANDOFF.md` | tutu (historical) | 2026-07-29 cycle-close; OPEN item-2/3/H1 still accurate; successor banner points here → bari |
| `FIRE_HANDOFF.md` | fire (historical / adjacent lane) | space-crisis + fast-follow authoring; attribution FIRE fallback notes |
| `LIMU_HANDOFF.md` | limu (historical; may co-claim Codex lane when active) | prior Codex-lane history |
| `LIFO_HANDOFF.md` | lifo (retired) | earlier lane; peer-owned, do not edit |
| `git show f3c95a5:HANDOFF.md` | halo (retired) | the 2026-06-20 state, verbatim (git history is the durable copy) |

Lineage: lifo → limu → tutu → **bari** (2026-07-31). Related lanes: `karo` /
`rako` = OMP, `nava`/`dino` = hcom, `reve` = fleet. `limu` may still co-claim
`LIMUX_CODEX_MGR` when active (non-superseding).

**Active goal (unchanged from halo, still correct):** improving Limux as the
tool the operator actually uses. The old Project Isolation Lab / VM goal is NOT
this repo's workstream.

---

## 8. TRAPS — learned the hard way, do not relearn

- **🔴 THIS CHECKOUT IS SHARED — do NOT `git checkout`/branch-switch it while a
  peer is live in it.** 2026-07-25: `huno` was live-editing
  `feat/named-session-profiles` in `/home/riche/MCPs/limux` while tutu was also
  operating there. tutu's orientation `git checkout main` (no clean-tree
  preflight) bumped huno's HEAD off their branch, carrying their uncommitted
  work onto main. Restored (huno's work proven intact by sha256), but the fix is
  structural: **two sessions branch-switching one checkout is the exact
  worktree-hygiene hazard.** Arrangement going forward — **huno owns the main
  checkout for the profile work; tutu (review/coordination) commits HANDOFF/docs
  via an EPHEMERAL worktree off `origin/main`** (`/tmp/worktrees/…`, push,
  `git worktree remove`), never touching the live checkout's HEAD. Run the
  new-work-lane preflight (`git status`/`branch --show-current`) BEFORE any
  branch op here.
- **Task state is PER-BRANCH.** A TaskMaster tag that looks "missing" is usually
  a branch-view difference, not data loss. This caused a false alarm.
- **Work strands on branches.** Three separate efforts were stranded on unmerged
  branches this session, and a background agent died with unpushed work.
  **Push immediately after every commit** — do not batch.
- **`git stash -u` sweeps the peer-owned untracked file.** It is untracked, so
  `-u` takes it. I did this and had to `git stash pop` to put it back. Prefer
  committing to a branch over stashing, or stash without `-u`.
- **Subagents die.** Of five background agents this session, three died to a
  session quota limit and one to process exit — losing unpushed work each time.
  Brief them to push after every commit, and check their worktrees for salvage
  before assuming a task never ran.
- **Never hand-edit `.taskmaster/tasks/tasks.json`** — use
  `task-master-reviewed` (`--title`/`--description` = manual, no LLM cost).
  Note `task-master-ai-reviewed` refuses non-AI subcommands like `list`.
- **`docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` is
  peer-owned untracked dirt** — do NOT stage, modify, or remove it.
- **`/tmp/limux-release-0.2.3-20260719`** — hamo's no-loss hold; do not remove
  without explicit operator release.
- **Vendored `ghostty/` is READ-ONLY.** Work through the C API.
- **Clippy `-D warnings` is a hard gate** and it *will* catch things. Fix, never
  suppress.
- **rtk rewrites some commands.** `grep`/`rg` output gets compacted ("N matches
  in M files") and `cargo test` collapses to a single summary line. If you need
  raw output use `awk`/`sed`/`tail`, and note `--type`/`--include` may not reach
  the real binary.
- **papa-git**: `export CLAUDE_SESSION_NAME=BARI_LIMUX_MGR CLAUDE_AGENT=claude`
  in *every* bash call that commits — it does not persist between tool calls,
  and lowercase names are REFUSED (`^[A-Z0-9_-]{1,50}$`).
- **Beware `cmd | tail -N` in a background job** — only the tail is saved, so
  the full log is lost. Redirect to a file instead if you need the whole thing.
- **The Codex PR bot is not reviewing** (fleet-wide Codex close-out), so PRs
  merge with 0 reviews / 0 checks. Weigh that when merging.

---

## 9. WHAT IS NOT CLAIMED

- **No live GUI verification of anything.** All five fixes are gate-verified
  (tests/clippy/fmt) and source-traced. Only #33 was additionally verified
  end-to-end by execution (building and reading `--version` across three tree
  states). The OMP scroll fix and #84 have **not** been observed working in a
  running terminal.
- #84's premise that a changed `ws_xpixel` raises `SIGWINCH` is reasoned from
  ghostty source, **not** confirmed against the kernel.
- The 2026-07-16 renderer evidence carried into #83 was produced by different
  binaries and is **not re-attested**.
- PRs #82–#85 merged with **no external review**, and the standing adversarial
  review never ran (see §4). Treat them as self-reviewed only.
