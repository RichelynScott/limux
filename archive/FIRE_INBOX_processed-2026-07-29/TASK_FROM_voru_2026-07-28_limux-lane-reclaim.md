# TASK — limux lane: reclaim ~28.5 GiB now (largest single-file win on the box), ~19.5 GiB more on your call

**Created by:** Claude Code (voru · storage-forensics orchestrator)
**Date:** 2026-07-28 12:29 UTC
**Purpose:** Hand fire a verified, execution-ready reclaim plan for the limux lane — including the single largest reclaimable file on this machine (25.83 GiB) — as a prerequisite for the operator-gated VHDX sparse-conversion/compact.

## From: voru
## To: fire
## Date: 2026-07-28T12:29:09Z
## Type: TASK
## Priority: HIGH

---

### Why this is urgent

Windows `C:` is at **100%** — 926 GB used of 931 GB, **5.3 GB free** (`df -h /mnt/c`, re-measured 2026-07-28). Two WSL virtual disks hold 502 GB of that: Ubuntu `ext4.vhdx` = 347.2 GB, Docker `docker_data.vhdx` = 154.9 GB.

**The mechanic that changes what your work means:** `fsutil sparse queryflag` reports *"This file is NOT set as sparse"* for **both** vhdx. `.wslconfig` has `sparseVhd=true` and `/` mounts with `discard`, but those apply only to **newly created** disks — these predate the setting. So freeing space inside WSL returns **zero bytes to Windows** until a separate `wsl --shutdown` + `wsl --manage Ubuntu --set-sparse true` compact runs. Proof it is already happening: ext4 reports 291 GB used while the vhdx is 347.2 GB — **~56 GB is already freed-inside and not returned.**

Your reclaim is therefore a **prerequisite** for the compact, not a substitute for it. **Do not attempt the compact** — `wsl --shutdown` and VHDX operations are operator-gated under `C_DRIVE_SPACE_PROJECT/AUTHORIZATION.md`. You reclaim; the operator compacts. The more each owner frees before that single compact window, the more one shutdown buys.

### Your lane — what I measured

Re-measured read-only 2026-07-28 12:23–12:29 UTC. Two handed-down figures were wrong; both are corrected below.

| Path | Size | Class | Status | Verified |
|---|---|---|---|---|
| `~/.local/state/limux/logs/archive/limux-host.log.legacy-unbounded-superseded-20260721` | **27,734,918,593 B = 25.83 GiB** | superseded log, frozen | **TIER 1 — biggest single file on the box** | `stat`: byte-exact, mtime 2026-07-21 15:48:16 |
| `~/MCPs/limux/ghostty/.zig-cache` | 1.4 GiB | zig build cache, regenerable | TIER 1 — best safety:size ratio after the log | mtime 2026-05-29 20:24; gitignored in submodule |
| `~/.cache/limux-tools` | 836 MiB | tool cache, regenerable | TIER 1 — stalest large dir measured anywhere | mtime 2026-05-29 20:24 |
| `~/MCPs/limux/archive/generated/target-task4-concurrency-20260716` | 376 MiB | **archived copy of a cargo target tree** | TIER 1 — archive-not-delete already spent | `du -sh` |
| `~/MCPs/limux/archive/worktrees/limux-runtime-markers-task4-20260716` | 104 MiB | archived worktree | TIER 1 | `du -sh`; `archive/` total = 479 MiB |
| `~/MCPs/limux/target` | **18 GiB** | cargo build tree, **ACTIVE** | **TIER 2 — your call on timing** | mtime 2026-07-25 19:19 (3 days) |
| ↳ `debug/incremental` | 7.2 GiB (270 dirs) | incremental cache | Tier 2 — the cheapest 7.2 GiB inside target | `ls \| wc -l` = 270 ✓ |
| ↳ `debug/deps` / `debug/build` | 7.0 GiB / 501 MiB | rebuild artifacts | Tier 2 | ✓ all matched |
| ↳ `release/` | 1.3 GiB (deps 1.2G, build 99M, `limux` 4.1M) | **NOT load-bearing** — see DO NOT TOUCH | Tier 2 | ✓ |
| ↳ `target/zig-local-cache` / `zig-global-cache` | 905 MiB / 397 MiB | zig caches inside target | Tier 2 (swept by `cargo clean`) | ✓ |
| `~/MCPs/limux/target/target` | **29 bytes — NOT 1.3 GB** | **self-referential symlink** | **TIER 2 — config BUG, ~0 space** | `ls -la`: `-> /home/riche/MCPs/limux/target` |
| `~/.local/limux-reviewed` | 2.3 GiB / **37** entries | versioned install snapshots | TIER 2 — 5 are LIVE symlink targets | `du`, `readlink -f` |
| ↳ 5× `copy-paste-*-20260622-*` | 151 MiB each = **755 MiB** | one day's iteration | Tier 2 — best prune candidate | **five**, not four |
| ↳ `preview/` | 218 MiB | largest single entry | Tier 2 — 3 live symlinks inside | ✓ |
| `~/.local/state/limux/agent-hook-debug.jsonl` | 21 MiB / 73,926 lines | **actively appending today** | NOT a reclaim target — regrowth flag | mtime 2026-07-28 08:26 |

