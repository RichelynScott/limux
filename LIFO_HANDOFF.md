# Limux Lifo Handoff

Author/runtime/date: lifo / Codex gpt-5.5 (xhigh) / 2026-06-29 09:15 EDT.

## 2026-07-21 Stable-Default Launcher And Hcom Recovery Wedge

- Active branch: `lifo/default-stable-launcher-20260721`.
- Implementation commit: `de444b42b427bf3930c111ae053338728276d157`.
- Open PR: <https://github.com/RichelynScott/limux/pull/79>.
- Stable installs now promote the plain `limux` / `limux-cli` aliases;
  legacy installs update only explicit `limux-legacy` / `limux-legacy-cli`
  rollback aliases. `limux doctor` fails when default launchers drift from the
  named stable install.
- Verification is green: focused doctor tests passed 8/8, release runtime
  isolation smoke passed, `./scripts/check.sh` exited 0, and `git diff
  --check` passed.
- The live operator aliases were corrected without restarting the stable host:
  plain `limux` identifies stable v0.2.3 at source `1a26bda0`; `limux-legacy`
  retains v0.2.2 at source `1005f58d`; plain `limux doctor --json` reports
  `ok=true` with CLI/host congruence.
- TaskMaster master task 30 is `in-progress` pending PR merge and source-based
  promotion. Master task 29 tracks terminal reflow corruption. Resource-crash
  tasks 2 and 3 are `review`; their PRs #67 and #68 remain open/conflicting and
  need clean current-main replacement branches.
- PR #58 remains open/conflicting. Its current-main TaskMaster additions were
  recreated through reviewed CLI in PR #79; close #58 as superseded after PR
  #79 is safely established.
- Ctrl+C selection-copy and pane-create timeout fixes shipped in v0.2.3 with
  automated coverage, but no durable human live selection/no-selection smoke
  record was found. Do not claim that manual smoke complete.
- Current hcom delivery is degraded by recovery transaction
  `a35d2f40-dc23-4745-bfe7-d255420107a8`, expired but stuck in `launching`.
  The live Codex process remains visible, while authenticated `lifo` list/send
  commands fail with ambiguous control authority. Exact evidence is owner-routed
  at `/home/riche/MCPs/hcom/HCOM_MGR_INBOX/DEFECT_FROM_lifo_20260721_expired-recovery-blocks-live-binding.md`.
- Immediate next action: request exact-head Codex bot review on PR #79 through
  the temporary gh fallback watcher, address findings, merge only after the
  review gate, then install/promote from merged main. Keep #67/#68 remediation
  separate and preserve the peer-owned Reve incident file.

## 2026-07-13 Reconciliation And PR State

- Active branch: `lifo/restart-checkpoint-20260711`; Option A planning commit
  `bf10273fb51e7899817a140bfcf15365a21f2d07` is pushed with remote parity.
- Canonical visibility/restart contract and PRD-I are durable. TaskMaster
  `23.1`, `23.2`, and `23.3` are done; pending implementation ladder is
  `23.4` through `23.11`, with `23.11` explicitly operator-gated.
- The base Task 23 product slice adds right-click canonical workspace and
  Surface/Pane context plus a copyable read command. A TDD fix now propagates
  `read-screen --scrollback --lines` through CLI, GTK bridge, Ghostty history,
  and the standalone dispatcher. This is not the future G4 runtime-incarnation
  or HCOM-fencing implementation.
- Verification is green: focused RED/GREEN tests, `cargo check -p
  limux-host-linux`, 346 host tests, `./scripts/check.sh`, Ghostty resource
  validation, and an unchanged-retry full Xvfb smoke. The first Xvfb attempt
  timed out after stage 6 had already created the split/proof; the retry passed.
- Task 24 project-skill guidance is in progress, not complete. Hamo's controlled
  elevated relaunch succeeded, but exact prior sandbox and approval-policy
  restoration evidence remains required before closeout or global promotion.
- Hamo owns read-only reconciliation evidence and writable Git execution only;
  lifo owns TaskMaster, mutation decisions, commit grouping, PR, and review.
- Immediate next action: exact-stage four logical groups, push, open one Limux
  PR, then run exact-head bot/peer review. Do not install, restart, merge, or
  activate the daily driver from this checkpoint.

