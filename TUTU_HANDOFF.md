# Limux — tutu Manager Handoff

**Owner:** `tutu` (`LIMUX_MGR`), manager claim `mgr-bdf89ce87f6b7506`
**Updated:** 2026-07-21 ~4:15 PM EDT
**Scope:** `/home/riche/MCPs/limux`
**Lineage:** lifo → limu → **tutu** (all 2026-07-21; limu closed out under the fleet-wide Codex close-out directive)

> ## ⚠️ PARTIALLY SUPERSEDED — read root `HANDOFF.md` FIRST
>
> Root `HANDOFF.md` was consolidated on 2026-07-21 into the session-agnostic
> directory-state doc and is now the **authoritative current state**. The
> earlier instruction here to distrust it is obsolete.
>
> Stale in THIS file, corrected in root `HANDOFF.md`:
> - **PR #84 is MERGED** (`f2b0a79`), not open. **PR #85 also merged**
>   (`3bf819f`, TaskMaster #33 build dirty-marker).
> - **Gate is 620 passed / 0 failed**, not 613.
> - **Installed is now `3bf819f6a949`** (`main-3bf819f6a949-all5fixes-20260721`),
>   containing all five fixes — not the `a5c0f987` three-fix build named below.
> - **The host is DOWN**; the operator restarts it by typing `limux` in a
>   Windows terminal. The restart's usual cost (killing every hosted pane
>   process) is already paid, so restarting now costs nothing.
> - The **standing adversarial review never ran** — commissioned 3×, died each
>   time. PRs #82–#85 are self-reviewed only.
>
> Everything below is retained as this session's detail and reasoning.

---

## 1. 🔴 THE ONE THING THAT NEEDS THE OPERATOR

**A Limux restart window must be scheduled.** Three merged fixes are installed
but **not live**, and a fourth is merged-pending.

**A host restart KILLS EVERY HOSTED PANE PROCESS.** Verified, not assumed:
- Pane shells are **direct children** of the host. At time of writing: **22
  direct pane shells**, with ~**124 `claude`**, ~25 `hermes`, ~17 `python`
  descendants inside them. All die.
- `layout_state.rs` comment (search `pretending process state`): *"rehydrate a
  fresh terminal at the last known directory instead of pretending process state
  can be restored."*
- **Restores:** workspaces, layout, pane tree, cwd. **Does NOT restore:** any
  running process. Agents return marked *"Agent suspended after an unclean
  Limux shutdown."*

So hamo's earlier "28 workspaces migrated hash-identically" was **layout**
fidelity, not session survival — consistent with dino landing in
`unclean_restore` after the previous force restart.

**Required sequence:** fleet checkpoint broadcast → all sessions push/commit +
write durable handoffs → operator closes Limux → runs `limux` → `limux doctor`
(expect green) → restart `omp` → cross-check symptoms with `karo`.

`karo` (OMP lane) is **holding the checkpoint broadcast pending operator go**.

---

## 2. CURRENT RUNTIME vs INSTALLED (they differ — this is expected)

| | Value |
|---|---|
| **Installed CLI** | `limux-cli 0.2.3 (a5c0f9876b29, release)` install-id `main-a5c0f9876b29-omp-scrollfix-20260721`, channel `stable` |
| **Running host** | still `main-1a26bda0-v0.2.3-20260719` (the pre-fix 7/19 build) |
| **`limux doctor`** | `[fail] socket: connected host build SHA differs from CLI build SHA` |

**That failure is CORRECT** — it is the launcher-drift guard from PR #79/#80
working as designed, reporting "new CLI, old host, restart needed." It clears on
restart. Do not "fix" it.

Install command used (already run; archives previous launchers, no destructive
overwrite):
```
cargo build --release -p limux-cli -p limux-host-linux
bash scripts/user-local-install/install-user-local.sh --apply --profile release \
  --channel stable --install-id main-a5c0f9876b29-omp-scrollfix-20260721
```

---

## 3. WHAT LANDED THIS SESSION

