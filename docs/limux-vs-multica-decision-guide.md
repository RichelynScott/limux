# Limux vs Multica Decision Guide

Date: 2026-05-29

Last updated: 2026-05-29 22:31 EDT

Authoring runtime/session: Codex / tipi

Scope: `/home/riche/MCPs/limux`

Purpose: make the Limux vs Multica decision easier to review, choose from, and send back as a clear next-step instruction.

Updated with the `$html-decision-packet` pattern: explicit packet metadata, roadmap lanes, completion/status accounting, execution-mode decision, skill checklist, skill-ranking toggle, notes, copy-back response, and a bottom Sources / Evidence section.

## Roadmap Lanes

| Lane | Status | Meaning |
|---|---|---|
| Context | Done | Limux and Multica were compared through local source inspection, public repo/docs review, and subagent reports. |
| Decision | Done | User chose Limux + hcom primary, no Multica pilot yet, and Limux fixes first. |
| Execution | Current | `agent-team` no longer writes `AGENTS.md` by default; zero-friction protocol discovery, launch-snippet hardening, typed-PTY control-character guards, and Phase 5B automatic bootstrap are implemented. |
| Canonical template update | Done | Kazu updated the global decision-packet pattern/template/skill with a bottom Sources / Evidence section when applicable. |
| Closeout | Current | Handoff docs now capture the morning resume path. |

## Completion Status

