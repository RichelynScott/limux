# Attribution Correction for Limux Commit `55e1f99`

- **Date:** 2026-07-30
- **Correct session:** `limu`
- **Correct role:** `LIMUX_CODEX_MGR`
- **Correct agent:** `codex-gpt-5.6-sol-high`
- **Affected commit:** `55e1f99927584f899f55b9b1d486acf71f3ddac5`
- **Incorrect trailer:** `Session-ID: FIRE`
- **Decision content:** unchanged — **A / GO, corrected scope**
- **Originating hcom incident:** `605357`

## Correction

Commit `55e1f99` was authored by the hcom session `limu`, acting as
`LIMUX_CODEX_MGR`. Fire did not author, approve, or own the OMP ask-waiting
decision.

PAPA_GIT policy forbids retroactively amending historical attribution trailers,
so the pushed commit remains byte-for-byte unchanged. This forward record is
the durable correction for its `Session-ID` field.

## Verified Cause

At the time of the commit:

- `PAPA_GIT_SESSION_ID` was unset.
- `PAPA_GIT_AGENT` correctly identified `codex-gpt-5.6-sol-high`.
- Limux's installed `.git/hooks/prepare-commit-msg` still accepted the shared
  `$HOME/.claude-session-name` fallback.
- That shared file contained `FIRE`.
- The installed Limux hook SHA-256 was
  `b5f360cf5836901567266e2f4f78ac7eb5fec7dc48a2d85b44b9bc8ec055e60e`.

The canonical PAPA_GIT source has already rejected this unsafe fallback:

- source fix: `b8d32f7` (`fix(attribution): reject shared global identity fallback`)
- current merged source head observed: `f47a7b4`
- canonical hook SHA-256:
  `859adf29b3bee1654aeb2b015153318268f6585728b4dc80e1f8b0da1b4d9b39`

The immediate Limux defect is therefore deployment drift: its installed hook
does not match the corrected canonical source.

## Scope

Do not describe every papa-git-hooked repository as affected without an
inventory. A repository is exposed when its installed hook still accepts the
shared global fallback and the committing process lacks a stronger explicit or
per-directory identity source.

Likewise, `Session-ID: FIRE` paired with a `codex-*` agent is decisive for this
incident because live session ownership proves Fire is a Claude session and
limu made the commit. A general zero-false-positive detector cannot infer
runtime solely from an arbitrary `Session-ID` string; it must compare against
an authoritative session/role registry or an explicitly runtime-tagged
identity namespace.

## Required Follow-Up

1. **PAPA_GIT owner:** inventory installed hook hashes across managed repos,
   reinstall the corrected hook in owner-approved lanes, and add an audit
   result that distinguishes canonical, stale, and unknown installed copies.
2. **Codex global-config owner:** repair the Limux/Codex bootstrap path so a
   live hcom Codex session supplies `PAPA_GIT_SESSION_ID` rather than reaching
   any fallback.
3. **Claude global-config owner:** verify the writer lifecycle for
   `$HOME/.claude-session-name`; treat the file as diagnostic-only and do not
   restore it as commit authority.
4. **Limux:** until its installed hook is reconciled, every Limux commit must
   supply explicit current-session PAPA_GIT identity or be refused.

No implementation history, decision content, or source authorship other than
the attribution field is changed by this correction.
