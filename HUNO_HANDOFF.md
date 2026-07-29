# HUNO_HANDOFF — session profiles lane + 2026-07-28 disk rescue

**Created by:** Claude Code (`huno` · session `f1e96226` · Claude Opus 5)
**Date:** 2026-07-29 02:55 UTC (2026-07-28 10:55 PM EST)
**Purpose:** Resume spec for whoever follows `huno`. Written deliberately BEFORE an
announced session-killing event (operator-gated `wsl --shutdown` + VHDX compact),
per `checkpoint-before-runtime-restart.md`.

> **Ownership:** this is `huno`'s own per-session file, per `HANDOFF.md` §7
> convention. The shared `HANDOFF.md` is **tutu's** (current LIMUX_MGR) — do not
> edit it from this lane; route corrections to tutu.

---

## 1. IMMEDIATE NEXT ACTION

**Nothing is blocked on this lane.** One operator action is outstanding and it is
**levu's** relay, not mine:

1. Operator pastes the composed compact payload into **Administrator PowerShell**.
   It is already on the Windows clipboard. **Re-read it with `Get-Clipboard` at the
   moment of hand-off — never from any record, including this file.**
2. That paste shuts down WSL, killing every agent session including this one.
3. On resume: verify the reclaim landed, then release the held snapshots (§5).

If you are the successor reading this after the restart, start at §5.

---

## 2. WHAT SHIPPED — named session profiles

The operator's original ask: multiple independent saved workspace sets, so a second
limux instance is not either a clone or empty.

| Item | Value |
|---|---|
| Feature | `--profile <name>` + `limux profile list/save/rm` |
| Build | `limux-cli 0.2.3 (93132aea47b1, release)` |
| Installed as | `limux-preview-sessions` (channel `preview:sessions`) |
| Socket layout | `$XDG_RUNTIME_DIR/limux/<lane>/profiles/<name>/limux.sock` |
| Data layout | `<XDG_DATA_HOME>/limux/<lane>/profiles/<name>/session/session.json` |

**The load-bearing design fact — do not collapse these two dimensions:**

- **channel** = build lane, supplied by the installed launcher (`--channel stable`)
- **profile** = session set, supplied by the operator (`--profile work`)

They are read independently. `AGENTS.md` records why.

### The P1 that forced a revert (read this before touching the parser)

PR #92 modelled profile and channel as *one* field. Every installed launcher pins
`--channel <lane>`, so an operator's `--profile work` arrived as
`--channel stable --profile work` and the code rejected it as contradictory user
input. **The feature was unreachable for 100% of installed users.** Reverted as
#96, re-landed correctly.

A mutation proof could not have caught it — mutation testing perturbs code the
tests already execute, and **no test entered the launcher route at all**. That gap
is now closed by `rust/limux-cli/tests/launcher_route.rs` (5 tests) which writes a
real launcher of installer shape and invokes the real binary through it.

> **Test the route, not the parser.** This is the durable lesson; it belongs with
> the repo's existing revert-the-call-site rule.

---

## 3. OPERATOR'S LIVE DATA — verified, and the access path is NOT obvious

Both profiles verified on disk 2026-07-29 02:52 UTC:

| Profile | Workspaces | session.json | mtime |
|---|---|---|---|
| `main` | 27 | 53,937 B | 2026-07-25 19:45:52 |
| `second` | 6 | 7,009 B | 2026-07-25 19:45:52 |

### ⚠ Three launchers, one of them works

```
limux-preview-sessions profile list   -> main, second        <-- THE CORRECT ONE
limux-preview-profiles profile list   -> "no saved profiles yet"
limux / limux-cli      profile list   -> "unknown command: profile"
```

- `limux-preview-profiles` and `limux-preview-sessions` are the **same binary**
  (`93132aea47b1`) installed under **two different channel names**. Profiles are
  lane-scoped, so the one named "profiles" is empty. **I created this trap** by
  installing the same build twice under different lane names.
- `limux` on PATH is the **stable** lane at `c757056d2539`, which predates the
  feature entirely.

