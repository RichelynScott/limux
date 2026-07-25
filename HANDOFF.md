# Limux — Directory State (session-agnostic)

**Updated:** 2026-07-21 ~6:35 PM EDT by `tutu` (LIMUX_MGR)
**Scope:** `/home/riche/MCPs/limux`
**Purpose:** ONE file that tells ANY session — not just the current manager —
what is going on in this directory. Read this first. Per-session detail lives in
the per-session handoffs listed in §7.

> Supersedes the halo/Codex handoff of 2026-06-20. Halo is retired under the
> fleet-wide Codex close-out. The original is preserved in git history —
> `git show f3c95a5:HANDOFF.md` — which is its durable record; a convenience
> copy also sits at `archive/HANDOFF_halo_2026-06-20.superseded.md`, but note
> `archive/` is gitignored, so that copy is local-only.

---

## 1. STATUS — restart DONE, host healthy (was the top operator item)

**The 2026-07-21 restart happened.** As of **2026-07-25 ~3:57 PM EDT** the host
is **UP** on the installed `c757056` build (2 host processes), and **`limux
doctor` is fully green** — launchers, processes, socket, stale-sockets, ghostty
resources all `[ok]`. The launcher-drift `[fail]` cleared on restart exactly as
predicted.

So the seven reviewed PRs (#81–#87) are **live**. The OMP scrollbar root-cause
fix is running.

**Still not live-verified in the GUI:** the OMP scroll fix and #84's resize
behaviour have not been *observed* working — only gate-verified and source-traced.
The host is now up, so verification is possible, but it means interacting with
the operator's live panes — coordinate with the operator + `karo` (OMP_MGR)
before poking the running host.

---

## 2. WHAT IS INSTALLED / RUNNING

| | Value |
|---|---|
| **Installed + running** | `limux-cli 0.2.3 (c757056d2539, release)` — clean, no `-dirty`; install-id `main-c757056d2539-adv-remediated-20260721`, channel `stable` |
| **Contains** | The SEVEN reviewed PRs #81–#87 (adversarial-review remediation included) |
| **NOT in the running build** | #88/#90 (bounded-logging) merged to main *after* this install. They are internal logging fixes; #88 alone was regressive but #90 fixed it. Reinstalling from main would pick them up — but there is no pressing reason to, and it would cost the operator a restart. Leave until there is one. |
| **Previous launchers** | archived via `mv` (not `rm`) at `~/.local/limux-reviewed/archive/20260721T223224Z/` |

---

## 3. WHAT IS MERGED ON MAIN

main @ `fc40cf5` — gate green: `./scripts/check.sh` exit 0, **660 passed / 0
failed / 0 ignored**, clippy `-D warnings` + fmt clean. (The previously-ignored
fd test was deleted with the unreachable fd layer in #90.)

> **PR #92 (2026-07-25) — `huno`'s named session profiles — REVIEWED, tutu APPROVE
> pending levu's boundary sign-off.** `RuntimeChannel::Profile(name)` + per-profile
> socket/session.json + `limux --profile`/`profile list|path|rm`. Head `4b68c4a`.
>
> - **HIGH-1 (found + fixed): auto-profile allocation data-loss TOCTOU.** The
>   original allocator was check-then-bind (probe socket free → adopt `auto-N`),
>   with a huge window between allocation (`main.rs:444`) and socket bind
>   (`window.rs:3219`); the loser's bind-fail is non-fatal (`control_bridge.rs:1664`)
>   so it kept running on the contended profile and clobbered the winner's
>   `session.json` → workspaces silently lost. **huno found it independently, and
>   tutu's adversary found the identical race** — two lenses converged. Fixed with
>   `AutoProfileClaim`: an flock (`LOCK_EX|LOCK_NB`) reservation taken at allocation
>   time, held for process life, `O_CLOEXEC`/`O_NOFOLLOW`/0600, runtime-dir. tutu
>   source-verified the fix closes the race at root (covers the pre-bind window the
>   probe can't see). **Evidence caveat:** tutu did NOT execute the host-crate flock
>   test (fresh-worktree ghostty unbuilt; safe workaround tripped the rm-guard near
>   huno's live checkout). Rests on source review + test read + deductive
>   load-bearingness + huno's reported pass/M4b-fail + the adversary's confirmation.
> - **LOW-2 (open, huno's disposition — NOT a blocker):** `profile rm` is
>   check-then-rename (`main.rs:505-524`) — small two-syscall TOCTOU; a host starting
>   `<name>` between check and archive gets its dir archived out from under it.
>   huno to fix (flock) or accept-with-rationale like the doctor gap.
> - **Cleared** (tutu's hands + adversary, falsified each): sanitizer allowlist,
>   mode-derived owner_only (#86 intact), 0600 load-bearing, no socket↔session
>   drift, single-source-of-truth via `session_paths`, `profile rm` archive-not-delete
>   + sanitized path. Test quality: adversary reverted 4/7 (all failed), tutu
>   reverted the 0600 pin (failed) — no decorative tests.
> - **Still open (acknowledged, not blockers):** launcher-drift reasoned-not-proven-live;
>   `doctor` does not enumerate profiles (tutu's design call, documented gap).
> - **Merge path:** code-ready; gates on levu's boundary-lint (DP-7 — `layout_state.rs`
>   path-gate). Full verdict + checklist sent to huno via hcom.

| PR | Content |
|---|---|
| #81 `70689b4` | limu's stranded audit + `LIMU_INBOX` + TaskMaster reconciliation |
| #82 `08abec1` | H1 cross-lane disclosure · `hook_session_id` misattribution · **OMP scrollbar peg/flash root cause** · PR #58 attestation salvage |
| #83 `a5c0f98` | PR #67 renderer backend diagnostics rebuilt + socket-mode P2 |
| #84 `f2b0a79` | TaskMaster #29 sub-cell resize deferral (word-wrap on width change) |
| #85 `3bf819f` | TaskMaster #33 build dirty-marker — untracked files no longer mark builds dirty |
| #86 `c757056` | Adversarial remediation: H-1 read-screen surface scoping · M-2 foreign-repo provenance guard · M-4 socket-mode fail-open · L-1 pipe-pane empty stream · send-key honest diagnosis |
| #87 `149e283` | Durable record of the adversarial findings |
| #88 `d8e7648` | Bounded host logging: **A1 GUI-hang** · P2 stderr-fd · A2 silent cap. Shipped a shutdown data-loss regression — **fixed by #90**. |
| #90 `51e8144` | **Fixes #88's regression.** H1 stderr-loss-at-exit (flush barrier + `atexit`) · H2 installer `O_CLOEXEC` test · **H3 deleted ~90 lines of unreachable `unsafe` fd code** · M2 detach guard. 660/0/**0** |
| #89 `9f469c1` | Three-state build-provenance test (closes the #33 gap) |

### The five defects, with mechanism (not just names)

1. **H1 — cross-lane information disclosure.** `read-screen --help` fell through
   to `surface.read_text` with no target; the server's global-focus fallback
   returned *another agent's pane content, including in-flight command text*.
   `read-screen` was also the only one of read-screen/send/send-key with **no
   `LIMUX_WORKSPACE_ID` fallback**, so it defaulted to global focus
   unconditionally. Corroborated by reve's 2026-07-19 incident.
2. **`hook_session_id` misattribution.** Preferred ambient `CLAUDE_SESSION_ID`
   over the payload's own `transcript_path`, and `limux_env_value` walks
   **ancestor process env** — so hook events for session A were attributed to
   whoever invoked the CLI. Decision recorded: **payload-first**. This also
   repaired a **non-deterministic quality gate** (the test passed under Codex,
   failed under Claude — which is why limu's audit recorded 597 passing while
   `CLAUDE.md` warned the same test failed; *both were correct*).
3. **OMP scrollbar peg/flash.** The scrollbar is a **layout sibling** of the
   terminal (`root.append(&overlay); root.append(&scrollbar)` in a horizontal
   Box). In GTK4 an invisible box child gets **zero allocation**, so every
   `total > len` flip changed GLArea width by ~13px → `connect_resize` →
   `ghostty_surface_set_size` → **column change** → reflow — and reflow moves a
   scrolled-back viewport to the active area. Ghostty's own comment on that
   path: *"this effectively pulls down scrollback"* — the operator's verbatim
   symptom. **limux-specific**: upstream ghostty uses `GtkScrolledWindow`
   overlay scrolling, which is layout-neutral by construction. Fix: layout
   participation is decided by config only; scroll state varies opacity and
   hit-testing only.
4. **PR #67 socket-mode P2.** The preview runner passed an inherited
   `LIMUX_SOCKET_MODE`/`CMUX_SOCKET_MODE` into the host, but its probe CLIs are
   children of the **runner**, so `is_descendant` rejected every probe and
   healthy backends were misreported unhealthy. Now cleared + forced
   `localUser`.
5. **#33 build dirty-marker.** `build.rs` computed dirtiness from
   `git status --porcelain`, which counts **untracked** files — so a clean
   release build was stamped `-dirty` because of one untracked peer-owned docs
   HTML. A second, latent defect in the same code: `command_stdout` folds empty
   output into `None`, making the `"false"` arm unreachable, so a clean tree
   reported `unknown` rather than *verified clean*. Both fixed; verified by
   execution in three tree states.

---

## 4. OPEN WORK

| Item | State |
|---|---|
| **Restart** | 🔴 operator — §1 |
| **🔴 26 GB of logs — operator decision** | `~/.local/state/limux/logs/limux-host.log` was **26 GB** — the legacy *unbounded* log (pre-bounded-logging; current code writes `limux-host.current.log`, and the codebase references that old name only as a test fixture called `legacy_incident`). Verified stale + unheld, then **archived via `mv`** to `logs/archive/limux-host.log.legacy-unbounded-superseded-20260721`. Live logs are now **112 KB**. **It still occupies 26 GB — deleting it is the operator's call** (archive-not-delete floor). Relevant to the C-drive-space lane. |
| **M1 — retained logs never pruned** | Real but **not yet triggered**: production is 64 MiB active / **10** retained / 640 MiB total, and the `retained/` dir does not exist yet (count 0). At the limit `rotate_managed_active` returns `StderrFallback`, so host logging degrades rather than erroring — and `doctor` does not check the retained budget. Needs ~640 MiB of host stderr to bite. |
| **Live verification** | Nothing below has been seen working in a running GUI. After restart, verify the OMP scroll fix and #84's resize behavior. For #84, `strace -e ioctl` `TIOCSWINSZ` counts on a slow split-drag (before/after) settles its one unverified claim. |
| **Standing adversarial review** | ✅ **RAN** (4th attempt). Found 3 HIGH / 5 MED / 4 LOW. Full record: **`docs/ADVERSARIAL_REVIEW_FINDINGS_2026-07-21.md`**. H-1/H-2/M-2/M-4/L-1 fixed in PR #86; M-1, M-3, M-5, L-2/L-3/L-4 **still open**. |
| **⚠ Test theater (highest-value remediation)** | The reviewer **reverted each fix and re-ran the suite**: **4 of the 5 behavioural fixes in the installed build survive a full revert with a green suite.** Pure-logic helpers are tested; the wiring that uses them is not. Only `hook_session_id` ordering and #84's grid predicate fail on revert. |
| **✅ STANDING CHECK — adopt this** | **Revert the call site (not the helper), re-run the suite, confirm something fails. If nothing does, the test is decorative.** Any fix shaped "extract a helper, call it from one site" has this hole by default — the helper test proves the helper works, nothing proves it is *reached*. Generalized with the hcom lane, which found the identical shape in a release it had already shipped. |
| **Mutation-verified as load-bearing** | H-1 read-screen scoping (revert → 1 fail) · M-4 socket fail-open (revert → 1 fail) · A1 GUI-hang (revert → 2 fail + a real **15.00s** writer hang) · send-key `enter`→`Return` (revert → 1 fail) · #84 grid predicate · `hook_session_id` ordering. |
| **✅ CLOSED — #33 test gap** | Found by running the standing check on my *own* work: #33 shipped with **no test**, so reverting `-uno` left the suite green. Closed by PR #89 (`9f469c1`), which pins the three-state semantics — `"false"` → `Some(false)` *verified clean*, `"unknown"`/absent → `None` *cannot attest*, `-dirty` only for `Some(true)`. Mutation-verified. Deliberately avoids `from_compile_env` (it calls `install_info_near_current_exe()`, which would make a provenance test depend on what sits beside the test binary — the same flake class as the old `hook_session_id` test). |
| **M-1 — scrollbar fix has a live residual path** | The fix's own test comment claims *"config is constant for the surface lifetime"*. **False** — `GHOSTTY_ACTION_RELOAD_CONFIG` stores `CURRENT_SCROLLBAR_ENABLED` at runtime. A config reload while scrolled back still drops the scrollbar out of layout → GLArea widens → column change → viewport reset. **This is the operator's own scroll-yank symptom, via the one remaining path.** |
| **PR #86** | ✅ **MERGED** `c757056`. DP-7 boundary review was **granted by two independent reviewers** (`gire` + `nava`), both of whom reproduced the false positive rather than taking my word; `boundary-reviewed` label applied. NOT self-certified even though HCOM_MGR was stale and the operator directive would have permitted it. |
| **Boundary-lint narrowing** | Tracked by the hcom lane, deliberately **not** shipped. Note for whoever picks it up: the obvious `grep -Ew` fix is a **trap** — `_` is a word constituent, so `-w` would break the `HCOM_` prefix token and silently disable most of the gate. Prefix / identifier / bare-word tokens each need different treatment. |
| **PR #68 / #88 bounded logging** | ✅ **MERGED** `d8e7648` — all three fixes (A1 GUI-hang, P2 fd hijack, A2 silent cap). Gate green, **655 passed / 1 ignored**. ⚠️ **NOT INSTALLED**: it is `unsafe` fd manipulation with no adversarial review, so a review is running before it reaches the daily driver. See §5. |
| **reve new-pane incident** | Needs a **v0.2.3 retest**; filed against legacy 0.2.2. `LIFO_INBOX/INCIDENT_FROM_reve_2026-07-19_*.md` |
| **nava design question** | hcom-TUI × Limux symbiosis. Design owner = the Limux manager. §6 has the ratified shape + a correction. |
| **ghostty wheel/mouse-reporting** | Wheel events eaten with **no alt-screen gate** (`Surface.zig:3601-3621`). Vendored ghostty is **read-only — do NOT patch**. karo advised checking the shift+wheel escape hatch first. An upstream issue must **not** be filed externally without operator approval. |

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
| `TUTU_HANDOFF.md` | tutu (current LIMUX_MGR) | this session's detail |
| `LIMU_HANDOFF.md` | limu (retired) | prior-lane history |
| `LIFO_HANDOFF.md` | lifo (retired) | earlier lane; peer-owned, do not edit |
| `git show f3c95a5:HANDOFF.md` | halo (retired) | the 2026-06-20 state, verbatim (git history is the durable copy; `archive/` is gitignored) |

Lineage: lifo → limu → **tutu** (all 2026-07-21). Related lanes: `karo` =
OMP_MGR (`~/Proj/oh-my-pi`), `nava`/`dino` = hcom, `reve` = fleet.

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
- **papa-git**: `export CLAUDE_SESSION_NAME=TUTU_LIMUX_MGR CLAUDE_AGENT=claude`
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