## 2026-07-12 Post-Restart Verification

- Limux legacy runtime `0.2.0` build `068872a1e162` is responding on the normal
  socket. `doctor --json` returned exit 2 only because sandboxed process
  discovery could not see the host PID; socket and resource checks passed.
- Two TaskMaster workspaces now exist:
  - `workspace:7b4f512e-472c-458d-9e31-8e279ec53175`, surface
    `166:terminal-0`, visibly contains Sage.
  - `workspace:895f67c3-6fa2-4b3d-93e6-d6d8e57a5b2e`, surface
    `161:terminal-0`, still contains and persists Dino.
- Sage hcom recovered after a live nonce round trip. Event `430023` confirmed
  authoritative session `019f0a87-5721-78d0-b360-f3e96e3827c2`; hcom now
  reports process, live delivery, terminal, hook, and transcript bindings with
  no control warnings.
- Sage's own `limux --json identify` incorrectly reported Lifo's focused
  workspace/surface (`aaacde98...`, `126:91b2...`) instead of Sage's visible
  `166:terminal-0`. Therefore Sage is usable through hcom but is not correctly
  bound to its Limux surface identity.
- Stale collision subscription `sub-4a47486e` was removed. Its alerts replayed
  Lifo status events from 2026-06-19 and were not evidence of current Sage
  edits.
- Do not close, exit, or merge either TaskMaster workspace until ownership and
  surface association are repaired. In particular, do not treat the old
  `161:terminal-0` pane as Sage; it is Dino.

## 2026-07-12 Linux Restart Checkpoint

Current authoritative state before the operator's Linux restart:

- Branch `lifo/restart-checkpoint-20260711` is clean and pushed through
  `bceacce`.
- Limux skill capability commits are durable on the remote branch:
  - `957a561 docs(skills): add existing-pane hcom resume recovery`
  - `bceacce docs(skills): fail closed on pane identity mismatch`
- Canonical changed files are `skills/limux-a2a/SKILL.md` and
  `skills/limux-use-guide/SKILL.md`. The live Codex mirror at
  `/home/riche/.agents/skills/limux-use-guide/SKILL.md` was also updated and
  validated.
- Durable notifications were written to both global-config inboxes as
  `NOTICE_FROM_lifo_limux-existing-pane-hcom-resume-capability_20260712.md`.
  Niru and Kazu were stopped/unavailable in hcom, so live notification failed;
  the inbox files are the delivery record.

Critical live incident:

- The requested TaskMaster surface was
  `workspace:895f67c3-6fa2-4b3d-93e6-d6d8e57a5b2e`, surface
  `161:terminal-0`, expected agent `sage`, authoritative Codex session
  `019f0a87-5721-78d0-b360-f3e96e3827c2`.
- An initial duplicate Sage attachment in Windows Terminal was detected and
  exited. Sage was then correctly resumed in the Limux pane and briefly passed
  UUID, `limux-host` ancestry, full hcom binding, surface-env, and nonce ACK
  `429837`.
- The original Limux host later terminated at approximately 16:25 EDT. A first
  restart attempt used a command-runner-owned `nohup` process and was killed by
  that runner; do not classify that second termination as a Limux crash.
- The detached replacement initially omitted the interactive `PATH`, causing
  restored panes to report `/bin/sh: hcom: not found`. It was replaced by user
  service `limux-manual-20260712-162856.service` with the correct `PATH`; that
  service is expected to stop with the Linux restart.
- After restore, TaskMaster `161:terminal-0` and persisted
  `$HOME/.local/share/limux/session.json` both mapped to `dino`, UUID
  `019ea7ce-aec1-71c3-8233-9ab52a47bb68`, not Sage. Hcom reported Dino live
  with missing transcript binding and Sage inactive. No `/exit` or further
  injection was performed because that could kill Dino.

Immediate next action after restart:

1. Do not trust or exit the TaskMaster first pane based on its workspace alone.
2. Before any mutation, capture `limux list-panels`, `read-screen`, the matching
   `session.json` agent record, `hcom list -v`, and process ancestry.
3. If the visible/persisted/hcom identities still disagree, preserve evidence
   and debug the Limux restore/session-association path. Do not hand-edit
   `session.json` and do not blindly run `hcom r sage`.