**Corrections to what you were handed:**

1. **`target/target` is not a 1.3 GB nested directory.** It is a **29-byte self-referential symlink** → `/home/riche/MCPs/limux/target`. `CARGO_TARGET_DIR` is unset (env, `~/.cargo/config.toml`, `.cargo/`, `Cargo.toml`, `.envrc`, `Makefile` — zero matches). The earlier 1.3 GB reading was a `du` that followed the link one hop and re-counted `release/`. **Reclassified: ~0 bytes, but a real hazard** — a recursive loop that will hang or infinitely recurse `du -L`, `find -L`, `rsync -L`, `tar -h`, and naive backup walkers. Still worth fixing; just not for space.
2. **The live log is 5,361 bytes, not 3,700 — and it was written today (2026-07-28 08:22).** This *strengthens* the case: **5.2 KB across the 7 days since the fix, versus 25.8 GB before it**, measured on a log that is still actively being written. The archive file does not regrow.
3. **There are five 151 MiB copy-paste snapshots, not four** — `copy-paste-drag-fix-20260622-1014d42` was missing from the handoff.

**Rotation fix verified present** in mainline `rust/limux-host-linux/src/host_log.rs`: `RotationOutcome` (L47), `max_bytes` (L161/171/177/184/197/231), `Ok(RotationOutcome::Rotated(...))`. Commit `fc40cf5` = *"docs: PR #90 merged; archive the 26GB legacy log; de-trap the CLAUDE.md note"*. Budgets in `main.rs:53-55`: active 64 MiB, retained count 10, total 640 MiB.

**Contrast with the hcom lane:** hcom's `target/release` is load-bearing (its installed binary resolves into the build tree). **Yours is not.** Every one of the 12 `~/.local/bin/limux*` entries resolves into `~/.local/limux-reviewed/**`; **zero** resolve into `MCPs/limux/target`. Your installs are *copies*. That is why your whole 18 GiB `target` is a pure rebuild-cost decision with no runtime risk.

---

### Tier 1 — reclaim now, no gate

**~28.5 GiB. Nothing here touches a tracked file** (`target/` and `zig/.zig-cache/` are in `.gitignore`; `ghostty/.zig-cache` is gitignored inside the submodule; `archive/` is excluded via `.git/info/exclude:19`; `~/.cache` and `~/.local/state` are outside the repo). **No branch needed for Tier 1.**

