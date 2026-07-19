# PRD: Limux Resource-Crash Containment and Recovery

## Metadata

- Owner: Lifo / Limux manager
- Incident thread: `LIMUX_CRASH_CPU_RAM_EXHAUSTION`
- Coordination source: `/home/riche/Proj/C_DRIVE_SPACE_PROJECT/COORDINATION/LIMUX_CRASH_CPU_RAM_EXHAUSTION_THREAD_KICKOFF_20260716.md`
- Limux mission brief: `/home/riche/Proj/C_DRIVE_SPACE_PROJECT/COORDINATION/TO_LIFO_LIMUX_RESOURCE_CONSERVATION_20260716.md`
- Implementation branch: `lifo/resource-conservation-p0-20260716`
- Runtime policy: isolated preview first; no daily-driver install, restart, log mutation, or lifecycle mutation without a later operator gate

## Problem

After a Limux/WSL crash, Limux restored 26 workspaces, 56 terminal tabs, and 21
agent tabs. The host became resource-heavy and unstable. Host-visible samples
showed `limux-host` private anonymous/dirty memory growing from roughly 435 MiB
to more than 1.45 GiB during investigation, substantial CPU use, llvmpipe
renderer threads, and WSL/VHD wait-accounting anomalies. The approximately
26 GiB append-only host log barely grew during the current RSS increase, so the
active heap growth and historical unbounded logging are distinct defects.

Current source behavior creates four compounding risks:

1. Limux runs a global unconditional 8 ms `ghostty_app_tick` timer.
2. Limux does not bind or call Ghostty's `ghostty_surface_set_occlusion` API.
   Hidden tabs and workspaces therefore are not explicitly marked invisible,
   even though Ghostty documents occlusion as the mechanism that pauses hidden
   surface rendering. Software OpenGL fallback can amplify this across many
   restored surfaces.
3. Limux redirects process stderr into one append-only host log without size,
   rotation, retention, or repeated-warning controls.
4. Every eligible restored agent receives a startup command. The existing
   two-per-750-ms shell-sleep stagger caps at six seconds, causing all later
   agents to converge into a burst. Limux does not distinguish clean and
   unclean startup, suspend agents after a crash, or enforce pressure-aware
   resume concurrency.

The occlusion/tick path is a strong source-backed hypothesis, not yet a proved
sole root cause. Preview evidence must separately measure CPU, RSS slope,
renderer activity, and process continuity.

## Goals

1. Stop hidden/unmapped Ghostty surfaces from rendering while keeping their
   terminal processes alive and immediately repainting when shown again.
2. Eliminate avoidable high-frequency ticking when no visible renderer work
   requires it, without starving Ghostty mailbox or input events.
3. Bound future host-log growth without truncating, deleting, rewriting, or
   rotating the existing incident log in place.
4. Detect whether the previous Limux run exited cleanly. After an unclean exit,
   restore layout and terminal metadata but keep agents suspended until an
   explicit controlled resume.
5. Replace shell-sleep restore bursts with an observable concurrency controller
   that honors resource pressure and supports cancellation.
6. Prove the fixes in an isolated preview runtime at realistic workspace and
   surface scale before any operator promotion decision.

## Non-Goals

- Do not change the WSL memory cap as the first remedy.
- Do not force a renderer backend in the daily driver without a measured
  backend matrix and rollback.
- Do not signal, kill, resume, or otherwise mutate existing agent processes.
- Do not truncate, delete, rewrite, compress, or move the existing incident log.
- Do not install or promote a preview build from task generation alone.
- Do not treat PSI alone as proof of disk saturation or as the only resource
  signal on WSL.

## Functional Requirements

### FR1: Surface Visibility and Tick Discipline

- Add the missing Ghostty occlusion FFI binding.
- Drive Ghostty occlusion from actual GTK mapping/visibility state, including
  creation under an already-hidden stack, map, unmap, tab switches, workspace
  switches, reparenting, and destruction.
- Default fail-safe behavior must not render a surface that has never been
  proven mapped.
- Keep process/PTY state alive while invisible.
- Coalesce redundant visibility transitions.
- Separate mailbox wakeup handling from renderer cadence. Any adaptive tick
  policy must retain a bounded fallback for renderer messages that do not wake
  the app and must not create a busy loop while no surface is visible.
- Expose bounded debug counters or test instrumentation for visibility
  transitions, tick invocations, and queued render actions without logging per
  frame.

### FR2: Renderer Selection Diagnostics

- Record the requested GTK renderer policy, selected runtime renderer when
  discoverable, and software-renderer/fallback indicators once at startup.
- Preserve existing safe defaults until preview testing proves a better WSL
  policy.
- Define a preview-only matrix for current desktop GL behavior and any viable
  D3D12/GL alternative. A failed backend must fail closed to the documented
  fallback rather than crash or silently corrupt terminals.

### FR3: Prospective Bounded Host Logging

- Replace unbounded append-only growth with a bounded prospective policy.
- Preserve the current incident log byte-for-byte and never make startup scan
  its full contents.
- Rotate before stderr redirection using constant-bounded metadata operations.
- Use deterministic maximum active size, retained file count, and total budget.
- Avoid clobbering existing retained files. Archive/move-aside semantics must
  comply with repository and operator no-delete policy.
- Rate-limit or deduplicate repeated renderer warnings by stable category while
  retaining first occurrence, counts, and recovery transitions.
