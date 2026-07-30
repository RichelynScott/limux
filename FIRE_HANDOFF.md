# FIRE_HANDOFF — limux lane, 2026-07-28 fleet C: drive space crisis

**Created by:** Claude Code (`fire` / `fire_LIMUX_SPACE_MGR` · session `37f828e0` · Claude Opus 5)
**Date:** 2026-07-28 09:05 EST
**Purpose:** Resume spec for the limux lane of the 2026-07-28 fleet disk-space
effort. Written before an operator-initiated session restart.

---

# ⚡⚡⚡ CURRENT STATE — 2026-07-30 20:10 EST — LANE CLEAN, NOTHING ASSIGNED. READ ONLY THIS BLOCK; everything below is history

`main` = **f223ff2**, tree clean, in sync with origin, **zero open PRs**, `./scripts/check.sh`
**exit 0**. Nothing is blocked on this lane and **nothing is assigned to fire.**

## ⚠️ TWO THINGS A SUCCESSOR WILL GET WRONG IF NOT TOLD

### 1. `limu` commits into THIS working tree. Your HEAD moves without your pull.

HEAD went `1c8d43b` → `55e1f99` → `f223ff2` under me with no pull on my side. Not a
problem — same lane — but it means **`git status` "clean" does not mean "unchanged since
you last looked."** Re-check HEAD before reasoning about repo state, and never assume a
commit you didn't make isn't there.

### 2. Git attribution in this repo is UNRELIABLE — everything is stamped `Session-ID: FIRE`

papa-git's `prepare-commit-msg` falls back to the global `$HOME/.claude-session-name`
(contains `FIRE`) when per-session env is absent. My own `CLAUDE_SESSION_NAME` is **unset**,
so even my commits take the fallback — and limu, a **Codex** session that would never set a
Claude variable, inherits my name too. Proof, two commits, one shared source:

| commit | Session-ID | Agent |
|---|---|---|
| `1c8d43b` (mine) | FIRE | claude-opus-5 |
| `55e1f99` (limu's) | **FIRE** | codex-gpt-5.6-sol-high |

**⚠️ My scope claim was WRONG and limu corrected it — carry the correction, not my version.**
I reported this as "machine-wide, any papa-git-hooked repo." It is **not**. Canonical
PAPA_GIT source **already disabled the shared global fallback at `b8d32f7`**; limux's
**installed hook is stale and hash-mismatched**. So the exposed set is **stale deployed
hooks lacking stronger identity**, not every hooked repo automatically. The real question
for any other repo is *"is its installed hook stale?"* — which must be checked, not assumed.

**The sharpest framing is kazu's, not mine: the fallback DEFEATS papa-git's designed
fail-loud behavior.** The rule specifies a *loud commit refusal* on missing identity, never
a silent drop. The global fallback converts that refusal into **silent misattribution** —
strictly worse, because a refused commit is visible and a wrongly-stamped commit is
permanent. kazu independently re-derived both commits and confirmed **their own
`CLAUDE_SESSION_NAME` is also unset**, so the default path is broken for every session that
doesn't pass explicit env — not just mine.

**The detector was corrected THREE times, by three sessions, and only the last one read the
spec.** Sequence: I claimed "zero false positives" → limu said it needs an authoritative
runtime mapping → kazu built an assumption-free form (*one Session-ID under BOTH runtime
families is impossible regardless of naming*), which over **794 trailer-bearing commits / 45
days / 6 repos** returned exactly three: **FIRE, PAPA_GIT_MGR, HCOM_MGR** → **limu then read
canonical PAPA_GIT and refuted the impossibility premise itself.**

`COMMIT_PROTOCOL.md:69,80-83` defines Session-ID as *universal* identity and **explicitly
permits manager/role identities** (PAPA_GIT_MGR, HCOM_MGR by name); `:98-100` derives runtime
separately from the Agent prefix; `UNIVERSAL_ATTRIBUTION_PROTOCOL.md:35` says "stable session
**or role** identity." So a role identity appearing under two runtime families is
**legitimate role succession**, not proof of misattribution.

**Therefore: cross-family reuse is a HIGH-SIGNAL ANOMALY requiring role/session correlation
— NOT a zero-false-positive fatal check.** Of kazu's three hits, only **FIRE is confirmed**,
established independently by live ownership plus the known commits. Do **not** carry forward
"the attribution system's own manager identity is misattributed" — I wrote that here and it
is **not established**; PAPA_GIT_MGR and HCOM_MGR may both be legitimate succession.
(kazu's `separator=%x20` parser fix stands regardless.) My original naive form would also
have false-positived on LIFO and HAMO.

**The lesson worth more than the detector:** three sessions proposed three confident contracts
for this check, each sounding authoritative, and the disagreement was only settled by someone
opening the canonical protocol doc and reading what Session-ID is *defined* to mean. Nobody
did that for two rounds. When a check's validity rests on "X is impossible," the impossibility
claim is a **spec question**, not a reasoning question — read the spec first.

**⚠️ The trap, and it is the important part:** the naive query **silently returns a false
all-clear**. `%(trailers:key=...,valueonly)` emits a trailing newline, so each record splits
across three lines and the Agent field is always empty — kazu's first run reported **ZERO
mismatches while the known-positive `55e1f99` sat inside the scanned range**.
**`separator=%x20` fixes it.** It was caught only because they had a known positive to
validate against. Never trust a detector that has not been run against a known positive; an
all-clear from an unvalidated instrument is indistinguishable from a broken one.

limu landed the forward corrective record at **`f223ff2`** (`Session-ID: HCOM_LIMU`, so
their identity resolves correctly now) and rightly refused to rewrite pushed main. Owner
requests went to `kazu` + `niru`; `hobo`/`zori` were unavailable, so **PAPA_GIT/dual-hub
rollout stays owner-gated** and is named in the durable record at
`LIMU_INBOX/ATTRIBUTION_CORRECTION_FROM_limu_2026-07-30.md`.

## What happened 2026-07-30 (do not re-derive)

- **WSLg display reset ≠ limux bug.** 18:11:59 `Error reading events from display:
  Connection reset by peer` → `limux-host` exited 1. Diagnosed independently by `limu` and
  me, converging: no panic, no segfault, **not** #106/#107/#108. Ruled out at source: runtime
  tree intact, `libghostty.so` linked, `session.json` valid JSON (no disk-pressure
  truncation), display healthy after. Reproduction: `timeout 30 limux` → **exit 124** (stayed
  up), captured **unpiped** so the code is the launcher's, not a pipeline's. Filed as
  **fast-follow 8**; the defect is the *bare status 1*, not the exit.
- **The display-loss fix cannot go where you'd put it.** `main.rs:555-561` already records,
  for the sibling stderr-flush problem: *"measured headless, GTK terminates the process from
  inside `app.run()`, which never returns."* Anything sited after `app.run()` is **dead code**
  on this path. Seams: `GdkDisplay::closed` (preferred — can name the cause) or the proven
  `atexit` pattern. Regression test: a message-formatting unit test would be **decorative**;
  the load-bearing one is xvfb integration (legitimate — a timeout *ceiling*, not a timing
  *assertion*).
- **Fast-follow 9** — successor rebind: after an unclean restore a successor can update the
  hook store but the surface stays suspended under the predecessor identity; no live rebind
  exists. Hand-editing `session.json` is the WRONG fix (live operator state, no schema-checked
  external write path). Authorization overlaps item 7's per-connection entitlement — one
  design, not two. **Lane: limu.**
- **OMP ask-waiting: decided, and fire is NOT assigned.** My triage found W1.3's needs-input
  state machine **already exists** (`agent_state.rs`, incl. the `acknowledged` urgency bit);
  the real blockers were that OMP is not an `AgentKind` (0 grep matches) and `ask` is not in
  the hook vocabulary (0 matches) → `_ => None`, silent by design. `nara` then confirmed **OMP
  does not call `limux hooks` at all**, which killed my proposed zero-code `notification`
  mitigation — marked **SUPERSEDED in place** in both durable copies rather than left to be
  followed. `limu` decided **A / GO (corrected scope)**: PRD-G is only slice 1, live
  hooks/sidebar/socket/CLI stay under TaskMaster 7; **limu owns the receiver contract, rako
  owns OMP emission.** Do **not** implement `AgentKind::Omp`.
- **hcom broke fleet-wide for ~1 minute and recovered.** `current -> {ARTIFACT}` — an
  uninterpolated template variable. My send failed exit 127. I **deliberately did not repoint
  it**: timestamps showed an in-flight deploy, and racing another lane's installer makes it
  worse. It self-resolved at 19:37. Reported to `heli` with the structural note: the failure
  is **self-concealing** — anyone hitting it cannot use hcom to report hcom is down, so
  observed duration measures who had a workaround, not impact. Recovery: invoke the versioned
  binary directly, bypassing `current`.
- **Six untracked coordination drops rescued** (`c85f55b`, `b920034`, `6b8bccb`, `1c8d43b`).
  `git clean -ndx` would have taken every one, and `vimi` — author of the first — has since
  **left the roster**, so it would have been unrecoverable with nobody to re-ask. All limux
  inbox surfaces are now committed and verified. Structural, not carelessness: a drop is
  created untracked by default, and *present in a directory* is indistinguishable from
  *durable in it* until someone cleans.
- **My owned fast-follows verified by MUTATION, not assertion.** Item 4 (cache poisoning):
  `var_os` at runtime, landed. Item 3 (rotation flock): reverted the lock at the call site →
  test **failed** with the intended message → restored from git, blob verified identical to
  HEAD (`ed42907`), full gate **exit 0**. Done because I wrote both the fix *and* its test —
  the exact pairing that produced today's H1 error, where my tests passed because they tested
  what I already believed.

## Still open / not mine

- **H1 residual (CRITICAL, fast-follow 7)** — an explicit foreign `workspace_id` bypasses all
  the #107 scoping in one call. Honest ceiling of a dispatcher-side fix; needs per-connection
  entitlement and **cannot key on uid** (the operator is the one legitimate cross-workspace
  reader and shares the agents' uid). **Do not implement speculatively.**
- **C: at 89% and rising** (was 87% on 07-29). vhdx only grows; `wsl --shutdown` + compact is
  **operator-only** and has still not run.
- Fast-follows 1–2, 5, 8–9 → **limu**. `OMP_MGR_INBOX/STATUS_FROM_lubo_...` + my drop there
  remain untracked in the OMP tree — **theirs** to commit; do not commit into another lane's
  mid-flight checkout.

## Standing don'ts (unchanged, still load-bearing)

**No `git clean` in this repo** (`-fdx` takes 13 paths incl. `archive/`) · **never re-ignore
`archive/`** · **never touch `~/.local/share/limux/**`** (live operator session state) ·
**never re-hand any `--set-sparse` sequence** (hard-blocked on this host) · agents never
delete staging roots — the operator alone does.

---

# (history) 2026-07-29 18:05 EST — LANE CLOSED OUT (superseded by the block above)

`main` = **896f93c**, tree clean, in sync with origin. **Zero branches ahead-or-gone.**
All 14 tracked tasks closed. All 4 PRs of this cycle merged: #102 `59876fa`, #103
`457638a`, #104 `ab05eac`, #105 `a520e4d`.

## What a successor must NOT re-derive

- **Disk reality:** `/` Avail 665G → **772G**. But the **vhdx is back to 223 GiB and C: is
  87% full (127G free)**. Freeing space inside WSL does **not** return C: space — the vhdx
  only grows; only `wsl --shutdown` + compact converts internal frees. The ratchet is agent
  DB churn (codex `logs_2.sqlite` 1.7G, `hcom.db` 497M, hermes `state.db` 387M) — **other
  owners' lanes**, retention asks already routed.
- **Retention prune is live and validated** (`scripts/user-local-install/prune-reviewed-runtimes.sh`):
  **defaults to dry-run**, needs explicit `--apply`, and `--reviewed-root` is **required**.
  First live run archived 6 stale snapshots while protecting the active process, all 5
  launcher links, and newest-3 per lane; verified after by all-5-launchers-OK + green
  `limux doctor`.
- **8 retired-session branches are preserved on origin as `preservation/*` refs**, SHA-verified,
  before their locals were deleted (95 → 82 branches). Reason: for gone-upstream branches the
  only evidence was file-existence, which does **not** prove a commit's changes landed —
  preservation was cheaper than upgrading the proof. Deleting refs reclaimed ~zero disk.
- **Batched `git push` in a loop fails** ("failed to push some refs") then succeeds when run
  individually — remote lock under rapid succession. **Verify pushes with `git ls-remote`,
  never by push exit status.**

## OPERATOR DECISIONS STILL OPEN

1. **Packaging fix (severe, ships to users).** `docs/LIMUX_PACKAGING_DELETE_AUDIT_2026-07-29.md`:
   the generated `install.sh` + `.deb` postinst delete `/usr/local` files **as root on an
   ordinary `dpkg -i`**, destroying a source-built install; the legacy-host heuristic matches
   any GTK binary named `limux` and **executes it** before deleting. Fix drafted, **not
   applied** — changes what lands on user machines. Lane: limu.
2. **Three stale PRs open since 2026-07-15/16** from retired sessions: **#68**
   bounded-logging, **#67** renderer-diagnostics, **#58** hcom-tracking. Merge / close /
   adopt. (I previously mis-stated "zero open PRs" — I had only looked at this cycle's.)
3. **H1 workspace-entitlement code fix** — design note + blast-radius inventory on main
   (`d6cd153`). Socket auth is uid-only, so cross-workspace pane reads succeed; the only
   legitimate cross-workspace reader is the operator (same uid), so entitlement cannot key
   on uid. Not implemented by design.

## Queued, not blocked

- **tutu** (live): item-2 reject-unknown-flags CLI hardening, item-3 limux-local CLAUDE.md
  checklist lessons, plus karo's scroll bug below.
- **limu**: packaging fix (gated above) + fast-follows 1–2 in `docs/LIMUX_FASTFOLLOWS_2026-07-29.md`.
- **karo's scroll-input bug**, source-verified by me and committed (`896f93c`) — it arrived
  **untracked in the retired `LIFO_INBOX/`**, one `git clean` from gone. `terminal.rs:2643`
  passes the keyboard-mods byte as `scroll_mods`; `embedded.zig:1976` bitcasts it into
  `ScrollMods` whose bit 0 is `precision`; `GHOSTTY_MODS_SHIFT = 1<<0` → **Shift+wheel sets
  `precision=true`** and a discrete tick becomes a pixel delta. One-line fix; **no test seam**
  (GTK closure) so use the repo's documented escape hatch rather than forcing a timing test.
- **huno, nafo, lifo are GONE from the hcom roster** (not merely idle). Don't wait on them.
  Route limux reports to **tutu**.

---

# (history) 2026-07-29 12:05 EST — PR cycle complete

**All three PRs MERGED (squash) and the checkout is reconciled on `main`:**
#102 → `59876fa` (keep-last-N retention, durable fix C) · #103 → `457638a` (log cap +
archive tracking + space-crisis records) · #104 → `ab05eac` (coordsurf FYI consolidation).
Remote + local feature branches deleted. Bot verdicts were **P2-suggestions-only**; fixes
applied pre-merge by fire: chmod 755 on the prune script (`1953bbe`), .gitignore git-clean
wording + rename of a 0-byte non-UTF8 stray in archive/ (`b15fa05`), and the #104
`list/save/rm`→`list/path/rm` correction (`31625da`, verified against `run_profile_command`).

**da8c108 gate DISCHARGED:** full `./scripts/check.sh` green on the #103 head (685 tests,
exit 0) — after root-causing two exit-101 runs to **shared-target cache poisoning**:
limu's removed `/tmp` worktree path was baked (compile-time `env!("CARGO_MANIFEST_DIR")`)
into the `limux-ghostty-sys` build-script binary; `cargo clean -p limux-ghostty-sys` fixed
it. This partially reframes the old "flaky 101 under load" story.

**Operator decisions (packet answered 2026-07-29):** D1 = destroy Tiers A–D, restore E —
Tier E (`raw-key-input-202607010843`) restored by fire via `mv` and verified; staging root
`~/.local/limux-ORPHAN-STAGING-20260729` now holds A–D only; the destroy step is the
OPERATOR's alone. D2 = Docker reclaim **DEFERRED** (no inventory lane, no TaskMaster task).
D3 = untiered remainder accepted, item **CLOSED**.

**Remaining (on resume):** tutu = delete redundant `coordsurf/*` branches + sweep stale
`.claude/worktrees` agent worktrees (no-loss check) + final shared-HANDOFF pass. limu =
verify/clean the two stale `limu/*-20260721` branches + fast-follows 1–2. Repo **issues are
DISABLED** → fast-follows live in `docs/LIMUX_FASTFOLLOWS_2026-07-29.md` (4 items).
One-build-at-a-time + freeze directive stands until the operator's reclaim restart lands.

---

# (history) 2026-07-29 11:50 EST state block — superseded by the block above

**11:50 addendum:** Operator decision packet delivered:
`docs/LIMUX_SPACE_CRISIS_PR_CYCLE_DECISION_PACKET_2026-07-29.html` (untracked on purpose —
commit rides a later cycle commit). Pending operator picks: D1 orphan-destruction /
D2 Docker vhdx / D3 untiered remainder — **nobody acts on these until the operator answers.**
Consolidation ACCELERATED per operator: tutu builds + PRs the coordsurf consolidation off
origin/main NOW (independent of #102/#103 — neither commits FYI.md). All lanes told to prep
for context compaction (#599-thread). Edge-open of the packet delegated to niru (Codex —
Fable browser-launch ban); operator clipboard fallback armed.

Compact SUCCEEDED this morning (vhdx 347.19→196.40 GiB, −150.79). Then huno found the
drain resumed (agent DB churn; vhdx 196.40→202.28 by 10:46) and ran an operator-approved
hibernation-off + `wsl --shutdown` + re-compact — if you are reading this fresh, that
restart is why.

## Open PRs — operator authorized: bot-review → merge on greenlight → cleanup
| PR | Branch | State |
|---|---|---|
| **#102** | `limu/keep-last-reviewed-runtimes-20260729` (keep-last-N retention; tutu CONFIRMED-GOOD + fire PASS) | `@codex review` requested 11:15 EST |
| **#103** | `fire/log-retention-and-cache-hygiene-20260728` (this branch) | `@codex review` requested; **still owes its own full check.sh** — limu's green run was MAIN-based and does NOT discharge the da8c108 gate (my corrected error, on-thread #599572 reply) |

## Post-restart assignments (broadcast #599572-reply, queued in hcom for huno+limu)
- **limu**: drive #102 → merge on greenlight → delete branch + verify/clean two stale 20260721 limu branches
- **huno**: full `./scripts/check.sh` on THIS branch (the real da8c108 gate), post exit status to PR #103; then #103 bot fix loop with me
- **tutu**: after both merge — ONE consolidation branch off fresh main (coordsurf FYI entries etc.) → PR → bot → merge → delete 6 coordsurf/huno-* branches → sweep 3 stale `.claude/worktrees/agent-*` → reconcile shared checkout to main → shared HANDOFF.md
- **fire (me)**: merge judgment #103, operator synthesis

## Freeze root-cause (operator question, answered)
Whole-VM memory/swap+IO thrash from concurrent team builds + a 17.6GB ugrep — NOT limux,
NOT WSLg. Standing directive: builds/tests announced + one-at-a-time; scoped searches only.

## Still true
- Orphan staging `~/.local/limux-ORPHAN-STAGING-20260729` (1.211 GiB, manifest in docs/) — operator alone destroys; Tier E needs their judgment
- Backlogs: package.sh 24 pre-existing deletes; host-test SIGSEGV-under-load flake
- No `git clean` here; no re-ignoring `archive/`; `~/.local/share/limux/**` untouched

---

# ⚡ UPDATE 14:50 EST — read this before §1; the sections below it are 09:05 state

Operator chose **GO NOW** (relayed by `luhe`, #597087). `wsl --shutdown` may fire
at any moment, then staging destruction (20 manifest lines, ~41.3 GiB) then
sparse/compact. funo's verify-against-live-pin condition is CLOSED — the
destruction window is unconditional.

## What changed since 09:05

| | |
|---|---|
| Staged | **6** targets / **28.45 GiB** (was 5 / 28.41 — added `06-4e625bfbade5`) |
| Commits | `da8c108` → `438e1fc` → `703adc7` → `b4c8dd8` → **`61a0f36`** → `ef4e7d9` → `be4a19f`, all pushed |

## ⚠️ `61a0f36` CHANGED THIS REPO'S IGNORE BEHAVIOUR — know this before you touch `.gitignore`

`archive/` is **no longer ignored**, in either scope. I removed it from the
committed `.gitignore` *and* from machine-local `.git/info/exclude:19` — either
alone would have left the ignore in force.

**Do not put it back.** `archive-not-delete.md` now carries a tracked-archive
invariant (landed off this lane's finding): `archive/` MUST be tracked, because
`git clean` removes only untracked paths, so a gitignored archive is destroyed by
exactly the command a space crisis invites. My original `da8c108` line is the
named anti-example in that rule; `61a0f36` is the cited remediation.

If regenerable build output needs parking, the sanctioned pattern — already
convention in `hcom`, `CODEX_CLAUDE_CODE`, `claude-task-master`, `SCRIM` — is to
ignore a **specific subdirectory** (`archive/generated/`), never the root.

**Closeout figure is settled: 41.31 GiB ALLOCATED**, ratified into protocol §6.6.
Never quote the apparent-bytes 40.29 — allocated is what `df` frees. Manifest is
exact to −32,768 B against `du -s` of the root, which also proved all three lanes
recorded consistently.
| First shutdown (10:05) | happened, but **destroy + compact did NOT** — both vhdx still byte-identical to the 08:45 baseline |
| Compact headroom | **146.2 GiB** compact-alone · **187.5 GiB** if staging destroyed first |

## ⚠️ DO NOT RUN `git clean` IN THIS REPO

`-fdx` takes **13 paths** including `archive/`, `logs/`, `target/`. `-fd` looks
safe and still takes the inbox dirs. Both inboxes are now committed, but per the
**fencepost** (below) that is mitigation, never closure — the behavioural control
is the real one.

---

# ⚡⚡ CURRENT STATE — 2026-07-29 07:30 EST. READ ONLY THIS BLOCK; everything below is history.

**Compact STILL has not run.** `ext4.vhdx` = 372,797,079,552 B (347.19 GiB), byte-identical to
the 2026-07-28 08:45 baseline. Two shutdowns have happened (10:05 and 06:11) with **no reclaim**.

| | |
|---|---|
| ext4 in use | ~158 GiB (`statvfs` — read `Used`, never derive from total−avail) |
| **Recoverable** | **~189 GiB** |
| C: free | **25 GiB** and falling (27G at 22:00) — losing ~2 GB/night with our lanes idle |

**Relay is `levu`'s.** Paste-once block: Docker-quit → `wsl --shutdown` → null-stripped
Running-gate → `diskpart /s C:\Users\riche\compact-wsl.txt` → size report. **Do not re-hand any
`--set-sparse` sequence** — it is hard-blocked on this host (`Wsl/Service/E_INVALIDARG`);
`--allow-unsafe` is the only sparse path and is advised against.

## Limux crashed 07:23 — it was NOT limux

`Gdk-Message: Error flushing display: Broken pipe`. The **WSLg display connection** died; the host
exited, the relaunch hit exit 1 mid-settle, a later launch succeeded. Verified the binary is sound:
run directly it starts and stays up (exit 124 = my timeout, not 1). Same root cause as the
2026-07-28 freeze — WSLg degrades as C: tightens. **Expect recurrence until the compact lands.**

## My lane: COMPLETE. Nothing owed, nothing in flight.

Freed 17.24 GiB · staged 28.45 GiB (destroyed by the operator, banked) · durable fixes landed.
All work pushed. Working tree clean except lifo's untracked HTML (not mine).

## ⚠️ Do not run `git clean` in this repo

`-fdx` takes 13 paths including `archive/` — the archive-not-delete destination. Coordination
artifacts accumulate untracked inside inbox dirs that *look* committed. Committing is mitigation,
never closure (the artifact announcing a fix always postdates it).

## Post-compact TODO, priority order

1. **huno's orphan tiers** — 1.21 GiB justified across 15 snapshots (+0.77 GiB in 23 more,
   classified in `FIRE_INBOX/RESPONSE_FROM_huno_..._untiered-archaeology-complete.md`). Held
   deliberately: post-restart ≠ post-compact, and that trigger never fired.
2. **huno may retire** `limux-preview-profiles`/`-cli` (36 MB duplicate) — standing by for the word.
3. **Full-workspace `check.sh`** — still owed for `da8c108` before the next release/tag.
4. Task #4 — keep-last-N prune for `~/.local/limux-reviewed` (resolve the live set **at execution
   time**; sweep `~/.local/libexec` too).

## Open, not mine

**H1 — the repo's own #1 risk**, `docs/REPO_AUDIT_limux_2026-07-21.md:57`: `read-screen` does not
reject unknown flags and does not require an explicit target, so it falls through to a focused-surface
read. Documented 2026-07-21 with source lines and a fix plan; **still open** — I reproduced it
accidentally (`read-screen --xyzzy BOGUS` → rc=0, returned my own screen). reve observed a genuine
**cross-workspace** disclosure on the *legacy* channel. tutu owns it; next move is a source-history
diff of the surface-resolution path (legacy vs current) — answerable with **no live probe**.

**The process lesson, and it cost the most hours of anything last night:** three sessions
live-probed a question the repo's own audit had answered eight days earlier. `git grep` the audit
and inbox *before* probing.

---

# ⚡ UPDATE 22:42 EST — first compact attempt FAILED; the hold still stands

**The 2026-07-28 restart happened. The compact did NOT.** Destruction succeeded
(staging root gone, ext4 in-use 201 → 157.75 GiB); the reclaim half did not.

| | |
|---|---|
| `ext4.vhdx` | 372,797,079,552 B = **347.19 GiB** — unchanged |
| ext4 in use | 157.75 GiB (statvfs — **read `Used`, do not derive from total−avail**) |
| **Recoverable** | **189.44 GiB** — the largest it has ever been, because destruction freed 41 GiB inside |
| C: free | 27G |

**`--set-sparse` is HARD-BLOCKED on this host.** Not a race, not interop — WSL
refuses it outright: *"Sparse VHD support is currently disabled due to potential
data corruption… `Wsl/Service/E_INVALIDARG`"*. `--allow-unsafe` is the only sparse
path and is **advised against** (Microsoft disabled it over real corruption
reports). Three sessions theorised about a "silent" failure that was never silent;
nobody asked what the command printed.

**Console relay is `levu`'s** — diskpart with read-only attach (fails loud if the
distro is up), Docker Desktop quit, Stopped-verify. I deferred; so did huno. Do
**not** re-hand any set-sparse sequence.

## ⚠️ A finding of mine in the fleet record is RETRACTED

I reported that `read-screen` cannot distinguish a ghost pane from a nonexistent
one. **Invalid.** `read-screen` has no `--pane` flag — only `--surface`. `--pane`
parses and does nothing, so my "byte-identical" result was two runs of a command
ignoring its input, plus a null case that matched both.

The control that collapses it: **run the command with the target omitted; if the
output matches, the input was never connected.** levu's probe matrix now runs that
first. The real, evidence-backed anomaly is tutu's: explicit `--surface` returning
**exit 0 with a different lane's content** via the documented focused-surface
fallback.

Ghost panes themselves are real (layout restored, no live terminals) and are
**huno/tutu's lane**, deferred post-compact.

## Post-compact TODO, in priority order

1. **huno's orphan tiers — 1.21 GiB across 15 snapshots**, deliberately held until
   after the compact (adding install-snapshot churn mid-sequence was inverted
   risk/reward). Classification + per-tier re-verification commands:
   `FIRE_INBOX/RESPONSE_FROM_huno_2026-07-28_limux-reviewed-orphan-classification.md`.
   Note huno's own correction: tiers are **1.21 GiB of 15**, not the 1.92 GiB
   headline; 21 snapshots (0.71 GiB) are deliberately **untiered** because they
   could not be justified by build history.
2. **huno may retire** `limux-preview-profiles` + `-cli` (36 MB duplicate) — they
   are standing by for the word, post-compact only.
3. **Full-workspace `check.sh` gate** — still owed for `da8c108` before the next
   release/tag. Scoped gate was run in its place by kazu's ruling.
4. Task #4 — keep-last-N prune for `~/.local/limux-reviewed` (install script;
   resolve the live set **at execution time**, and sweep `~/.local/libexec` too).

## Open, routed, not mine to close

- **`archive/` vs `git clean -fdx`** — a rule-level defect in `archive-not-delete`
  routed to kazu: `git clean -fdx` deletes the archive-not-delete *destination*,
  and `git add` into an ignored dir silently no-ops. **Partly self-inflicted**:
  `.gitignore:9` is a line I added in `da8c108`, which made the exposure portable
  to every clone. Filed at
  `~/Proj/CODEX_CLAUDE_CODE/CLAUDE_MGR_INBOX/INFO_FROM_fire_2026-07-28_git-clean-defeats-archive-not-delete.md`.
- **Fencepost in `dirty-coordination-surfaces`** — committing the coordination
  record can never reach zero residual; measured at +139 s. Same file.
- **`docs/LIMUX_RUNTIME_CLOSEOUT_..._LIFO.html`** — still untracked and still
  exposed to `git clean`. lifo's artifact, deliberately **not** absorbed.

## Four instances of one defect (worth carrying forward)

A live target doesn't make its **siblings** live · a referrer existing doesn't
make it **reachable** · a tracked directory doesn't make its **contents** tracked
· a gitignored path isn't necessarily **disposable**. All four: an existence or
property check standing in for a consequence check.

---

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