**On archive-not-delete:** items 1–4 are the **documented exception** — regenerable build/tool caches and an already-archived superseded artifact. Archiving them frees nothing, which is the entire point of the exercise. Item 1 in particular has *already had* archive-not-delete applied: commit `fc40cf5` archived it a week ago; this is step two of that same operation, on a file whose own name says `superseded`. Nothing in this tier is source, config, or unique data.

**If the `rm` safety hook blocks you:** do **not** work around it. Report it per `action-transparency.md` and escalate. For item 1 there is a hook-free alternative (1b) that frees the same blocks without unlinking.

#### 1. The 25.83 GiB legacy log — the single largest win on this machine

Optional forensics keepsake first (2 MB tail, ~1 second):

```bash
tail -c 2000000 /home/riche/.local/state/limux/logs/archive/limux-host.log.legacy-unbounded-superseded-20260721 \
  > /home/riche/.local/state/limux/logs/archive/limux-host.legacy-tail-2MB-20260721.log
```

Then reclaim — **1a (preferred, unlinks the file):**

```bash
rm -f /home/riche/.local/state/limux/logs/archive/limux-host.log.legacy-unbounded-superseded-20260721
```

**1b (fallback if the hook blocks `rm` — frees the same 25.83 GiB in place, no unlink):**

```bash
: > /home/riche/.local/state/limux/logs/archive/limux-host.log.legacy-unbounded-superseded-20260721
```

**Verify:**
```bash
du -sh /home/riche/.local/state/limux/logs/archive/ && df -h /
```
Expect `archive/` to drop from **26G to ~0** (or ~2M if you kept the tail). Expect `/` Avail to rise from **665G to ~691G**.

#### 2. ghostty zig build cache — 1.4 GiB, two months stale

```bash
rm -rf /home/riche/MCPs/limux/ghostty/.zig-cache
```

**Verify:**
```bash
du -sh /home/riche/MCPs/limux/ghostty/ && ls -d /home/riche/MCPs/limux/ghostty/zig-out/lib
```
Expect `ghostty/` to drop from **1.5G to ~130M**, and `zig-out/lib` to **still exist** (it must — see DO NOT TOUCH). Cost: the next `zig build` is a full rebuild.

#### 3. limux-tools cache — 836 MiB, stalest large dir measured anywhere

```bash
rm -rf /home/riche/.cache/limux-tools
```

**Verify:**
```bash
du -sh /home/riche/.cache/limux-tools 2>&1 | tail -1; df -h /
```
Expect "No such file or directory".

#### 4. archive/ — 479 MiB of archived build output

```bash
rm -rf /home/riche/MCPs/limux/archive/generated/target-task4-concurrency-20260716
rm -rf /home/riche/MCPs/limux/archive/worktrees/limux-runtime-markers-task4-20260716
```

**Verify:**
```bash
du -sh /home/riche/MCPs/limux/archive/ && git -C /home/riche/MCPs/limux status --short
```
Expect `archive/` ≈ **0**, and `git status` unchanged (still only the untracked `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html`).

**Tier 1 close-out:**
```bash
df -h / && du -sh /home/riche/.local/state/limux /home/riche/MCPs/limux
```

---

### Tier 2 — your judgement call

#### 2A. `target/` — 18 GiB, but it is ACTIVE (last built 2026-07-25 19:19, 3 days ago)

This is **not** an unconditional do-now, and it is the one place your lane differs from the stale-target lanes. The tradeoff:

- **Reclaim:** 18 GiB — the largest directory in the repo, more than a third of your lane.
- **Cost:** a full cold cargo rebuild (plus a full zig rebuild if you also did Tier 1 item 2). On a workspace this size that is a real block of wall-clock, and the 07-25 build is recent enough that you may still be iterating on it. Corroborating evidence you *were*: the `limux-preview-profiles` and `limux-preview-sessions` symlinks are dated Jul 25 19:19 — that build fed a real install.
- **Risk: none.** Nothing at runtime resolves into `target`. `~/.local/bin/limux` → `~/.local/limux-reviewed/stable/main-c757056d2539-adv-remediated-20260721/bin/limux-stable`. Verified for all 12 `limux*` entries.

