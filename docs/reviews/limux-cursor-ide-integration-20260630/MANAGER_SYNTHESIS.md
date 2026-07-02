# Manager Synthesis: Cursor IDE Integration Draft

Date: 2026-06-30
Author/runtime: lifo / Codex
Status: draft revised after native, GLM, and direct MiniMax review; Kimi broad
review lane parked pending Hermes post-response core-dump isolation

## What Was Completed

- Saved the draft plan at
  `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`.
- Saved the Hermes reviewer brief at
  `docs/reviews/limux-cursor-ide-integration-20260630/REVIEW_BRIEF.md`.
- Updated the earlier options note with a pointer to the new draft.
- Checked TaskMaster state and confirmed this repo does not have a live
  `.taskmaster/config.json`, `.taskmaster/state.json`, or task store.
- Ran the operator-approved temporary `.env` fallback for only
  `OLLAMA_API_KEY`, `MINIMAX_API_KEY`, and `FIRECRAWL_API_KEY`, without
  printing or documenting values.
- Completed four substantive GLM reviews and four direct MiniMax reviews under
  this directory.

## Planned Reviewer Matrix

The intended buffered wave is 11 reviewers:

| Lane | Count | Model Target | Lenses |
|---|---:|---|---|
| GLM | 4 | Ollama Cloud GLM 5.2 | architecture, Cursor API feasibility, Limux protocol fit, workflow acceptance |
| MiniMax | 4 | MiniMax M3 | security, WSL/runtime failure modes, test strategy, sequencing |
| Kimi | 3 | Ollama Cloud Kimi K2.7 Code | v2 attach feasibility, no-dependency extension correctness, source/reference accuracy |

The wave should write one file per reviewer under this directory, then a compact
synthesis should update the plan.

## Historical Blocker / Hermes Hotfix

Hermes CLI model smokes failed before review jobs were launched:

- `ollama-cloud` GLM/Kimi routes return HTTP 404.
- OpenRouter MiniMax is unavailable because no `OPENROUTER_API_KEY` is
  configured.
- Direct MiniMax is unavailable because no MiniMax API key is configured.
- Direct Z.ai/Kimi providers return authentication failures.
- A custom Ollama Cloud bridge using the existing Ollama key still produced a
  missing-authentication-header error.

See `HERMES_MODEL_SMOKE_STATUS.txt` for the evidence summary.

Follow-up hcom Hermes worker attempts confirmed the same gate:

- `limux-cursor-glm-review-pita`: `ollama-cloud` / `glm-5.2` failed with HTTP
  404 while targeting `https://chatgpt.com/backend-api/codex`.
- `limux-cursor-kimi-review-zomu`: `ollama-cloud` /
  `kimi-k2.7-code:cloud` failed with HTTP 404 while targeting
  `https://chatgpt.com/backend-api/codex`.
- `limux-cursor-minimax-review-nova`: `minimax` / `minimax-m3` failed with HTTP
  401 and a missing `X-Api-Key` requirement.

The hcom launch path can pass `--model` and `--provider`; the blocker is still
Hermes provider resolution / credentials, not hcom delivery.

Rumi identified the likely Hermes-side root cause for GLM/Kimi Ollama Cloud
failures: stale `model.base_url` routing in `~/.hermes/config.yaml`. The live
runtime has `provider=ollama-cloud` and `default=glm-5.2`, but
`model.base_url` still points at `https://chatgpt.com/backend-api/codex`, so
explicit Ollama Cloud model requests hit the wrong backend. Rumi also verified
Ollama Cloud itself is healthy and that the short model IDs are available there:

- `glm-5.2`
- `kimi-k2.7-code`
- `minimax-m3`

Routing correction from Rumi/BigBoss: do not use `minimax-m3` through Ollama
Cloud for this review wave. MiniMax M3 must use the direct MiniMax provider
because the paid MiniMax token plan has the available capacity. Rumi is adding a
Hermes-side fail-closed guard so `ollama-cloud + minimax-m3` is not allowed.