### Orphaned duplicate under `stable/`

`~/.local/share/limux/stable/profiles/{main,second}` holds **byte-identical**
copies (same sizes, same mtime). Harmless, but the stable binary cannot read them
(`unknown command: profile`), so it is unreachable data. Almost certainly written
by the profiles-capable binary invoked without a channel-pinning launcher — a bare
invocation resolves `channel=stable` (confirmed: `./target/debug/limux-cli
--version` reports `channel=stable`).

**Do not delete it.** `~/.local/share/limux/**` is operator do-not-delete, it is a
redundant copy of live data, and it is inside the VHDX so removing it frees nothing
on C: anyway.

**Cleanup owed (not urgent):** collapse the duplicate preview lanes to one, decide
whether `stable` should carry profiles at all.

### ⚠ The code I shipped inherited the systemic unknown-flag defect

The 2026-07-21 audit flagged this as **"Medium beyond H1. Systemic"** — *"makes
each new command responsible for remembering its own unknown-flag and help rules"*
(`docs/REPO_AUDIT_limux_2026-07-21.md:76`). A new command shipped seven days later
and forgot. Verified 2026-07-29:

| Probe | Result |
|---|---|
| `profile list --xyzzy BOGUS` | **rc=0**, flag silently ignored ❌ |
| `profile list --profile work` | **rc=0**, flag silently ignored ❌ |
| `--xyzzy BOGUS profile list` | rc=1 `unknown command: --xyzzy` ✅ (global position is fine) |

`profile list --profile work` is the dangerous one — same shape as `read-screen
--pane`: a flag that is **real elsewhere in the CLI**, accepted here, doing
nothing. That is precisely what made three sessions trust `--pane`.

**Severity is lower than H1** — be accurate about this. `profile list` only
enumerates lane-scoped profile names; there is no fallback into another lane's
data, so the consequence is confusion, not disclosure. But the **root cause is
identical**, and it should be fixed by the systemic Q1/T0.1 change (reject unknown
flags), not by special-casing `profile`.

**Validation that IS correct** (do not "fix" these):

| Probe | Result |
|---|---|
| `--profile ""` | rc=1, rejected with a clear message ✅ |
| `--profile "../escape"` | rc=1, path traversal blocked by the sanitizer ✅ |
| `--profile does-not-exist target-info` | rc=0 — **intended**; `target-info` reports the path that *would* be used, and create-on-demand is how a profile is made |

---

## 4. DISK RESCUE — destruction banked, reclaim pending

