# Direct MiniMax SCRIM Wrapper

Date: 2026-06-30
Author/runtime: lifo / Codex
Status: reviewed command surface for SCRIM grant setup; not yet executed with a
SCRIM secret grant

Canonical SCRIM owner recipe:

```text
/home/riche/Proj/SCRIM/coordination/operator-runbooks/LIMUX_HERMES_MINIMAX_SCRIM_GRANT_RECIPE_2026-06-30.md
```

Use that SCRIM-owned runbook as the authoritative grant/approval recipe. This
file records the Limux-side wrapper and expectations.

## Purpose

The Limux Cursor integration review wave needs MiniMax M3 reviewers through the
direct MiniMax provider only. Rumi/BigBoss explicitly disallowed
`ollama-cloud + minimax-m3` for this wave, and Hermes now fail-closes that
route.

SCRIM ruled that direct MiniMax can be used only through an exact reviewed
wrapper or exact command. This directory-local wrapper provides that narrow
surface:

```text
docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
```

## Exact Grant Shape

Use one L0 exact SCRIM grant per review slot. Do not grant broad arbitrary
Hermes CLI access.

Required properties:

- Working directory: `/home/riche/MCPs/limux` or
  `/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630`.
- Executable:
  `/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh`.
- Slot selection: pass exactly one positional argv value, such as
  `minimax-security`; do not use `LIMUX_REVIEW_SLOT` or any non-secret
  environment inheritance for slot selection.
- Network: allow.
- Timeout: tight; suggested 900 to 1200 seconds per slot.
- Capture: `exit_only` because the wrapper writes artifacts. Use
  redacted stdout/stderr only for a separately reviewed diagnostic slot.
- Secret env mapping: `MINIMAX_API_KEY=MINIMAX_API_KEY`.
- No secrets in argv, config, `.env`, hcom, chat, shell history, reviewer
  prompts, or committed files.

Exact command shape per slot:

```bash
/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh minimax-security
/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh minimax-runtime
/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh minimax-tests
/home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh minimax-sequencing
```

Exact grant argument shape per slot:

```text
exec: /home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
arg: minimax-security
env mapping: MINIMAX_API_KEY=MINIMAX_API_KEY

exec: /home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
arg: minimax-runtime
env mapping: MINIMAX_API_KEY=MINIMAX_API_KEY

exec: /home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
arg: minimax-tests
env mapping: MINIMAX_API_KEY=MINIMAX_API_KEY

exec: /home/riche/MCPs/limux/docs/reviews/limux-cursor-ide-integration-20260630/run-minimax-direct-review.sh
arg: minimax-sequencing
env mapping: MINIMAX_API_KEY=MINIMAX_API_KEY
```

## Wrapper Contract

The wrapper:

- hardcodes `--provider minimax --model "MiniMax-M3"`;
- uses `#!/usr/bin/bash`;
- calls `/home/riche/.local/bin/hermes`, `/usr/bin/timeout`, and
  `/usr/bin/date` directly instead of relying on `PATH`;
- accepts only four non-secret positional slot values;
- requires `MINIMAX_API_KEY` in the child environment and does not print it;
- writes one artifact per slot in this review directory;
- refuses broad or arbitrary model/provider use.

If the direct route still reports HTTP 401 or missing `X-Api-Key` under the
SCRIM grant, classify it as a Hermes direct-MiniMax adapter/header/env-name
issue, invalid credential, endpoint issue, or entitlement issue. It is not a
Limux planning problem.

## Diagnostic Smoke Grant

Before launching review slots, run one exact diagnostic smoke if SCRIM confirms
or the operator adds `MINIMAX_API_KEY`.

SCRIM owner-approved smoke surface:

```text
executable: /home/riche/.local/bin/hermes
args: chat --provider minimax --model MiniMax-M3 -Q --max-turns 1 -q "Reply exactly OK-MINIMAX"
working directory: /home/riche/MCPs/limux
secret: MINIMAX_API_KEY
env mapping: MINIMAX_API_KEY=MINIMAX_API_KEY
network: allow
timeout: about 120 seconds
capture: redacted_stdout_stderr
```

Do not use hcom/Hermes broad env inheritance for this smoke. If this exact smoke
still reports HTTP 401 or missing `X-Api-Key`, route the result to Rumi as a
Hermes direct-MiniMax adapter/header/env-name, credential, endpoint, or
entitlement issue.

## Operator Gate

SCRIM owner could not live-confirm vault contents from agent context. Live
existence remains operator-gated until one of these is true:

- `scrim list` or an equivalent approved SCRIM view shows a masked
  `MINIMAX_API_KEY` row; or
- an approved SCRIM daemon profile/grant lists `MINIMAX_API_KEY`.

If absent, the operator action is to add `MINIMAX_API_KEY` with provider
`minimax` through SCRIM's hidden/secure prompt path, then approve the exact
grant or profile. Keep values out of hcom, env files, shell history, configs,
and prompts.