Expected hcom launch shape after Hermes config/source remediation:

```bash
hcom hermes --dir /home/riche/MCPs/limux --terminal tmux --model glm-5.2 --provider ollama-cloud --hcom-prompt <brief>
hcom hermes --dir /home/riche/MCPs/limux --terminal tmux --model kimi-k2.7-code --provider ollama-cloud --hcom-prompt <brief>
hcom hermes --dir /home/riche/MCPs/limux --terminal tmux --model MiniMax-M3 --provider minimax --hcom-prompt <brief>
```

Alternate direct MiniMax route if needed:

```bash
hcom hermes --dir /home/riche/MCPs/limux --terminal tmux --model minimax-m2.7-highspeed --provider minimax --hcom-prompt <brief>
```

Do not mutate Hermes runtime config or source from the Limux lane without an
operator-approved Hermes-side fix. Rumi has acked ownership in
`/home/riche/Proj/hermes-agent` and will report back with source/runtime status
or an exact operator-approved config change if runtime mutation is needed.

Rumi later reported the Hermes runtime/source hotfix is active:

- Runtime files copied into `/home/riche/.hermes/hermes-agent`:
  `hermes_cli/runtime_provider.py` and
  `hermes_cli/cli_agent_setup_mixin.py`.
- Source commit: `14b50cd16` on branch
  `fix/provider-ollama-cloud-routing-20260630`.
- Source PR: `https://github.com/RichelynScott/hermes-agent/pull/52`.
- Codex review on PR #52 was requested and pending as of
  2026-06-30 09:50 EST.
- `~/.hermes/config.yaml` and secrets were not mutated.
- Installed resolver verification now shows stale `model.base_url` no longer
  overrides `ollama-cloud`; resolver returns `https://ollama.com/v1`.
- `ollama-cloud + minimax-m3` now fails closed before API use.
- Live GLM completion was intentionally not run by Rumi to avoid Ollama Cloud
  quota spend.

Limux-side post-hotfix smokes:

- `ollama-cloud + minimax-m3` now fails closed with the expected routing error,
  before API use.
- Direct MiniMax `--provider minimax --model MiniMax-M3` still returns HTTP 401
  / missing `X-Api-Key` from this shell, so the direct MiniMax credential/SCRIM
  path remains the blocker for MiniMax reviewers.
- GLM/Kimi live completion smokes were intentionally not run from Limux to avoid
  spending Ollama Cloud quota before final reviewer commands are ready.

SCRIM later ruled that direct MiniMax can proceed only through an exact reviewed
wrapper or exact command with an L0 exact grant. The grant should inject only:

```text
MINIMAX_API_KEY=MINIMAX_API_KEY
```

The reviewed wrapper surface for MiniMax reviewers is:

```text
docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
```

Grant details are recorded in:

```text
docs/reviews/limux-cursor-ide-integration-20260630/MINIMAX_SCRIM_WRAPPER.md
```

If direct MiniMax still returns HTTP 401 / missing `X-Api-Key` under that grant,
the remaining issue is Hermes adapter/header/env-name handling, invalid
credential, endpoint, or entitlement, not Limux planning.

Rumi then confirmed the direct MiniMax command shape is correct and the current
failure is credential delivery. Verified without printing secrets:

- `/home/riche/.hermes/.env` has no `MINIMAX_API_KEY`.
- Current shell env has `MINIMAX_API_KEY` present but empty.
- Hermes `provider=minimax` expects a non-empty `MINIMAX_API_KEY` and routes to
  `https://api.minimax.io/anthropic`.

Moka/SCRIM could not live-confirm the vault contains `MINIMAX_API_KEY` without a
human unlock or existing capability. Treat the live secret as operator-gated
until SCRIM shows a masked `MINIMAX_API_KEY` row or an approved grant/profile
lists it. If absent, the operator needs to add `MINIMAX_API_KEY` to SCRIM for
provider `minimax` via SCRIM's secure hidden-prompt path, then approve the exact
grant/profile.