4. Only resume Sage after proving the target surface is not occupied by Dino or
   another live agent; require one native client, `limux-host` ancestry,
   authoritative UUID, complete hcom bindings, and a nonce ACK.

Restart hold: no source edits or commits remain unpushed. Live Sage recovery is
blocked on the corrupted TaskMaster pane/session mapping.

## 2026-07-11 PC Restart Checkpoint

Current authoritative state:

- PR #56 (`chore(release): prepare Limux 0.2.1`) received a clean Codex bot
  review on exact head `f79485a67afa8e513ae86d98ab578806bce29ea9` and
  squash-merged to `main` as `57347774852447032406eb9a350d16ac259fc401`.
- Local `main` was fast-forwarded to that merge and matched `origin/main`
  cleanly before this checkpoint branch was created.
- Full source gates passed before merge: `./scripts/check.sh`, Ghostty resource
  validation, runtime-isolation smoke, and all eight Xvfb integration stages.
- An isolated preview install of product SHA `bf20af1ffa4b` reported Limux
  `0.2.1`, ran simultaneously with the legacy daily driver on its own socket,
  and passed live `doctor --json` with exit code 0.
- The legacy daily driver remains Limux `0.2.0` build `068872a1e162`. No stable
  or legacy runtime replacement was performed before the PC restart.
- TaskMaster `product-hygiene` subtask `1.1` remains `review` until the exact
  merged-main SHA is preview-installed and verified. Master task `22` is
  correctly `done`.

Immediate next action after restart:

1. Rebind `lifo` through hcom and verify `main` still equals `origin/main` at
   `5734777`.
2. Build and install exact merged `main` into the isolated preview channel.
3. Run the post-install checklist against that exact SHA. Do not promote or
   replace the stable/legacy daily driver until the checklist passes and the
   operator approves the runtime restart window.

Restart hold: all release code is merged and pushed; no load-bearing source
work is uncommitted. This checkpoint branch exists only to make the resume
state durable before the announced PC restart.

## 2026-07-10 Primary Checkout Reconciliation

Current remote `main` is `efbca2a` after PR #49 merged the restored GTK
child-teardown log-flood fix. Draft PR #50 carries the newer TaskMaster
`master` state where task 21 is done and task 22 tracks this reconciliation;
the TaskMaster store in this documentation-only PR still stops at task 20.

The previous primary checkout remained parked on the merged June 27 branch
with mixed historical state. Unique local evidence was preserved outside the
public repository before reconciliation; public mechanics and current task
state are staged in draft PR #50.

Current open work:

- Draft PR #50 stages `reconcile-via-limux`, its PRD-lite, and TaskMaster task
  22. Global installation is held for cross-runtime review and dogfood.
- One stale lifecycle worktree contains an unlanded explicit-targeting patch
  for `agent-team`; preserve and port it on a separate current-main branch
  before removing that worktree.
- The daily-driver runtime is still build `068872a1`; merged `main` has not yet
  been installed over active Limux sessions.

Immediate next action: finish worktree removal through the no-loss gate, land
or park the explicit-targeting patch through its own PR, obtain review on PR
#50, then schedule a reviewed runtime install/restart window.

## 2026-07-08 Restart Checkpoint - TaskMaster Update

Runtime restart was requested because TaskMaster tooling/state was updated and
active sessions need to restart onto the corrected environment.

Durable state before restart:

- TaskMaster reconciliation branch:
  `lifo/taskmaster-product-hygiene-close-20260708`.
- Commit: `9665241 chore(taskmaster): close product hygiene lane`.
- PR: <https://github.com/RichelynScott/limux/pull/46>.
- PR state when checkpointed: open, merge state clean, no status checks posted.
- Verification run:
  `task-master-reviewed list --tag product-hygiene`,
  `task-master-reviewed tags list --show-metadata`, and `git diff --check`.

What changed in PR #46:

- `product-hygiene` TaskMaster tag now reports `3/3` done after PR #43 and
  PR #45 landed.
- Active TaskMaster tag remains `cmux-parity-20260707`.

Important local checkout caveat:

- Primary checkout `/home/riche/MCPs/limux` is still on stale/gone branch
  `lifo/hermes-workspace-highlight-resize-20260627` at `16a638d` with unrelated
  dirty/untracked local state.