| Category | Status |
|---|---|
| Done | Created the readable Markdown guide and dark-mode HTML decision sheet; user filled it out and selected the Limux-first path. |
| Done | Commit `cec067f` moved `agent-team` protocol output to `LIMUX_AGENTS.md` by default and added `--protocol-path`. |
| Done | Ran a five-subagent brainstorm on near-zero-friction Limux agent-team workflow. |
| Done | Added generated `LIMUX_AGENTS.md` instruction-source discovery, generated marker, local sidecar policy guidance, no-overwrite protection, and Phase 5B post-launch bootstrap prompts. |
| Done | Added shared typed-PTY text validation for CLI, bridge/core, and host sink paths; terminal controls now use `send-key` instead of text injection. |
| Verified | `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, `git diff --check`, full `./scripts/check.sh`, and debug/release Xvfb smoke pass locally. |
| Gated | Multica adoption and install remain deferred. The next Limux work is roster/ledger and cross-team coordination polish, not Multica migration. |
| Local prerequisite | Fresh clones or cleaned worktrees still need the reviewed Ghostty/Zig build lane before host checks. |
| Explicitly not done | Did not clone, install, build, or execute Multica. Did not edit global Codex/Claude templates directly from this Limux session. |

## Short Answer

Do not replace Limux with Multica right now. The selected path is Limux + hcom
primary, with Multica deferred until after the Limux workflow fixes.

Recommended path:

1. Keep Limux as the live operator cockpit for visible Codex, Claude Code, and reviewer panes.
2. Keep hcom plus durable files for cross-project and cross-team messaging.
3. Defer the Multica pilot until after Limux fixes.
4. Continue improving Limux around project/team roster, durable review ledger, and cross-team routing support.

The subagents converged on the same conclusion: Multica is promising, but it is not a drop-in replacement for Limux's terminal-first workflow.

## Decisions Received

The user selected:

- Keep Limux + hcom primary.
- Do not pilot Multica now; revisit after Limux fixes.
- Use hcom for cross-project and cross-team communication.
- Use durable files for long instructions and send hcom pointers.
- Let the current session handle the next scoped step.
- Use a skill-ranking subagent before complex execution.
- Prioritize:
  - protect repo `AGENTS.md` files,
  - sidecar protocol support,
  - project/team roster,
  - durable review and consensus ledger.

## What Each Tool Is Best At

| Need | Limux | Multica | Practical winner |
|---|---|---|---|
| Live Codex and Claude Code sessions | Real terminal panes, visible shells, direct send/read/notify | Daemon-dispatched tasks with logs and comments | Limux |
| 4+ projects | Workspaces can map to projects, but roster/routing is manual | Workspaces, projects, issues, agents, and runs are native | Multica |
| One Codex plus one Claude per project | Straightforward as panes | Supported as runtimes, but Codex has important limitations | Limux for live work |
| Subagent teams | Can host them, but orchestration is convention | Squads can route work through a leader | Multica partially |
| Adversarial review and consensus | Not native; use skills, hcom, and files | Not native; model with issues/comments/status conventions | Neither native |
| Cross-team communication | Needs hcom or another bus | Good inside one workspace, weaker across isolated workspaces | hcom plus Multica if piloted |
| Security and ops burden | Smaller local Rust/GTK surface | Server, DB, Docker/Helm, daemon, installers, skills, auth | Limux |
| Fit for current workflow | Narrow but aligned | Broad but heavier and early | Hybrid |

## Recommendation

Choose the hybrid path:

```text
Limux = live operator cockpit
hcom + durable files = cross-project and cross-team messaging
Multica = optional async issue/task ledger pilot
```

Do not migrate the main workflow to Multica until a pilot proves that Codex, Claude Code, subagents, adversarial review, and cross-project routing all work under real pressure.

## Why Not Switch Wholesale

### 1. Multica is async-first, not terminal-first

Limux gives the operator visible panes and direct intervention. That matters for your workflow because you often need to watch Codex and Claude Code work together, inject corrections, read a terminal, and coordinate live reviewers.

Multica turns work into issues, comments, agent runs, and daemon-dispatched tasks. That is useful, but it changes the operating model.

### 2. Codex support has sharp edges in Multica

The adversarial subagent flagged two serious blockers:

- Codex subagents are disabled by default because Multica can lose child-agent output when the parent task completes.
- Codex session resumption is documented as unusable, while Claude and several other agents have stronger support.

That directly conflicts with your normal model of at least one Codex session per project, often with subagents.

### 3. Review and consensus are not built-in

Neither Limux nor Multica has native adversarial-review consensus gates.

You can implement this with conventions:

- dedicated reviewer agents
- required status comments
- issue labels or metadata
- hcom messages
- durable decision files
- GitHub PR gates

But Multica does not remove the need to design the process.

### 4. Multica has a much larger supply-chain and operations surface

Multica adds:

- Go backend
- Next.js frontend
- PostgreSQL 17 plus pgvector
- Docker or Helm deployment
- optional Redis/rate limiting concerns
- CLI and daemon lifecycle
- desktop/mobile clients
- installers
- third-party skills
- auth/email/backup/upgrade concerns

Limux is narrower and easier to reason about as a local terminal workspace manager.

## Important Multica Risks To Resolve Before Adoption

| Risk | Why it matters |
|---|---|
| `curl | bash` installer path | Do not run unverified installers on production machines. |
| Mutable Docker tags | Pin release tags or image digests. |
| Auto-update behavior | Disable or control updates for serious workflows. |
| Unsigned desktop release concerns | Avoid desktop installer trust until reviewed. |
| Claude `bypassPermissions` and Codex sandbox behavior | Needs explicit permission policy review. |
| Local-directory execution | Agents can mutate real repos in place. |
| Plaintext custom env in DB | Do not put real secrets into agent env fields. |
| License is not plain Apache-2.0 | Needs business/legal review before external service use. |

## Limux Risks To Fix

| Risk | Recommended action |
|---|---|
| `agent-team` previously clobbered `AGENTS.md` | Shipped in `cec067f`: default output is now `LIMUX_AGENTS.md`; later Phase 5A added generated-marker, instruction sources, and no-overwrite semantics. |
| Agent-team peers previously needed manual orientation | Shipped in Phase 5B: peer panes launch with bare agent commands, then receive a sanitized post-write bootstrap prompt. |
| Browser bridge parity is unfinished | Keep browser automation separate until parity is implemented. |
| `read-screen` is viewport-oriented | Do not rely on it for full long-running transcript capture. |
| No durable consensus ledger | Add a repo-side review ledger or integrate with hcom files. |
| No project/team roster | Add a simple project roster mapping workspaces, sessions, owners, and related teams. |

## Post-Decision Subagent Brainstorm

Five native subagents brainstormed how to make Limux + hcom feel near-zero
friction. The consensus was:

- Do not silently make `LIMUX_AGENTS.md` inherit from, copy, or merge
  `AGENTS.md`.
- Keep repo instruction files authoritative.
- Use `LIMUX_AGENTS.md` as a generated runtime protocol sidecar.
- Add an `Instruction Sources` section pointing agents to `AGENTS.md`,
  `CLAUDE.md`, `GEMINI.md`, and similar files directly.
- Add generated-marker and no-overwrite semantics before relying on automatic
  regeneration.
- Launch agent binaries first, then send a small bootstrap prompt after pane
  readiness and after the protocol file exists.

Implemented next implementation: Phase 5B two-phase bootstrap in
[`cmux-parity-plan.md`](cmux-parity-plan.md).

Recommended next implementation: project/team roster plus durable review and
consensus ledger, so four-project workflows can track workspaces, sessions,
owners, related teams, reviewer findings, and open decisions without relying on
terminal scrollback.

## HCOM Updates From The Thread

These hcom improvements shipped during the conversation and are relevant to your workflow:

| Update | Why it helps |
|---|---|
| `hcom r` and `hcom f` picker | Easier resume/fork selection with session name, tool, status, cwd, age, tag, and resumable state. |
| `hcom r --yolo` picker support | Flags forward through the picker. |
| Non-TTY roster output | Agents get a text list instead of an interactive menu. |
| Ack-on-inform allowed | Receipt can be recorded for inform messages. |
| `HCOM_NAME` sticky identity for send | Less repeated `--name` typing. |
| `hcom list --compact` | Denser one-line roster. |
| `hcom list --no-inbox` | Reduces pending-message noise. |
| Collision warning improvements | Fewer false positives and better partner state/repo context. |
| Update notice rate limit | Less update spam. |

These do not replace Limux or Multica, but they strengthen the cross-session communication layer.

## Decision Options

### Option A: Recommended

Keep Limux plus hcom as the primary workflow. Pilot Multica in a fenced repo.

Use this if you want the least disruption while still testing Multica's useful ideas.

Risk/tradeoff: keeps the current live workflow stable, but Multica's async-task value is only proven after a contained pilot.

### Option B

Use Multica only as an async issue/task board, while keeping Limux for live sessions.

Use this if you want issue comments, run history, squads, and assigned agent work, but do not want to risk terminal workflow replacement.

Risk/tradeoff: adds operational complexity while preserving Limux; useful only if the issue/run ledger reduces coordination overhead.

### Option C

Switch fully to Multica.

Not recommended yet. Only consider after a pilot proves Codex subagents, Codex resume, review/consensus, security posture, and local-directory safety.

Risk/tradeoff: highest migration risk because it replaces a live terminal cockpit with an async server/daemon workflow.

### Option D

Stay Limux-only and copy Multica ideas into Limux/hcom.

Use this if you want to avoid Multica ops burden entirely, but still want squads, run ledgers, and review metadata.

Risk/tradeoff: lowest external-tool risk, but requires local Limux/hcom feature work to capture Multica-like benefits.

## Execution-Mode Decision

Choose how the next step should be handled:

| Mode | Use when |
|---|---|
| Current session | Best for editing this guide, making small Limux docs changes, or preparing a concrete next-step plan. |
| Native subagents | Best for parallel read-only review, focused repo inspection, or independent implementation slices. |
| Buffered subagents | Best for broad multi-wave implementation or PRD execution work. |
| hcom-managed worker | Best only when a separate persistent runtime or cross-tool owner is required. |
| Defer/document only | Best if you want no execution until you make the strategic decision. |

## Skill Checklist For Next Step

Recommended skills:

- `$html-decision-packet` for dark-mode decision packets with copy-back payloads.
- `$evaluate-repo` for external repo adoption review.
- `$agent-orchestration` if subagents are used.
- `$hcom` for cross-session routing and direct Kazu coordination.
- `$methodical-modification-protocol` before changing Limux behavior.
- `$adversarial-assessment` or `$adversarial-review` before adopting Multica or changing global templates.
- `$browser-automation` if the HTML needs visual QA in a browser.

Optional toggle:

- Spin up a dedicated skill-ranking subagent before complex execution if the next step grows beyond a small local edit.

## Suggested Pilot Plan

If you choose to pilot Multica:

1. Use a disposable VM or non-sensitive repo.
2. Do not use `curl | bash`.
3. Pin Multica release `v0.3.11` or a specific commit/image digest.
4. Verify checksums and avoid unsigned desktop installers.
5. Disable or control auto-update behavior.
6. Use one workspace with multiple projects if cross-team communication is important.
7. Create one Codex runtime and one Claude Code runtime per project.
8. Add a squad leader only for routing tests.
9. Run one implementation task, one adversarial review task, and one cross-project relevance notification.
10. Track whether Multica saved operator time or added overhead.

## Morning Resume Instruction

Use this to continue:

```text
Resume from HANDOFF.md. Phase 5A protocol discovery, generated launch-snippet
hardening, typed-PTY control-character guards, and Phase 5B automatic bootstrap
are implemented and verified. Next implement the project/team roster plus
durable review and consensus ledger.
```

## Original Decisions Template

This was the copy-back template used to capture the now-selected path:

```text
My decisions after reviewing the Limux vs Multica guide:

1. Main path:
   - [ ] Keep Limux + hcom primary, pilot Multica only.
   - [ ] Use Multica as async layer only, not replacement.
   - [ ] Stay Limux-only and copy useful Multica ideas.
   - [ ] Switch fully to Multica. I understand this is not recommended yet.

2. Multica pilot:
   - [ ] Yes, run a fenced pilot.
   - [ ] Not now.
   - [ ] Decide later after Limux fixes.

3. Multica install/security posture if piloted:
   - [ ] Disposable VM or throwaway repo only.
   - [ ] No curl-pipe-shell.
   - [ ] Pin release/commit/image digest.
   - [ ] Verify checksums.
   - [ ] No real secrets.
   - [ ] Disable/control auto-update.

4. Limux priorities:
   - [x] Fix agent-team AGENTS.md clobber risk first.
   - [x] Add sidecar protocol file support.
   - [x] Add LIMUX_AGENTS.md instruction-source discovery.
   - [x] Add generated-marker and no-overwrite semantics for LIMUX_AGENTS.md.
   - [x] Add Phase 5B post-launch bootstrap prompts.
   - [ ] Add project/team roster.
   - [ ] Add durable review/consensus ledger.
   - [ ] Improve read-screen/transcript capture.

5. hcom posture:
   - [ ] Use hcom for cross-project communication.
   - [ ] Use hcom compact list/resume picker improvements.
   - [ ] Add relevance labels/topics when available.
   - [ ] Keep long instructions in files and send pointers.