| PR | Content | State |
|---|---|---|
| **#81** | limu's stranded audit + `LIMU_INBOX` + TaskMaster → main | MERGED `70689b4` |
| **#82** | H1 cross-lane disclosure · `hook_session_id` misattribution · **OMP scrollbar root cause** · PR #58 attestation salvage | MERGED `08abec1` |
| **#83** | PR #67 rebuilt on main + socket-mode P2 | MERGED `a5c0f98` |
| **#84** | TaskMaster #29 reflow — sub-cell resize deferral | **OPEN**, 620/0 green |

**main gate: 613 passed / 0 failed**, clippy `-D warnings` + fmt clean.

### The four defects fixed

1. **H1 — cross-lane information disclosure.** `read-screen --help` fell through
   to `surface.read_text` with no target; the server's global-focus fallback
   returned *another agent's pane content including in-flight command text*.
   `read-screen` was also the only one of read-screen/send/send-key with **no
   `LIMUX_WORKSPACE_ID` fallback**, so it defaulted to global focus
   unconditionally. Corroborated by reve's 2026-07-19 incident.
2. **`hook_session_id` — cross-lane misattribution.** Preferred ambient
   `CLAUDE_SESSION_ID` over the payload's own `transcript_path`, and
   `limux_env_value` walks **ancestor process env**. Hook events for session A
   were attributed to whoever invoked the CLI. **Decision recorded:
   payload-first** — all explicit payload identity before any ambient value.
   This also repaired a **non-deterministic quality gate**: the test passed under
   Codex and failed under Claude, which is why limu's audit recorded 597 passing
   while `CLAUDE.md` warned the same test failed. *Both were correct.*
3. **OMP scrollbar peg/flash — root cause found.** The scrollbar is a **layout
   sibling** of the terminal (`root.append(&overlay); root.append(&scrollbar)` in
   a horizontal Box). In GTK4 an invisible box child gets **zero allocation**, so
   every `total > len` flip changed the GLArea width by ~13px → `connect_resize`
   → `ghostty_surface_set_size` → **column change** → reflow → and reflow moves a
   scrolled-back viewport to the active area. Ghostty's own comment on that path:
   *"this effectively **pulls down** scrollback"* — the operator's verbatim words.
   **limux-specific**: upstream ghostty uses `GtkScrolledWindow` overlay
   scrolling, which is layout-neutral by construction. Fix: layout participation
   now decided by **config only**; scroll state varies opacity/hit-testing only.
4. **PR #67 socket-mode P2.** The preview runner passed an inherited
   `LIMUX_SOCKET_MODE`/`CMUX_SOCKET_MODE` into the host, but its probe CLIs are
   children of the **runner**, so `is_descendant` rejected every probe and healthy
   backends were misreported unhealthy. Now cleared + forced `localUser`.

---

## 4. OPEN WORK, PRIORITIZED

| Item | State |
|---|---|
| **Restart window** | 🔴 operator-gated, §1 |
| **PR #84 (#29 reflow)** | open, gate-green, **needs live drag verification** at restart |
| **PR #68 rebuild** (bounded logging) | NOT started. Complete file:line plan exists — see §5 |
| **Task #33** | build dirty-marker false-positives on untracked files (see §6) |
| **reve new-pane incident** | needs a **v0.2.3 retest** — filed against legacy 0.2.2 (`LIMU_INBOX/`) |
| **nava design question** | hcom-TUI × Limux symbiosis; **tutu/successor is the named design owner**. Cheapest seam already ships (`pane-action set_flag_color`). See `LIMU_INBOX/DESIGN_QUESTION_FROM_nava_…` |
| **Root `HANDOFF.md`** | still stale Halo 2026-06-20; consolidation open (§7) |
| **ghostty wheel/mouse-reporting** | wheel events eaten with **no alt-screen gate** (`Surface.zig:3601-3621`). Vendored **read-only** — do NOT patch. karo advised: check shift+wheel escape hatch first; draft an upstream issue but **operator approves before filing externally** |

---

## 5. PR #68 — plan exists, NOT implemented

An Opus agent produced a complete, file:line-exact rebuild plan. Key facts it
**verified by direct execution** (not inference):