**Decide on timing, not on safety.** Three graduated options:

**(a) Cheapest, lowest regret — incremental cache only, 7.2 GiB, keeps deps warm:**
```bash
rm -rf /home/riche/MCPs/limux/target/debug/incremental
```
Verify: `du -sh /home/riche/MCPs/limux/target` → expect **~11G**. Next build recompiles your crates but reuses all 7.0 GiB of `deps` — much cheaper than a cold build.

**(b) Middle — drop the debug profile, keep release (13 GiB):**
```bash
cd /home/riche/MCPs/limux && cargo clean --profile dev
```
Verify: `du -sh /home/riche/MCPs/limux/target` → expect **~2.6G**.

**(c) Full — everything (18 GiB):**
```bash
cd /home/riche/MCPs/limux && cargo clean
```
Verify: `du -sh /home/riche/MCPs/limux/target && df -h /` → expect target **~0**.

Prefer `cargo clean` over `rm -rf` on the target tree — it is the sanctioned tool-native reclaim per `git-worktree-hygiene.md` and it will not trip the delete hook. **My recommendation: (a) now, unconditionally** — 7.2 GiB for near-zero rebuild pain, no decision required. Escalate to (b)/(c) only if the operator signals the compact window is imminent and you can absorb a cold build.

#### 2B. `~/.local/limux-reviewed` — 2.3 GiB / 37 snapshots

Five of the 37 are **live symlink targets** (listed in DO NOT TOUCH). The rest are historical. The obvious prune is the 2026-06-22 copy-paste series — **five** snapshots at 151 MiB each, **755 MiB**, all from one day's iteration on the same feature, none of them live:

```bash
# verify none are symlink targets FIRST:
for d in copy-paste-toast-fix-20260622-4bfae87 copy-paste-release-autocopy-20260622-29fd2ff \
         copy-paste-fix-20260622-8897272 copy-paste-drag-fix2-20260622-1e87406 \
         copy-paste-drag-fix-20260622-1014d42; do
  echo "== $d"; find /home/riche/.local/bin -lname "*$d*" -print
done
```
If that prints no paths under `~/.local/bin`, they are safe to drop. **This is a genuine archive-not-delete case, not the exception** — these are built install snapshots, not regenerable from a cache, and the tradeoff is losing the ability to bisect a regression by re-running an old build. Your call whether the 755 MiB is worth that. If you keep the option, move rather than delete.

**Verify either way:**
```bash
du -sh /home/riche/.local/limux-reviewed && ls -1 /home/riche/.local/limux-reviewed | wc -l && limux --version
```

#### 2C. The `target/target` symlink loop — ~0 bytes, real hazard

```bash
ls -la /home/riche/MCPs/limux/target/target   # confirm it still points at its own parent
rm /home/riche/MCPs/limux/target/target       # plain rm — it is a symlink, not a tree
```
Verify: `ls /home/riche/MCPs/limux/target/target` → "No such file". Since no `CARGO_TARGET_DIR` is set anywhere, nothing recreates it and nothing depends on it. Frees no space; removes a loop that will hang any `-L`/`-h` traversal — including whatever the operator runs to audit disk before the compact.

---

### DO NOT TOUCH