6. Execution mode:
   - [ ] Current session.
   - [ ] Native subagents.
   - [ ] Buffered subagents.
   - [ ] hcom-managed worker.
   - [ ] Defer/document only.

7. Skills:
   - [ ] $html-decision-packet.
   - [ ] $evaluate-repo.
   - [ ] $agent-orchestration.
   - [ ] $hcom.
   - [ ] $methodical-modification-protocol.
   - [ ] $adversarial-assessment / $adversarial-review.
   - [ ] Other:

8. Skill-ranking subagent:
   - [ ] Yes, use one before complex execution.
   - [ ] No, not needed for the next step.

9. Notes:
   -
```

## Sources / Evidence

This section is intentionally included under the `$html-decision-packet` pattern. When sources are not applicable, future packets should explicitly say so.

Local Limux references:

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [docs/cmux-parity-plan.md](cmux-parity-plan.md)
- [HANDOFF.md](../HANDOFF.md)
- [FYI.md](../FYI.md)
- [rust/limux-cli/src/main.rs](../rust/limux-cli/src/main.rs)
- [rust/limux-host-linux/src/control_bridge.rs](../rust/limux-host-linux/src/control_bridge.rs)

Multica references:

- <https://github.com/multica-ai/multica>
- <https://github.com/multica-ai/multica/blob/main/CLI_AND_DAEMON.md>
- <https://github.com/multica-ai/multica/blob/main/SELF_HOSTING.md>
- <https://github.com/multica-ai/multica/blob/main/package.json>
- <https://github.com/multica-ai/multica/releases/tag/v0.3.11>
- <https://github.com/multica-ai/multica/blob/main/LICENSE>

Methodology:

- Used `$evaluate-repo` for adoption comparison.
- Used `$agent-orchestration` because subagents were explicitly requested.
- Used `$html-decision-packet` for this updated packet structure.
- Used subagents for Multica analysis, Limux analysis, and adversarial review.
- Sent a direct hcom request to `@kazu` for canonical pattern/template/skill updates because those global artifacts live under `/home/riche/Proj/CODEX_CLAUDE_CODE`.
- Stayed read-only for Multica: no clone, install, build, or execution.