- Clean TaskMaster reconciliation work lives in
  `/tmp/limux-taskmaster-reconcile-20260708`.
- After restart, do not trust `task-master-reviewed list` from the stale
  primary checkout until it is reconciled onto current `origin/main`.

Hcom restart notice:

- Sent checkpoint request on thread `runtime-restart-taskmaster-20260708` to
  live sessions `gile`, `moka_aux`, `mori`, `rumi`, `sage`, `mula`, `kazu`,
  `niru`, `mimi`, and `boho`.
- Acks observed from `mimi`, `niru`, `gile`, and `mula` before this note.

Immediate next action after restart:

1. Check PR #46 for bot/owner comments.
2. If clean, merge PR #46.
3. Reconcile the primary Limux checkout from `origin/main` before resuming
   normal Limux work or trusting local TaskMaster output there.

## Immediate Next Action

The current user-local Limux symlink now points at the reviewed branch build
from `lifo/hermes-workspace-highlight-resize-20260627`:

- Branch/head: `60d960302cbd` (`fix(host): coalesce terminal resize updates`).
- Install root:
  `/home/riche/.local/limux-reviewed/resize-stability-60d9603`.
- Symlinks:
  `/home/riche/.local/bin/limux` and `/home/riche/.local/bin/limux-cli`
  both point into that install root.

Important runtime caveat: at 2026-06-29 09:14 EDT, the currently running
`limux-host` process was still PID `42009` from the old install path
`/home/riche/.local/limux-reviewed/lifo-hermes-highlight-cedcb3a/libexec/limux-host`.
Open Limux windows keep their original host binary until closed/relaunched. To
actually run `60d960302cbd`, the operator must close the old Limux host and
start `limux` again after this handoff update.

PR #6 was pushed to GitHub and a fresh `@codex review` was requested at
`https://github.com/RichelynScott/limux/pull/6#issuecomment-4833008516`.
At 2026-06-29 09:15 EDT, the bot had not yet posted a new review for
`60d960302cbd`; the prior bot review only covered `ffa6ec3021`.

If runtime resize corruption still reproduces after the restart, capture:

- exact agent/runtime type (`codex`, `claude`, `hermes`) and whether it uses a
  normal-screen TUI or alternate screen;
- whether the pane was being drag-resized, workspace-switched, refocused, or
  cross-monitor moved;
- `limux read-screen --scrollback --lines 500` from the affected surface, if
  readable;
- the running host path from `ps -eo pid,cmd | rg 'limux(-host|-cli)?'`.

## 2026-06-29 Resize Stability Fix

Research sources checked:

- Limux upstream PRs #83, #95, and #100 all point at Ghostty surface sizing
  correctness around physical pixels / GLArea framebuffer scale.
- cmux issue #3052 and PR #4765 tie Claude/Ink live-region duplication to
  redundant resize/layout events and recommend coalescing pixel-only resizes.
- cmux issue #2789 ties idle prompt growth on resize to Ghostty shell
  integration / OSC prompt marker behavior.
- cmux issue #5299 records column-grow/refocus corruption in libghostty's
  incremental grow path.
- tmux PR #5101 records a pane-resize reflow bug where saved cursor restore can
  overwrite prior shell output.

Fix completed:

- Added a trailing 90 ms resize coalescer in
  `rust/limux-host-linux/src/terminal.rs` so GTK drag-resize storms do not send
  every intermediate size into Ghostty / SIGWINCH-sensitive agent TUIs.
- Added redundant-size suppression by checking `ghostty_surface_size()` before
  calling `ghostty_surface_set_size`.
- Preserved physical-pixel sizing for HiDPI / fractional scale behavior.
- Relaxed Ghostty resource discovery in `rust/limux-host-linux/src/main.rs` so
  shell integration can be installed and used even when compiled terminfo is
  absent.
- Added installer fallback in
  `scripts/user-local-install/install-user-local.sh` to copy
  `/home/riche/MCPs/limux/ghostty/src` as Ghostty resources when
  `ghostty/zig-out/share/ghostty` is unavailable.

Verification completed:

- `cargo test -p limux-host-linux surface_resize`
- `cargo test -p limux-host-linux resolves_shell_integration_without_terminfo`
- `cargo test -p limux-host-linux resource_env_sets_shell_integration_without_optional_terminfo`
- `cargo test -p limux-host-linux surface_size_match_uses_physical_pixel_dimensions`
- `scripts/user-local-install/install-user-local.sh --dry-run --profile release --install-id resize-stability-check`
- `./scripts/check.sh`
- `git diff --check`
- `python3 /home/riche/.codex/scripts/static_check_no_delete_api.py --target-dir scripts/user-local-install`
- `cargo build --release -p limux-cli --bin limux-cli`
- `cargo build --release -p limux-host-linux`
- `scripts/user-local-install/install-user-local.sh --apply --profile release --install-id resize-stability-60d9603`
- `/home/riche/.local/bin/limux --help`

Known limitation: installer now finds Ghostty shell integration from
`ghostty/src`, but still reports `Ghostty terminfo: not found`. Do not treat
terminfo generation as solved; that remains a separate packaging task.

## 2026-06-27 Runtime Install Correction

Root cause of the post-restart mismatch: PR #6 work had been pushed, but the
user-local install symlink had not been updated before the operator restarted
the PC/Limux. Before correction, `/home/riche/.local/bin/limux` resolved to
`/home/riche/.local/limux-reviewed/main-20260622-2fcfc55/bin/limux`.

Corrective actions completed:

- Verified branch state: `lifo/hermes-workspace-highlight-resize-20260627`
  tracking origin at `cedcb3ade43d`.
- Ran `cargo check -p limux-host-linux`.
- Ran `cargo test -p limux-host-linux`: 226 passed.
- Ran `./scripts/check.sh`: passed.
- Built release CLI and host artifacts.
- Ran user-local installer:
  `scripts/user-local-install/install-user-local.sh --apply --profile release --install-id lifo-hermes-highlight-cedcb3a`.
- Verified install checksum manifest from inside the install root:
  `sha256sum -c SHA256SUMS`: all OK.
- Verified new CLI wrappers print expected `limux --help` / `limux-cli --help`.

The installer archived the previous `~/.local/bin/limux` and `limux-cli`
symlinks under
`/home/riche/.local/limux-reviewed/archive/20260627T101506Z/`.

If the operator resumes Limux runtime testing, first confirm that no running
`limux-host` process still points at `main-20260622-2fcfc55`.

If the operator resumes Limux runtime testing, start from
`/home/riche/MCPs/limux` on branch
`lifo/hermes-workspace-highlight-resize-20260627`.

If live runtime issues continue, capture
`~/.local/state/limux/logs/limux-host.log` and exact
`GSETTINGS` / `GTK` / `GDK` / `XDG` / `LIMUX` environment values from the
affected pane.

## 2026-06-22 Restart Prep

Nato installed the reviewed current-main build as
`main-20260622-2fcfc55` and verified:

- `~/.local/bin/limux` points at `main-20260622-2fcfc55/bin/limux`.
- `SHA256SUMS` verified OK.
- Old `29fd2ff` symlinks were archived under `archive/20260622T203030Z`.
- Source tree had no source patch from the GTK/GLib investigation.

Before the operator's Limux restart, this session found generated hook/session
logs under repo-local `logs/`. They are not source artifacts, so `.gitignore`
now ignores `logs/` to prevent accidental commits. Do not commit those generated
logs.

Current runtime gate:

- Operator must fully restart Limux to unload the old live host.
- If GTK/GLib launch/runtime errors recur on the current build, capture exact
  user action/time, host log, session JSON snapshot, process/socket list, build
  id/symlink target, and whether multiple runtimes are active before changing
  source code.

## 2026-06-22 Merge Closeout

| PR | Result | Evidence |
|---|---|---|
| #2 `feat(host): add workspace attention UI lane` | Merged to `main`. | Merge commit `794f2233b3310e5ccde47b22f038494c83725116`. |
| #3 `feat(host): improve workspace sidebar notifications` | Merged to `main`. | Merge commit `f28ee2ed228cdd648f02f39ac760fb6931aeabf6`; Codex bot P3s fixed in `6f70858`. |
| #4 `fix(host): harden terminal selection copy and paste` | Merged to `main`. | Merge commit `c0534074c245db85af38bdd40e6de96b0b5b1206`. |
| #5 `docs(limux): track future integration options` | Merged to `main`. | Merge commit `9c5d9862b345e3aefa88626a3fd0a9a842561380`. |