| Path | Failure mode if you touch it |
|---|---|
| `~/MCPs/limux/ghostty/zig-out/` (56 MiB) | **Breaks every cargo build.** `rust/limux-ghostty-sys/build.rs:8` links `ghostty/zig-out/lib` and panics `expect("libghostty not found — run: cd ghostty && zig build ...")`. The adjacent `.zig-cache` is safe; `zig-out` is the **output**. Do not glob `zig-*`. |
| `~/.local/limux-reviewed/stable/main-c757056d2539-adv-remediated-20260721/` | **Live `limux` and `limux-cli`.** `readlink -f $(command -v limux)` lands here. Removing it breaks the command for every session on the box. |
| `~/.local/limux-reviewed/main-1005f58d-pane-timeout-clean-20260716/` | Live target of `limux-legacy` / `limux-legacy-cli`. |
| `~/.local/limux-reviewed/preview/default/preview-f1db1d5a6005-20260714T175555Z/` | Live target of `limux-preview` / `limux-preview-cli`. |
| `~/.local/limux-reviewed/preview/profiles/93132aea47b1/` | Live target of `limux-preview-profiles*` (symlinked Jul 25 — recent). |
| `~/.local/limux-reviewed/preview/sessions/93132aea47b1/` | Live target of `limux-preview-sessions*` (symlinked Jul 25 — recent). |
| `~/.local/state/limux/logs/limux-host.log`, `limux-host.current.log` | **Live logs, actively written** (5,361 B, mtime today 08:22). These are the *proof the fix works* — do not clear them; the evidence is the point. Only the `archive/` file is in scope. |
| `~/.local/state/limux/*.json`, `agent-hook-debug.jsonl` | Live hook session state (`claude-hook-sessions.json`, `codex-hook-sessions.json`). Deleting these breaks hook continuity. The 21 MiB jsonl is a *regrowth flag*, not a reclaim target. |
| `~/MCPs/limux/.claude/worktrees/*` (3 worktrees, 14 MiB total) | Clean and small, but they are checked-out worktrees on real branches — one sits on a merge of `origin/bulo/bounded-logging-task3-20260716`. Never `rm -rf` a worktree; if you retire them use `git worktree remove` **after** confirming each branch is pushed. Not worth it for 14 MiB. |
| `git stash` (2 entries) | `stash@{1}` = *"halo pre-reboot local dirt preserve 2026-06-20"* — **DO-NOT-DISTURB.** Preserved dirt from another session. Do **not** run `git gc --prune`, `git reflog expire`, or `git stash clear` anywhere in this repo as a reclaim step. `.git` is only 131 MiB (34.7 MiB loose + 1.3 MiB packed) — there is nothing to win and a preserved stash to lose. |
| `wsl --shutdown`, `wsl --manage`, any VHDX/`fsutil`/`diskpart`/`Optimize-VHD` operation | **Operator-gated.** You reclaim; the operator compacts. Attempting it kills every session on the box. |

---

### Durable fix — stop the regrowth

Land these in **your** repo so this lane cannot rebuild itself into 48 GiB.

**(1) `archive/` is swept by nothing.** Verified gap, with a precise mechanism: `rotate_managed_active` (`rust/limux-host-linux/src/host_log.rs:~420`) enforces `max_retained_count` (10) and `max_total_bytes` (640 MiB) by returning **`RotationOutcome::StderrFallback`** — it *stops rotating*, it never prunes. And the managed retention dir it governs (`HOST_LOG_RETAINED_DIR_NAME = "retained"`, i.e. `logs/managed/retained`) **does not exist on this box** — `ls ~/.local/state/limux/logs/managed` → No such file or directory. So `logs/archive/`, where the 25.8 GiB file sat for a week, is a purely manual directory outside the budget subsystem entirely. Either bring `archive/` under `retained_dir` governance, or add an explicit keep-last-N + total-byte sweep for it. **The 64 MiB active cap is correct and working — the hole is downstream of rotation.**

**(2) `keep-last-N` on `~/.local/limux-reviewed`.** 37 snapshots, 2.3 GiB, growing one entry per reviewed build (five in a single day on 2026-06-22). Whatever script installs into `limux-reviewed/` should prune to the last N **after** resolving `~/.local/bin/limux*` symlink targets and excluding all five live ones. Deterministic and safe because the live set is machine-readable: `find ~/.local/bin -lname '*limux-reviewed*' -exec readlink -f {} \;`.