| Measure | Value |
|---|---|
| Freed by destruction (fire's lane) | **41.31 GiB** — allocated, not apparent |
| VHDX allocated on C: | 347.19 GiB (372.80 GB decimal) |
| ext4 actually used inside | 157.83 GiB |
| **Allocated-but-empty (the reclaim)** | **189.36 GiB** |
| C: free now / projected | 26.43 GiB → ~215.62 GiB |

Verified with three independent instruments (`ls`, Python `st_blocks`, `statvfs`).
A fourth `stat` call was malformed on my end and **discarded rather than
reconciled**.

**`wsl --manage --set-sparse true` is HARD-BLOCKED on this host**
(`Wsl/Service/E_INVALIDARG`; Microsoft disabled it over corruption reports). The
only sparse path is `--allow-unsafe`, which **must not** be used — that disk holds
everything. Compact is the sanctioned route.

Live payload on the clipboard (verified independently by huno and fire, and its
own contents differ from an earlier written record of them):

```
Stop-Process "Docker Desktop" -Force
wsl --shutdown ; Start-Sleep 6
$s = (wsl -l -v | Out-String) -replace "`0",""
if ($s -notmatch "Running") { diskpart /s C:\Users\riche\compact-wsl.txt ; <report size> }
else { "WSL STILL RUNNING - wait 10s and paste this again" }
```

The **gate is the control, not the sleep** — a short sleep with a Running-gate that
refuses and re-prompts beats a long sleep with no gate. `compact-wsl.txt` attaches
the vdisk **readonly**, which fails loud if the distro is up.

---

## 5. HELD — release only after a successful compact

**1.98 GiB of limux install snapshots**, classified by build history, staged and
**held** pending a successful compact (fire's stated condition, which has NOT
fired). Do not release them before verifying the reclaim landed.

No agent deletes anything: staging only, operator destroys. `archive/` is TRACKED
(un-ignored by `61a0f36`) and therefore immune to `git clean -fdx`; only the
regenerable subdirs `archive/generated/` and `archive/worktrees/` are ignored,
which is the sanctioned pattern.

---

## 6. OTHER LANES — not this one's work

| Item | Owner | State |
|---|---|---|
| Console relay / compact hand-off | **levu** | one operator paste pending |
| `read-screen` explicit `--surface` across a workspace boundary | **tutu** | **UNTESTED** — zero evidence either direction |
| `--pane` accepted-and-inert on `read-screen` | **tutu** | confirmed 3× (2 runtime, 1 at source) |
| Per-command targeting-flag matrix | **tutu** | queued |
| Repo CLAUDE.md review-checklist additions | **tutu** | queued, post-compact |
| Stale `archive/ is gitignored` claims | **tutu** | `HANDOFF.md:13`, `:329`, `docs/REPO_AUDIT_limux_2026-07-21.md:45` — **NOT bounded to 1**; see §7.10 |

**Do not run the cross-workspace disclosure probe ad-hoc.** Running it *is* the
disclosure. tutu's plan is correct: static-trace the explicit-surface resolution
path first (read-only, may answer it outright); only if inconclusive, a scoped live
test against lanes tutu stands up personally.

### ⚠ PRIOR ART nobody in the 2026-07-28 thread cited — read this first

Three sessions spent hours rediscovering a **documented, triaged, High-severity**
finding from eight days earlier. It is the repo's own **top-listed risk**:

- `docs/REPO_AUDIT_limux_2026-07-21.md:57` — **H1**: "`read-screen --help` can read
  another focused pane… builds a read request without recognizing `--help`, without
  rejecting unknown flags, and without requiring an explicit workspace or surface"
  → `rust/limux-cli/src/main.rs:4984-5014`. **Severity: High. Size: S.**
- Prescribed fix already written: **Q1 / T0.1** — "reject unknown flags *before
  socket contact*"; done-signal "`read-screen --help` cannot contact a host."
  This is the *same fix tutu independently derived on 2026-07-28*. Two derivations,
  eight days apart, never actioned.
- **H1's flag-rejection half IS STILL OPEN**, demonstrated 2026-07-29 on the
  installed **stable** build (fire): `limux-cli read-screen --xyzzy BOGUS` →
  **rc=0, returned terminal contents**. The unknown flag was not rejected, the
  socket *was* contacted, and a read *was* performed. Done-signal unmet.
  - My own earlier probe on `preview:sessions` returned `failed to connect to
    socket` (rc=1). That still shows socket contact precedes flag rejection — but
    it is **failure-path evidence, masked by that lane's host not running**. Cite
    fire's success-path result; it is the stronger demonstration. (A probe whose
    outcome is produced by an unrelated failure proves less than it appears to.)
  - **Precision — do not over-read it:** fire got *their own* surface, because
    they were inside a limux pane, so focused == own. That is consistent with H1's
    mechanism and does **not** by itself show cross-lane disclosure on stable.
    reve's cross-workspace read was on **legacy**. The disclosure half stays
    exactly where the build-version diff below puts it.

**The cited black-box incident materially changes item 1's status.**
`LIMU_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md`
§4 records an actual **cross-workspace** disclosure:

> "It falls through and performs a read of the **currently focused surface
> globally** — which, for me, was a completely different agent's pane in a
> different workspace (`~/Proj/oh-my-pi`), returning that agent's screen content
> including its in-flight command text."

So item 1 is **not** "zero evidence either direction." There is a documented
observation of a global, cross-workspace read. **But it does not simply confirm the
hazard either** — reve states plainly that *only* `main-1005f58d` (**legacy**
channel) was exercised, whereas the 2026-07-28 captures were preview/stable and
showed the fallback returning `not_found` across a workspace boundary.

**The reconciliation is therefore a BUILD-VERSION question and is answerable from
source history with no live probe:** diff the surface-resolution path between
`main-1005f58d` and current. Either the fallback was narrowed from global to
workspace-scoped at some point, or the two paths differ. Do that before standing up
any live test.

---

## 6b. DOC GAPS — audited 2026-07-29, ready to execute, NOT yet done

`FYI.md` is **done** (`92be9985`). Three tracked-doc gaps remain. All three are
small, all three were caused by writing the docs *before* the launcher trap and
the flag defect were discovered.

| # | File | Gap | Fix |
|---|---|---|---|
| D1 | `README.md` | 28 profile mentions but **zero** "lane-scoped"/"per-lane" language, and `limux-preview-sessions` appears in **no tracked doc at all** | State that profiles are scoped per build lane, and name the launcher a user actually types |
| D2 | `CLAUDE.md` | "The two-binary gotcha" section has **0** profile mentions; reality is now 7 launcher pairs across 4 lanes | Extend the gotcha: a profile saved under one lane is invisible to every other lane, by design |
| D3 | `AGENTS.md` | 16 profile mentions; records *why* channel/profile must not be folded (good) but not the operator-visible consequence | Add the lane-scoping consequence |

**Why D1 is the one that matters:** the operator cannot discover
`limux-preview-sessions` from any tracked file. `limux` (stable) has no `profile`
command; `limux-preview-profiles` — the name that says "profiles" — is the *empty*
lane. Someone following the README today concludes their workspaces are gone.

**Do NOT fix by renaming or deleting a lane** until after the compact — the data
lives inside the VHDX, `~/.local/share/limux/**` is operator do-not-delete, and
removing anything frees nothing on C: anyway.

Routed to tutu (repo docs lane). If tutu would rather not carry someone else's
feature docs, this is mine to land post-compact via ephemeral worktree off
`origin/main` — ask before opening a PR, per repo convention.

## 6c. VERSION + RUNTIME AUDIT — operator-tasked 2026-07-29, nothing landed yet

Operator asked huno/tutu/fire to clean up limux docs, version, and runtime. Audited
before proposing. **Nothing lands before the compact.**

### Installed lanes, verified 2026-07-29

| Launcher | Version | Build | Channel | Note |
|---|---|---|---|---|
| `limux` | 0.2.3 | `c757056d2539` | stable | **no `profile` command** |
| `limux-preview-profiles` | 0.2.3 | `93132aea47b1` | preview:profiles | has it, **empty lane** |
| `limux-preview-sessions` | 0.2.3 | `93132aea47b1` | preview:sessions | has it, **holds the data** |
| `limux-preview` | 0.2.1 | `f1db1d5a6005` | preview:default | stale (2026-07-14) |
| `limux-legacy` | 0.2.2 | `1005f58d92a1` | legacy | reve's incident build |

### The headline: two different builds both claim `0.2.3`

They differ by **25 commits including a `feat:`**. `93132ae` did **not** bump the
workspace version — it only touched `rust/limux-control/Cargo.toml` for a dependency.
So one `0.2.3` has `profile` and one does not.

**Do not over-read this.** The embedded short SHA *does* disambiguate — which is
precisely why limux was named the **exception** in the fleet build-provenance rule.
Provenance is fine. The narrower, real defect: **the semver is uninformative.** Anyone
reasoning from version alone — changelog, release notes, "which version has profiles?"
— gets a wrong answer.

### Items, with owners

| # | Item | Owner |
|---|---|---|
| **V1** | No MINOR bump for a `feat:` — profiles shipped under an already-published `0.2.3` | **huno** |
| **V2** | `CHANGELOG.md` not updated for PR #99. Its own first line: *"All notable Limux changes should be recorded here when a PR merges."* The `0.2.3` entry is dated 07-19; profiles merged 07-25 and is absent | **huno** |
| **V3** | install-id breaks convention — established: `main-c757056d2539-adv-remediated-20260721`, `preview-f1db1d5a6005-20260714T175555Z`; mine: bare `93132aea47b1` | **huno** |
| **V4** | The entire `0.2.x` line is **untagged** — 14 tags exist, newest `v0.1.19`. Pre-existing | **tutu** |
| **R1** | Same build under two lane names; the one named "profiles" is empty | **huno** |
| **R2** | `preview:default` stale at 0.2.1 — retire, refresh, or document | **tutu** |
| **R3** | `legacy` 0.2.2 retained — keep until the H1 build-version diff is done, then decide | **tutu** |
| **D1–D3** | Doc gaps, see §6b | **tutu** or **huno** |

### Sequencing — read before acting

- **R1 must NOT be touched before the compact.** Data is inside the VHDX,
  `~/.local/share/limux/**` is operator do-not-delete, and removing a lane frees
  nothing on C: anyway.
- **V2 cannot use the coordination carve-out** — `CHANGELOG.md` is a repo source doc,
  and the guard correctly refuses anything outside `FYI.md` / `HANDOFF*.md` /
  `*_INBOX/`. It needs a real branch + PR, which needs asking first per repo convention.
- Post-compact: ephemeral worktree off `origin/main`, ask before any PR.

## 7. TRAPS FROM THIS SESSION — do not relearn

1. **Test the route, not the parser.** §2. A feature can pass every unit test and
   be unreachable through the only path users have.
2. **If you need a command's exit code, never put it in a pipeline.**
   `OUT=$(cmd); RC=$?` then filter the variable — identical in bash and zsh.
   Seven instances across three sessions in one night; it is the most-repeated
   failure and the only purely mechanical one. `${PIPESTATUS[0]}` (bash) vs
   `${pipestatus[1]}` (zsh, lowercase, 1-indexed) is a real incompatibility that
   caused an unset-param abort and left a production file mutated.
3. **Restore goes in `trap 'restore' EXIT`, never a trailing line.** A trailing
   restore is unsound on *every* abort path. This caused the night's only real
   damage.
4. **A single measurement cannot reveal its own instrument error.** Vary the
   instrument until two independent readings agree.
5. **Existence is not consequence.** A dir existing ≠ data in it; a dir tracked ≠
   its contents tracked; a ref pushed ≠ the blob you verified.
6. **Records of mutable state go stale.** Clipboard contents, gitignore status,
   installed versions. Prefer recording claims whose truth does not depend on
   mutable config ("git history is durable" needs no re-check; "archive/ is
   gitignored" needs it forever). Re-read at point of use.
7. **After changing a state, sweep the records describing it.** One `rg`, at the
   moment you already know what changed.
8. **An alarming reading gets audited; a reassuring reading gets accepted** — and
   the reassuring one closes the inquiry, so nobody revisits it. Trigger the
   verification checks on *closure*, not on alarm.
9. **Knowing a control does not fire it.** I ran a null case, published the
   result, stated what it meant, and 208 seconds later drew a conclusion the same
   capture forbade. These are decision-point controls, not knowledge.
10. **A closing claim travels furthest and is audited least.** This file originally
    recorded "bounded to 1 hit" as verified. Chain: a narrow regex returned 1 → its
    author asserted "no others hiding" → I wrote it here as fact with a peer's
    verification attached → three more sweeps found three more hits. Four hops from
    a pattern that could not match either line as written. **For a completeness
    sweep, grep the SUBJECT ALONE and read the hits** — markdown backticks defeat
    literal-phrase patterns, and an under-matching instrument fails *silently and
    in the reassuring direction*: it returns a smaller number, and a smaller number
    reads as good news. Reading 114 lines has no silent failure mode; any pattern
    narrower than the subject does.
11. **Classify a sweep, do not just count it.** Dated + attributed artifacts
    (inbox packages, preserved drops, audits) claim a *moment* and must be LEFT —
    they are the evidence. Undated prose in a living reference doc claims *now* and
    rots. "Fix all hits" corrupts the incident record.