SCRIM owner-approved diagnostic smoke before reviewer launch:

```text
executable: /home/riche/.local/bin/hermes
args: chat --provider minimax --model MiniMax-M3 -Q --max-turns 1 -q "Reply exactly OK-MINIMAX"
cwd: /home/riche/MCPs/limux
env: MINIMAX_API_KEY=MINIMAX_API_KEY
network: allow
timeout: about 120 seconds
capture: redacted_stdout_stderr
```

Reviewer launches should use the reviewed wrapper with one positional slot arg
per exact grant rather than broad hcom/Hermes environment inheritance.

SCRIM owner wrote the canonical clean grant recipe at:

```text
/home/riche/Proj/SCRIM/coordination/operator-runbooks/LIMUX_HERMES_MINIMAX_SCRIM_GRANT_RECIPE_2026-06-30.md
```

That SCRIM-owned runbook is the authoritative operator path for checking masked
secret existence, adding `MINIMAX_API_KEY` through a hidden prompt if absent,
creating/approving the L0 exact smoke grant, and shaping reviewer grants.

## Operator Override / External Review Results

The operator then explicitly approved the interim credential fallback documented
in the SCRIM skill for these keys only:

```text
/home/riche/Proj/CODEX_CLAUDE_CODE/.env
```

Only key names and non-empty status were checked in command output; plaintext
values were not printed, hcom-sent, copied into docs, committed, or placed in
argv.

Smokes:

- `ollama-cloud` / `glm-5.2`: passed.
- direct `minimax` / `MiniMax-M3`: passed.
- `ollama-cloud` / `kimi-k2.7-code`: returned `OK-KIMI`, then Hermes dumped
  core during/after CLI cleanup and returned rc 134.
- `ollama-cloud` / `kimi-k2.7`: returned 404 and is not a valid fallback.

Rumi advised proceeding with GLM + direct MiniMax only and keeping Kimi out of
broad reviewer waves until the Hermes post-response crash is isolated.

Substantive external review artifacts:

```text
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-architecture.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-runtime.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-cursor-api-rerun.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-control-trust.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-security.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-runtime.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-tests.md
docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-sequencing.md
```

One GLM cursor-api attempt produced an empty response with exit 0 and is not
counted:

```text
docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-cursor-api.md
```

## Manager Findings Before External Review

1. The v1/v2 split is the right default. A Cursor tree plus read-only snapshots
   gives useful file-explorer integration without destabilizing terminal state.
2. V1 should keep arbitrary terminal text injection out of scope. That command
   would create a new trust boundary and should be reviewed separately.
3. The "Cursor inside WSL/Remote first" assumption needs to stay explicit.
   Windows-host Cursor directly reaching a WSL Unix socket is a separate
   transport problem.
4. The v2 `surface.attach` idea should not be mixed into v1. It needs its own
   protocol PRD because copy/paste, resize, alternate-screen TUIs, and PTY
   ownership are all high-risk.
5. TaskMaster needs bootstrap/repair before it can be the durable task store
   for this repo. The plan intentionally avoids inventing task IDs.

## Native Review Synthesis

Three native Codex read-only reviewers completed fallback adversarial review
before the operator-approved `.env` fallback allowed GLM and direct MiniMax to
run.

Verdict:

- No P0s.
- V1 direction is feasible.
- Plan must be reworked before implementation.
- V2 attach mode is `REWORK BEFORE PRD`.

High-confidence changes incorporated into the plan:

- V1 must use typed request builders and an allowlist, not raw Limux socket JSON
  passthrough from Cursor.
- V1 must explicitly forbid `surface.send_text`, `surface.send_key`, CLI `send`
  aliases, `pane.create.command`, and `workspace.create.command`.