- Log setup failure must be visible and must not prevent Limux from starting.

### FR4: Clean/Unclean Startup State

- Persist a runtime-incarnation/start marker atomically before layout restore.
- Mark clean shutdown only after the normal close path has saved session state.
- Treat missing, stale, malformed, or previous-incarnation markers as unclean.
- An unclean start restores the workspace/pane/tab layout without automatically
  launching or resuming agent commands.
- Each suspended agent retains provider, hcom identity, native session ID,
  working directory, and safe launch metadata needed for later explicit resume.
- The UI and control surface must report why an agent is suspended and whether
  the state came from unclean restore, pressure gating, cancellation, or a
  user choice.
- A clean start may use the configured automatic-resume policy, but it still
  passes through the concurrency controller.

### FR5: Pressure-Aware Resume Controller

- Replace shell-prefixed sleeps with a host-owned queue and explicit maximum
  concurrency.
- Default to suspended/manual agents after an unclean start.
- Permit explicit resume-all only through the same queue and exact current
  runtime/surface identity checks.
- Sample multiple bounded signals: MemAvailable, swap activity, process/RSS
  growth, active launch count, and WSL I/O evidence. PSI may inform but may not
  independently block or release work.
- Pause admission on pressure, resume only after hysteresis, and never terminate
  already-running agents as part of admission control.
- Expose queue state and per-agent outcome through CLI/control APIs.
- Support cancellation of queued, not-yet-launched agents without affecting
  running sessions.

### FR6: Isolated Preview Resource Harness

- Use an isolated preview channel, socket, state directory, and session store.
- Disable agent auto-resume; use inert/plain terminal workloads for scale tests.
- Stage surface counts `1 -> 10 -> 30 -> 56`; each stage requires a stable
  sample before advancing.
- Capture source SHA, install ID, renderer selection, visible/hidden surface
  counts, process CPU, RSS/PSS/anonymous/dirty memory, swap delta, relevant WSL
  wait signals, tick counters, and log growth.
- Include visible, hidden-tab, hidden-workspace, minimized/unmapped, repeated
  switch, split/reparent, and close/reopen scenarios.
- Include clean shutdown, simulated unclean shutdown using only an approved
  preview harness, layout-only restart, controlled resume, pressure pause,
  hysteresis release, and queue cancellation.
- Preserve terminal process continuity and byte-correct screen/input behavior.

## Verification and Acceptance

### Code-Level Gates

- Test visibility-state transition logic before implementation.
- Test initial hidden creation, duplicate map/unmap, reparent/unrealize/realize,
  destruction, and stale callback handling.
- Test adaptive tick decisions and fallback wake behavior without relying on
  wall-clock sleeps.
- Test clean/unclean marker state transitions, malformed markers, incarnation
  mismatch, and atomic-write failure.
- Test resume queue ordering, concurrency, pressure pause, hysteresis, cancel,
  exact-target mismatch, and no termination of running agents.
- Test prospective log rollover boundaries, retained-name collisions, setup
  failures, and repeated-warning accounting using small temporary fixtures.
- Run formatting, Clippy, targeted crate tests, workspace tests, and the Xvfb
  smoke only in released low-load windows with one Cargo job.

### Preview Resource Gates

- An invisible surface receives `ghostty_surface_set_occlusion(surface, false)`
  and a visible surface receives `true`; redundant transitions are coalesced.
- Hidden/unmapped surface CPU is no more than 10% of the same-preview visible
  baseline and does not exceed 10% of one CPU core at idle.
- After warm-up, 56-surface RSS grows no faster than 2 MiB/min over a 15-minute
  hidden hold; PSS anonymous/dirty growth is reported separately.
- No swap growth occurs during the hold.
- Repeated identical renderer warnings cannot cause unbounded log growth.
- All terminal child processes survive hide/show and retain usable content,
  keyboard input, clipboard behavior, and exact surface identity.
- An unclean preview restart launches zero agents automatically and exposes all
  intended agents as suspended.
- Controlled resume never exceeds configured concurrency and pauses/restarts
  admission under the tested pressure/hysteresis inputs.

### Operational Gates

- No load work starts while replacement host predicates are red: block-device
  inflight activity, D-state task census, low MemAvailable, or swap growth.
- Build/test concurrency is one Cargo job and never overlaps another Limux or
  HCOM full suite.
- No new agent worker is required for implementation setup.
- Daily-driver install/restart/promotion requires exact-head review, passing
  preview evidence, rollback instructions, and explicit operator approval.

## Task Dependency Shape

Generate six high-priority tasks in this order:

1. Implement and unit-test GTK-to-Ghostty visibility/occlusion plus adaptive
   tick discipline and bounded instrumentation.
2. Add renderer-selection diagnostics and the preview backend matrix. Depend on
   task 1 instrumentation.
3. Implement prospective bounded host logging and warning aggregation. May run
   after task 1 but must not touch the incident log.
4. Implement clean/unclean runtime markers and layout-only suspended-agent
   restore. Depend on task 1 for safe large-layout restore.
5. Implement the pressure-aware explicit resume controller and control/CLI
   state. Depend on task 4.
6. Run the staged isolated-preview resource/crash matrix and produce the
   operator promotion/rollback packet. Depend on tasks 1 through 5.

Every generated task must include the relevant code paths, tests, resource
gate, no-daily-driver boundary, and an observable definition of done. Do not
mark any task complete from static analysis alone.