Final local verification on merged `main`:

```bash
./scripts/check.sh
```

Result: passed on 2026-06-22 after the #2-#5 merge sequence.

Notes:

- Codex PR auto-review was unavailable during final closeout because the
  account hit the usage window. The #3 P3 findings were fixed before merge, #4
  had a prior clean Codex bot review on its head, and the final merged `main`
  passed the canonical local gate.
- Future Cursor/Limux integration ideas are tracked in
  `docs/future-improvements/limux-cursor-integration-options-after-pr-greenlight.md`.
- Do not start Cursor integration implementation or TaskMaster tasking until
  the operator explicitly opens that lane.

### HUMAN NOTE/ADD: THIS SECTION AND REQUEST WAS DIRECTLY ADDED BY HUMAN AFTER ALL SESSIONS FINISHED COMPACTION AND I CLOSED THEM OUT AND I WAS GOING TO CLOSE DOWN THE LIMUX PROCESS BUT SAW THESE ERRORS I WANTED TO MAKE SURE WE DOCUMENTED SO YOU DOUBLE CHECK THAT THESE ARE GETTING ADDRESSED OR GOT ADDRESSED: 
"""
➜  ~ limux

(limux-host:99589): GLib-GIO-CRITICAL **: 14:52:13.844: g_settings_schema_source_lookup: assertion 'source != NULL' failed

(limux-host:99589): Gtk-WARNING **: 14:52:13.902: While adding page: duplicate child name in GtkStack: terminal-0

(limux-host:99589): Gtk-CRITICAL **: 14:52:13.902: gtk_box_append: assertion 'gtk_widget_get_parent (child) == NULL' failed
limux: control socket at /run/user/1000/limux/limux.sock

(limux-host:99589): Gtk-WARNING **: 14:52:13.913: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-horizontal-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-WARNING **: 14:52:13.913: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-vertical-symbolic.svg: Unrecognized image file format
libEGL warning: failed to get driver name for fd -1

libEGL warning: MESA-LOADER: failed to retrieve device information

libEGL warning: failed to get driver name for fd -1

MESA: error: ZINK: vkCreateInstance failed (VK_ERROR_INCOMPATIBLE_DRIVER)
libEGL warning: egl: failed to create dri2 screen

(limux-host:99589): Gdk-WARNING **: 14:56:20.383: Compositor doesn't support moving popups, relying on remapping

(limux-host:99589): Gtk-WARNING **: 14:56:54.573: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-horizontal-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-WARNING **: 14:56:54.573: Failed to load icon /home/riche/MCPs/limux/rust/limux-host-linux/icons/hicolor/scalable/actions/limux-split-vertical-symbolic.svg: Unrecognized image file format

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.417: gtk_scrolled_window_get_child: assertion 'GTK_IS_SCROLLED_WINDOW (scrolled_window)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.417: gtk_viewport_get_child: assertion 'GTK_IS_VIEWPORT (viewport)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:48.418: gtk_stack_set_visible_child_name: assertion 'GTK_IS_STACK (stack)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_scrolled_window_get_child: assertion 'GTK_IS_SCROLLED_WINDOW (scrolled_window)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_viewport_get_child: assertion 'GTK_IS_VIEWPORT (viewport)' failed

(limux-host:99589): Gtk-CRITICAL **: 15:30:50.334: gtk_stack_set_visible_child_name: assertion 'GTK_IS_STACK (stack)' failed
"""

## Completed This Session

| Time | Item | Evidence |
|---|---|---|
| 2026-06-20 02:08 EDT | Continued G0 Limux stability work in isolated worktree. | Worktree `/home/riche/MCPs/limux-workspaces-sidebar-notifications`, branch `lifo/g0-stability-20260620`, base `49fb4cf`. |
| 2026-06-20 02:39 EDT | Integrated multi-subagent G0 fixes. | Commit `276aafd fix(host): harden g0 runtime stability`. |
| 2026-06-20 02:49 EDT | Opened stacked PR for G0 stability. | PR `https://github.com/RichelynScott/limux/pull/1`, base `lifo/workspaces-sidebar-notifications-20260620`. |
| 2026-06-20 02:56 EDT | Addressed Codex bot P2 about display-dependent GTK unit test. | Commit `8798eaa test(host): skip gtk traversal test without display`. |
| 2026-06-20 03:06 EDT | Received Codex bot clean rereview. | Bot issue comment `4756805228`: "Didn't find any major issues" for `8798eaa839`. |
| 2026-06-20 03:08 EDT | Merged PR #1 after Codex bot clear and Halo verification. | Squash merge commit `299a8fc762dc5f4a168d7d37c8148c58d0aedb08`. |
| 2026-06-20 03:09 EDT | Reconciled local worktree after merge. | Local and remote `lifo/workspaces-sidebar-notifications-20260620` both at `299a8fc`; worktree clean. |

