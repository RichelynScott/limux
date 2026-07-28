# FIRE_HANDOFF — limux lane, 2026-07-28 fleet C: drive space crisis

**Created by:** Claude Code (`fire` / `fire_LIMUX_SPACE_MGR` · session `37f828e0` · Claude Opus 5)
**Date:** 2026-07-28 09:05 EST
**Purpose:** Resume spec for the limux lane of the 2026-07-28 fleet disk-space
effort. Written before an operator-initiated session restart.

> Per-session file per this repo's convention (§7 of the shared `HANDOFF.md`,
> owned by the LIMUX_MGR — `tutu` as of 2026-07-21, operator now refers to
> `huno`). **I did not write the shared `HANDOFF.md`** — peer-owned, route-only.
> A pointer request for its §7 index is owed to the current LIMUX_MGR.

---

## 1. IMMEDIATE NEXT ACTION

**Nothing is blocked on this lane.** Two items need a decision, neither urgent:

1. **PR is pushed but NOT opened.** Branch
   `fire/log-retention-and-cache-hygiene-20260728` @ `da8c108`, pushed to
   origin. This repo's `CLAUDE.md` says *"Don't open PRs or issues from inside
   Claude Code without asking"* — so it is deliberately unopened, awaiting the
   operator. Do not open it unprompted.
2. **Remaining durable item, unstarted:** keep-last-N prune for
   `~/.local/limux-reviewed` (37 snapshots, 2.2 GiB, grows one per reviewed
   build). See §5 for why it was deliberately not rushed.

---

## 2. ⚠️ TWO THINGS THAT WILL BITE A SUCCESSOR

### 2.1 `target/` is EMPTY. Do not build before the operator's compact window.

`cargo clean` banked **17.24 GiB**. A cold GTK4+libghostty rebuild re-consumes
roughly all of it, and the whole point of the reclaim is that the space is free
*at the moment* the operator runs the vhdx compact. If you need to verify Rust
changes before then, scope it — `cargo test -p limux-cli` cost only **707 MB**
because `limux-cli` has no ghostty/GTK dependency (verified in its
`Cargo.toml`). The full-workspace `check.sh` gate is **deferred to
post-compact** by explicit ruling, and is **owed before the next release/tag**.

### 2.2 28.41 GiB is STAGED, not deleted. Do not touch it.

`~/.space-crisis-pending-delete/fire/` holds 5 targets awaiting the **operator's**
single destruction command (`rm -rf ~/.space-crisis-pending-delete`) immediately
before `wsl --shutdown` + compact. Under the protocol, **agents never delete** —
staging is reversible `mv`, and the operator's own hand on that one command is
the authorization for every lane at once.

- Protocol: `~/Proj/C_DRIVE_SPACE_PROJECT/nafo_INBOX/PROTOCOL_FROM_kazu_2026-07-28_bounded-deletion-staging.md`
- Manifest: `~/.space-crisis-pending-delete/MANIFEST.jsonl` (my 5 lines; 15 total across lanes)
- If it must be un-staged, tell `nafo` first — the fleet total is sized off that manifest.

---

## 3. LANE RESULT

| | Bytes | State |
|---|---|---|
| Freed — `cargo clean` | 18,515,296,256 = **17.24 GiB** | already in `df` |
| Staged — `mv`, 5 targets | 30,510,125,056 = **28.41 GiB** | frees 0 until operator §5 |
| **Contribution to compact window** | | **45.65 GiB** |

`df` movement was bracketed *tightly* around each single command — lanes were
freeing concurrently, and a wide-window `df` would have claimed other lanes'
work (see §4).

Staged contents (`~/.space-crisis-pending-delete/fire/`):

| | Origin | Bytes | Class |
|---|---|---|---|
| 01 | `~/.local/state/limux/logs/archive/limux-host.log.legacy-unbounded-superseded-20260721` | 27,734,933,504 | superseded-archived |
| 02 | `~/MCPs/limux/ghostty/.zig-cache` | 1,396,670,464 | regenerable |
| 03 | `~/.cache/limux-tools` | 876,552,192 | regenerable |
| 04 | `~/MCPs/limux/archive/generated/target-task4-concurrency-20260716` | 393,728,000 | regenerable |
| 05 | `~/MCPs/limux/archive/worktrees/limux-runtime-markers-task4-20260716` | 108,240,896 | regenerable |

---

## 4. FINDINGS THAT CHANGED FLEET-WIDE DECISIONS

1. **`du -sb` under-reports; use `du -s`.** `du -s` (allocated blocks) predicted
   the real `df` movement to within **one 4 KiB block**; `du -sb` (apparent) was
   0.55% low here and **13.4% low** on `~/.cache/limux-tools` (many small files).
   `nafo` had asked all lanes for `du -sb`, then propagated this correction.
2. **Double-count hazard.** 47 GB was freed by *other* lanes between my baseline
   and my own command. Any lane reporting a wide-window `df` silently claims
   everyone else's reclaim. Tight bracketing is now required fleet-wide.
3. **Durable list re-prioritised (ratified by kazu).** The inherited ranking put
   an `archive/` sweep first and the jsonl cap last. Reading the code inverted it:
   `logs/archive/` is written by **no code at all** (it exists because commit
   `fc40cf5` moved a file there by hand), `logs/retained/` is bounded *by
   construction* because `rotate_managed_active` enforces its budget by refusing
   to rotate (`StderrFallback`) rather than growing, and `agent-hook-debug.jsonl`
   was the **only genuinely uncapped writer in the codebase**.