- **The stderr-fd P2 is real and deterministic.** With fd 2 closed, `pipe()`
  returns `(2,3)` — the read end **always** lands on fd 2. `dup2` then destroys
  the drain thread's reader; the thread exits, drops its `File`, and **closes fd
  2 entirely**. The next `open()` claims fd 2, so subsequent stderr writes
  **silently corrupt an unrelated file**. That is data-integrity, not lost logs.
- Fix: `reserve_standard_fds()` before the pipe + `relocate_above_stderr()` +
  `pipe2(O_CLOEXEC)`.
- **Two findings it raised unprompted:** (A1) if the log sink fails the drain
  loop `break`s and nobody drains the pipe — once the 64KiB buffer fills, the GTK
  main thread blocks forever = **GUI hang**; (A2) at the byte cap `write_bounded`
  returns `Ok(false)` silently and rotation is startup-only, so the log dies
  **permanently and silently**.
- Only conflict with main is `.taskmaster/tasks.json` — **drop that hunk**; main
  is newer. All four Rust files auto-merge.

Not implemented because it involves `unsafe` fd manipulation the agent could not
compile, and shipping it unverified would contradict the discipline that caught
the other defects.

---

## 6. TRAPS AND DISCIPLINE (learned the hard way this session)

- **Task state is PER-BRANCH.** A TaskMaster tag that looks "missing" is usually
  a branch-view difference, not data loss. This caused a false alarm.
- **Work strands on branches.** limu's audit, my HANDOFF consolidation, and the
  #29 tests were each stranded on unmerged branches. **Land things on main.**
- **Never hand-edit `.taskmaster/tasks.json`** — use `task-master-ai-reviewed`
  (`--title`/`--description` for manual, no LLM cost).
- **The build dirty-marker lies.** `build.rs` computes it from
  `git status --porcelain`, which counts **untracked** files. The installed build
  is stamped `-dirty` solely because of an untracked peer-owned docs HTML;
  `git diff --stat HEAD` is empty. Task #33 has the fix (`--porcelain -uno`).
- **`docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` is
  peer-owned untracked dirt** — do NOT stage, modify, or remove it.
- **`/tmp/limux-release-0.2.3-20260719`** — hamo's no-loss hold stands; do not
  remove without explicit operator release.
- **Vendored `ghostty/` is read-only.** Work through the C API.
- **Clippy `-D warnings` is a hard gate** and it *will* catch things
  (`nonminimal_bool` on a helper here). Fix, never suppress.
- **Shared checkout with concurrent agents.** Preflight every non-trivial edit;
  `HEAD` can move under you.
- **The Codex PR bot is not reviewing** (fleet-wide Codex close-out), so PRs
  merge with 0 reviews / 0 checks. Weigh that when merging.

---

## 7. RECOMMENDED NEXT ACTIONS

1. Get the **restart window** scheduled (§1) — nothing shipped is live without it.
2. After restart: `limux doctor` green, then **live-verify** the OMP scroll fix
   and #84's resize behavior. Capture `strace -e ioctl` `TIOCSWINSZ` counts on a
   slow split drag, before/after, to settle #84's one unverified claim.
3. Implement **PR #68** from the §5 plan (A1 GUI-hang deserves priority).
4. Consolidate root `HANDOFF.md` into one session-agnostic current-state doc —
   this file plus `LIMU_HANDOFF.md` are the raw material. The fleet-wide
   HANDOFF-reconciliation remedy (doni/dino) is adopting the same shape.
5. reve's new-pane **v0.2.3 retest**; then answer nava as design owner.

---

## 8. WHAT IS NOT CLAIMED

- **No live GUI verification of anything.** Every fix is gate-verified
  (tests/clippy/fmt) and source-traced, but the OMP scroll fix and #84 have not
  been observed working in a running terminal.
- #84's premise that a changed `ws_xpixel` raises `SIGWINCH` is reasoned from
  ghostty source, **not** confirmed against the kernel.
- The 2026-07-16 renderer evidence carried into #83 was produced by different
  binaries and is **not re-attested**.
- PRs #82/#83 merged with **no external review** (Codex bot down).