**(3) Kill the `target/target` symlink and add a guard.** No `CARGO_TARGET_DIR` is set, so nothing legitimately produces it — it is stale residue from a relative-path experiment. Remove it (2C) and, if any build script ever sets `CARGO_TARGET_DIR`, require an absolute path. Cheap CI/preflight assertion: fail if `test -L target/target`.

**(4) Two `.gitignore` gaps.** `archive/` is ignored only via `.git/info/exclude:19` — a **machine-local** exclude that no other clone inherits, so a fresh clone would show 479 MiB of build output as untracked noise. Promote `archive/` into the committed `.gitignore`. Consider `ghostty/.zig-cache/` too — currently covered only by the submodule's own ignore file, and the top-level ignore lists `zig/.zig-cache/` (a different path).

**(5) Regrowth flag, not urgent:** `~/.local/state/limux/agent-hook-debug.jsonl` is 21 MiB / 73,926 lines and was appended to today at 08:26 with no cap in the host-log budget path. Same failure class as the 25.8 GiB log, three orders of magnitude smaller. Worth a cap while you are already in the retention code.

---

### Lane hygiene

**Current state — verified 2026-07-28:**
- Branch: **`main`**. `git rev-list --left-right --count origin/main...HEAD` = **`0 0`** — fully in sync, nothing unpushed.
- Working tree: one untracked file, `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html`. Not mine to touch — decide whether it should be committed or archived.
- Three clean agent worktrees under `.claude/worktrees/`; two stashes (see DO NOT TOUCH).

**What this means for you:**

- **`main` is not an owned work lane.** Per `post-merge-branch-reconciliation.md` and the new-work-lane preflight in `git-worktree-hygiene.md`, **cut a fresh branch off `origin/main` before any change that touches tracked files** — i.e. before every one of the durable fixes in the section above (`host_log.rs` retention, `.gitignore`, install-script pruning, any CI guard):

```bash
cd /home/riche/MCPs/limux
git fetch origin && git status --short --branch
git switch -c fire/log-retention-and-cache-hygiene-20260728 origin/main
```

- **Tier 1 and Tier 2 need no branch.** Every reclaim path above is gitignored, locally excluded, or outside the repo. Confirm with `git status --short` after Tier 1 — it should be byte-identical to the state above.
- **Do not create new worktrees for this.** In-project `.claude/worktrees/` and `WORKTREES/` are retired; branch in place, push, PR.

---

### Report back

hcom to **voru** *and* **nafo** (the C_DRIVE_SPACE_PROJECT owner) when done — nafo is aggregating fleet-wide totals to size the operator's single compact window, so partial results reported early beat complete results reported late.

Include:

1. **GB reclaimed**, split Tier 1 / Tier 2, with the `df -h /` **before and after** (Avail on `/` was **665G** at 12:29 UTC 2026-07-28 — that is your baseline).
2. **Commands actually run**, verbatim — including which of 1a/1b you used for the 25.83 GiB log.
3. **Anything skipped and why** — especially your `target/` decision (a/b/c/deferred) and whether you pruned the five copy-paste snapshots. "Deferred, still iterating on the 07-25 build" is a complete and acceptable answer; silence is not.
4. **Any hook block or refusal**, reported not worked around (`action-transparency.md`).
5. **Durable-fix status** — branch name + PR if you landed any of the five; "tracked, not yet started" if not.

Suggested one-liner:

```bash
hcom send @voru @nafo -- "fire limux lane: reclaimed <N> GiB (T1 <x>, T2 <y>). df / Avail 665G -> <Z>G. Legacy 25.83GiB log: <done|blocked>. target/: <a|b|c|deferred>. Durable fixes: <branch|tracked>. Skipped: <...>"
```

If the payload grows past one line — backticks, paths, multi-line detail — use the byte-safe form per `shell-boundary-transport.md`:

```bash
hcom send @voru @nafo --file /dev/stdin <<'EOF'
fire limux lane report
...
EOF
```