4. **`target/target` was a 29-byte self-referential symlink**, not the reported
   1.3 GB nested directory — a `du` that followed it one hop had double-counted
   `release/`. Three sessions converged on this independently. Upgraded to a
   *blocking* finding because a cleanup script written with a trailing slash or a
   cd-into-it form would have followed the link and destroyed the **active build
   tree**. It went away with `cargo clean`; `scripts/check.sh` now guards it
   permanently.

---

## 5. WHAT I DELIBERATELY DID **NOT** DO

- **Did not use the `: > file` truncation shortcut** that circulated as a
  "hook-free fallback" for content the `rm -rf` deny blocks. It destroys the same
  bytes while producing no permission prompt and no audit trail — evasion by
  mechanism. `kazu` has since **withdrawn and prohibited** it fleet-wide.
- **Did not use `rm -r` to dodge the `rm -rf` deny pattern.**
- **Did not use the `PAPA_GIT_ACCEPT_UNKNOWN=1` bypass** when the first commit was
  refused — that produces a commit attributed to `unknown`. Fixed the real cause
  instead (see §7).
- **Did not prune the 5 `copy-paste-*-20260622` snapshots** (755 MiB). These are
  built install snapshots, not regenerable from a cache, and dropping them costs
  the ability to bisect a regression. Not worth it on a 48 GiB lane.
- **Did not rush the `limux-reviewed` keep-last-N prune.** It edits the install
  script, and getting the live-set resolution wrong deletes a live binary — the
  exact failure class confirmed **three times** on this box today (hcom, SCRIM,
  and a near-miss on limux). It needs the live set resolved *at execution time*:
  `find ~/.local/bin -lname '*limux-reviewed*' -exec readlink -f {} \;` — and
  sweep `~/.local/libexec` too, which is where hcom's real binary turned out to
  live and where a `~/.local/bin`-only grep would have missed it.

---

## 6. DURABLE FIX — landed

Commit **`da8c108`** on `fire/log-retention-and-cache-hygiene-20260728` (pushed):

- **`rust/limux-cli/src/main.rs`** — 8 MiB cap + single retained generation on
  `agent-hook-debug.jsonl`. It had **no bound of any kind**
  (`append_debug_line` did create+append+`write_all` with no size check, reached
  from 8 call sites) and was at 20.3 MiB / 73,926 lines, still appending. An
  oversized single write still lands rather than being dropped — losing telemetry
  is worse than one oversized file, and the next write rotates it.
- **`scripts/check.sh`** — tripwire failing the gate if `target/target` is a
  symlink.
- **`.gitignore`** — `archive/` promoted from `.git/info/exclude` (machine-local,
  inherited by no clone) into the committed ignore.

**Both mutation-tested** per this repo's review checklist: removing the rotation
call fails 2 of the 3 new tests; removing the tripwire lets the hazard ship
undetected (exit 0). The rotation test asserts `total-kept < total-written`
*specifically because* an `active <= cap` assertion alone would still pass
against an unbounded implementation — that is how this class of test ends up
decorative.

**Correction to an inherited suggestion:** adding `ghostty/.zig-cache` to the
top-level `.gitignore` is **not actionable** — `ghostty` is a git *submodule*
(`git check-ignore` → `fatal: Pathspec ... is in submodule 'ghostty'`), and a
parent `.gitignore` cannot govern paths inside one.

Scoped gate green: `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets
-- -D warnings`, `cargo test -p limux-cli` → **140 pass, 0 fail**.

---

## 7. ENVIRONMENT GOTCHA (will hit other sessions on this box)

The first commit was **refused**: `papa-git: CLAUDE_SESSION_NAME unset — commit
refused.` The sidecar `~/.claude-session-name` **did not exist** on this box.
Fixed by writing it (`FIRE`) — the var must match `^[A-Z0-9_-]{1,50}$`, so the
lowercase hcom name uppercases. `CLAUDE_AGENT` was already set
(`claude-opus-51m`). If a successor hits the same refusal, fix the sidecar; do
not reach for the `unknown` bypass.

---

## 8. KEY CONTEXT FILES

| Path | What |
|---|---|
| `FIRE_INBOX/TASK_FROM_voru_2026-07-28_limux-lane-reclaim.md` | The originating task package (untracked) |
| `~/Proj/C_DRIVE_SPACE_PROJECT/nafo_INBOX/PROTOCOL_FROM_kazu_2026-07-28_bounded-deletion-staging.md` | The staging protocol + authorization record (A1/A2 are mine) |
| `~/.space-crisis-pending-delete/MANIFEST.jsonl` | Audit record surviving destruction |
| `~/Proj/C_DRIVE_SPACE_PROJECT/AUTHORIZATION.md` | Project gate — §3 still unratified for this effort |
| `rust/limux-host-linux/src/host_log.rs` | Rotation subsystem (`rotate_managed_active` ~L419) |
| `rust/limux-cli/src/main.rs` ~L1857 | `append_debug_line` + the new cap |

**Peers:** `nafo` (NAFO_SPACE_MGR, execution coordinator — aggregating fleet
totals), `voru` (storage-forensics, overseer), `kazu` (global-config, rule
owner), `remi` (hcom lane), `funo` (taskmaster lane).

**Two-layer authority:** kazu's protocol clears the **rule** layer only. The
**project** gate (`AUTHORIZATION.md` §3) is `nafo`'s operator-ratification item.
Both must clear before the staged content is destroyed.