## Key Files For Context

| Path | Purpose |
|---|---|
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/rust/limux-control/src/socket_path.rs` | Runtime/debug socket path resolution; debug mode now ignores inherited runtime socket env unless `--socket` is explicit. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/rust/limux-cli/src/main.rs` | Hook notification debug records now include `resolved_socket`. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/rust/limux-host-linux/src/terminal.rs` | Ghostty surface sizing now passes physical pixels for HiDPI. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/rust/limux-host-linux/src/window.rs` | Wrapped workspace roots now descend to the real pane for focus/attention; GTK traversal regression test is headless-safe. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/scripts/tests/validate-split-icons.sh` | Static validator for split SVG source/package install paths. |
| `/home/riche/MCPs/limux-workspaces-sidebar-notifications/.taskmaster/docs/workspaces-sidebar-notifications-20260620.md` | TaskMaster experience note; wrapper saw no usable task store/config, so no task IDs were invented. |

## Verification

Manager-run checks before PR merge:

```bash
env -u DISPLAY -u WAYLAND_DISPLAY GDK_BACKEND=x11 cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
env -u DISPLAY -u WAYLAND_DISPLAY GDK_BACKEND=x11 cargo test -p limux-host-linux -- --nocapture
xvfb-run -a cargo test -p limux-host-linux find_leaf_pane_descends_wrapped_workspace_root_to_pane -- --nocapture
git diff --check && cargo fmt --check
./scripts/check.sh
```

Earlier G0 checks included `cargo test -p limux-control socket_path`,
`cargo test -p limux-cli hook`, `cargo test -p limux-host-linux terminal`,
`xvfb-run -a cargo test -p limux-host-linux window::tests::`,
`bash scripts/tests/validate-split-icons.sh`, and a no-delete static scan over
the new shell test helper.

## Current Git State And Branching

- Current worktree:
  `/home/riche/MCPs/limux-workspaces-sidebar-notifications`.
- Current branch:
  `lifo/workspaces-sidebar-notifications-20260620`.
- Current commit:
  `299a8fc762dc5f4a168d7d37c8148c58d0aedb08`
  (`fix(host): harden G0 runtime stability`).
- PR #1 is merged:
  `https://github.com/RichelynScott/limux/pull/1`.
- The spent feature branch `lifo/g0-stability-20260620` was left in place; do
  not add new work on it.
- The separate main checkout `/home/riche/MCPs/limux` had pre-existing
  Halo-owned dirt (`LIFO_HANDOFF.md`, `archive/`) and was not mutated by this
  closeout.

## Critical Behavior Rules

- Do not continue work on the spent G0 branch after PR #1. Start from
  `lifo/workspaces-sidebar-notifications-20260620` or a fresh branch from the
  intended base.
- Preserve Halo-owned/local dirt in `/home/riche/MCPs/limux`; use the isolated
  worktree above for this lane.
- `./scripts/check.sh` now runs plain `cargo test --workspace`; the GTK
  traversal regression test must remain safe without a display.
- Codex bot feedback is actionable even when Halo classifies it as
  non-blocking; fix it before merge when practical.

## Residual Risks

- The live stuck-left-click/copy-paste behavior still needs a fresh runtime
  repro capture if it reappears. The G0 patch improves adjacent focus/pane and
  terminal sizing behavior, but does not prove the live input bug is gone.
- Existing EGL/Mesa/Zink warnings remain environment/driver warnings unless
  they correlate with a reproducible Limux failure.
- True live-refresh of already-running Limux runtimes is not implemented;
  running hosts keep old in-memory code until restart.