- Socket resolution should mirror Limux source: explicit setting/env,
  `${XDG_RUNTIME_DIR}/limux/limux.sock`, then the existing `/tmp/limux.sock`
  compatibility fallback. Do not invent `/run/user/${uid}` as a separate
  fallback unless Limux's resolver adds it.
- Cursor must identify the target Limux runtime and warn/refuse on stale or
  ambiguous sockets.
- `workspace.list` needs `folder_path`, `openable_path`, and `path_source`
  metadata before "Open in Cursor" and "Open current Cursor folder in Limux" can
  be implemented cleanly.
- Read-only snapshots should be called visible viewport snapshots through
  `surface.read_text`; do not imply scrollback support.
- Cursor folder launch must accept only local `file:` workspace folders in the
  same Linux/WSL environment, canonicalize the path, require an existing
  directory, and use quick-pick for multi-root workspaces.
- V1 extension tests should use Node built-in `node --test` and avoid
  npm/npx/vsce/yo package-execution paths.
- V2 attach mode needs a separate protocol PRD/spike covering attach lifecycle,
  output stream framing, backpressure, reconnect, host shutdown, input taxonomy,
  resize authority, and multi-client behavior before implementation.

External GLM and direct MiniMax review is complete for this planning pass. Kimi
is parked as a Hermes diagnostic follow-up because the provider response
succeeds before the runtime crashes.

## External Review Synthesis

Verdict across substantive external reviewers:

- Overall: `PASS_WITH_CHANGES`.
- Multiple P0/P1 findings are real and now incorporated into the plan.
- The v1 direction remains viable, but only if the server-side trust boundary,
  socket/runtime identity, method registry, and executable acceptance gates are
  pinned before implementation.

Required changes incorporated into the plan:

- Extension request builders are not a security boundary. The current Limux
  socket is same-user broad in `LocalUser` mode, so Cursor state-changing
  commands require a server-side restricted method set, separate restricted
  socket, or caller role/nonce with server-enforced allowlist.
- The socket resolver must mirror Limux source: explicit setting/env, then
  `${XDG_RUNTIME_DIR}/limux/limux.sock`, then existing `/tmp/limux.sock`
  fallback. Do not invent `/run/user/${uid}` unless Limux resolver adds it.
- Ambient `/tmp/limux.sock` must be treated as risky; Cursor should identify
  the host before use and the host should later harden stale/symlinked socket
  handling.
- Runtime identity must include enough stable fields for stale/duplicate socket
  rejection, and state-changing calls must pin the selected runtime.
- Supported v1 topologies are native Linux Cursor and Windows Cursor with
  Remote-WSL extension host inside the distro. Native Windows Cursor direct to
  the WSL Unix socket is out of v1.
- Method names are pinned: `workspace.list`, `workspace.select`,
  `window.present`, `cursor.pane_create_empty`, `surface.read_text`, and
  `cursor.workspace_open_folder`.
- `cursor.pane_create_empty` must reject `command` and unknown payload fields
  server-side, not only in extension builders.
- `surface.read_text` is visible viewport only and must reject scrollback
  params until a separate protocol supports them.
- Limux-to-Cursor launch should require a configured absolute executable path
  such as `limux.cursorExecutable` or `LIMUX_CURSOR_BIN`; PATH lookup is not the
  default.
- Tests need executable acceptance coverage: Node `node --test`, actual
  framing-path rejection tests, Rust contract tests, fake/stale socket harness,
  `./scripts/check.sh`, and Xvfb smoke.

## Kimi Diagnostic Follow-Up

Do not run Kimi as part of broad reviewer waves until the Hermes post-response
core dump is isolated. If Kimi is required later, run a single bounded
diagnostic lane with crash/core/log capture:

```bash
hermes chat --provider ollama-cloud --model kimi-k2.7-code -Q --max-turns 1 -q "<diagnostic prompt>"
```

Do not use `kimi-k2.7`; that model ID returned 404. Do not substitute MiniMax
through Ollama Cloud; MiniMax M3 should use the direct MiniMax provider only.
