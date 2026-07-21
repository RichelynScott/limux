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

## 1. 🔴 THE ONE THING THAT NEEDS THE OPERATOR

**The Limux host is DOWN. The operator restarts it by typing `limux` in a
Windows terminal.** Nothing shipped is live until then.

Verified 2026-07-21 ~18:2x EDT: no host process, and
`/run/user/1000/limux/stable/limux.sock` does not exist.

**The usual restart cost is already paid.** A host restart normally kills every
hosted pane process (pane shells are direct children of the host; layout and cwd
restore, process state does not). The host is already down, so there is nothing
left to lose — restarting now costs nothing.

After restart: expect `limux doctor` to go green, then re-check the OMP
scrollbar behavior with `karo` (OMP_MGR).

---

## 2. WHAT IS INSTALLED RIGHT NOW

| | Value |
|---|---|
| **Installed** | `limux-cli 0.2.3 (c757056d2539, release)` — clean, no `-dirty` |
| **install-id** | `main-c757056d2539-adv-remediated-20260721`, channel `stable` |
| **Contains** | ALL SEVEN merged PRs (#81–#87), including the adversarial-review remediation |
| **Previous launchers** | archived via `mv` (not `rm`) at `~/.local/limux-reviewed/archive/20260721T223224Z/` |

`limux doctor` currently shows two `[warn]`s — "no running Limux host process"
and "socket not connectable". **Both are correct and expected while the host is
down.** They clear on restart. Do not "fix" them.

---

## 3. WHAT IS MERGED ON MAIN

main @ `3bf819f` — gate green: `./scripts/check.sh` exit 0, **620 passed / 0
failed**, clippy `-D warnings` + fmt clean.

| PR | Content |
|---|---|
| #81 `70689b4` | limu's stranded audit + `LIMU_INBOX` + TaskMaster reconciliation |
| #82 `08abec1` | H1 cross-lane disclosure · `hook_session_id` misattribution · **OMP scrollbar peg/flash root cause** · PR #58 attestation salvage |
| #83 `a5c0f98` | PR #67 renderer backend diagnostics rebuilt + socket-mode P2 |
| #84 `f2b0a79` | TaskMaster #29 sub-cell resize deferral (word-wrap on width change) |
| #85 `3bf819f` | TaskMaster #33 build dirty-marker — untracked files no longer mark builds dirty |

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
| **Live verification** | Nothing below has been seen working in a running GUI. After restart, verify the OMP scroll fix and #84's resize behavior. For #84, `strace -e ioctl` `TIOCSWINSZ` counts on a slow split-drag (before/after) settles its one unverified claim. |
| **Standing adversarial review** | ✅ **RAN** (4th attempt). Found 3 HIGH / 5 MED / 4 LOW. Full record: **`docs/ADVERSARIAL_REVIEW_FINDINGS_2026-07-21.md`**. H-1/H-2/M-2/M-4/L-1 fixed in PR #86; M-1, M-3, M-5, L-2/L-3/L-4 **still open**. |
| **⚠ Test theater (highest-value remediation)** | The reviewer **reverted each fix and re-ran the suite**: **4 of the 5 behavioural fixes in the installed build survive a full revert with a green suite.** Pure-logic helpers are tested; the wiring that uses them is not. Only `hook_session_id` ordering and #84's grid predicate fail on revert. |
| **M-1 — scrollbar fix has a live residual path** | The fix's own test comment claims *"config is constant for the surface lifetime"*. **False** — `GHOSTTY_ACTION_RELOAD_CONFIG` stores `CURRENT_SCROLLBAR_ENABLED` at runtime. A config reload while scrolled back still drops the scrollbar out of layout → GLArea widens → column change → viewport reset. **This is the operator's own scroll-yank symptom, via the one remaining path.** |
| **PR #86** | ✅ **MERGED** `c757056`. DP-7 boundary review was **granted by two independent reviewers** (`gire` + `nava`), both of whom reproduced the false positive rather than taking my word; `boundary-reviewed` label applied. NOT self-certified even though HCOM_MGR was stale and the operator directive would have permitted it. |
| **Boundary-lint narrowing** | Tracked by the hcom lane, deliberately **not** shipped. Note for whoever picks it up: the obvious `grep -Ew` fix is a **trap** — `_` is a word constituent, so `-w` would break the `HCOM_` prefix token and silently disable most of the gate. Prefix / identifier / bare-word tokens each need different treatment. |
| **PR #68 rebuild** (bounded logging) | Branch `tutu/bounded-logging-pr68-20260721` (pushed) = main + a completed merge of the bounded-logging work, tasks.json resolved to main's version. The **three fixes on top are NOT implemented** — see §5. |
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

Not implemented because it involves `unsafe` fd manipulation that should not
ship unverified.

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
