# Upstream Feature Research Database

This directory tracks candidate features, fixes, and product ideas from
`manaflow-ai/cmux`, upstream `am-will/limux`, and this fork's local planning
docs. It is a planning database, not an implementation branch.

Snapshot date: 2026-07-02

## Files

- `items.json` is the canonical machine-readable seed database.
- `items.md` is the human review table for prioritization.
- `sources.md` records the GitHub queries and local sources used for this
  snapshot.

## Rules

- Treat cmux as an inspiration and prioritization feed, not as a source-copy
  target. cmux is Swift/AppKit/macOS; Limux is Rust/GTK/libadwaita/Linux.
- Direct copying of cmux source, assets, or implementation details requires a
  separate license and compatibility review.
- Treat upstream `am-will/limux` PRs as design and patch candidates, not
  automatic merge targets. This fork is substantially ahead and has divergent
  runtime/session work.
- Keep vendored `ghostty/` read-only from the Limux layer; use the C API.
- Convert accepted items into PRDs, TaskMaster tasks, or narrowly scoped
  implementation branches before coding.

## Item Schema

Each entry in `items.json` uses these fields:

- `id`: Stable local item id.
- `source_project`: `cmux`, `limux-upstream`, or `local-doc`.
- `source_type`: `release`, `pr`, `issue`, `branch`, `doc`, or `taskmaster`.
- `source_url`: Primary source URL or local path.
- `source_title`: Source title.
- `source_date`: Source creation, publication, or snapshot date.
- `theme`: Product/technical theme.
- `kind`: `feature`, `fix`, `ux`, `architecture`, `performance`, `security`,
  `packaging`, `test`, or `docs`.
- `status`: `candidate`, `prd-needed`, `task-created`, `accepted`,
  `rejected`, or `done`.
- `priority`: `high`, `medium`, or `low`.
- `fit`: `direct`, `translate`, `inspiration-only`, or `not-fit`.
- `value_score`, `risk_score`, `complexity_score`: 1 to 5.
- `limux_native_design_note`: How this should become Rust/GTK/Linux-native.
- `gtk_rust_implications`: Likely GTK/Rust code surfaces or constraints.
- `related_taskmaster_ids`: TaskMaster ids when known.
- `related_local_docs`: Local docs that seed or constrain the item.
- `acceptance_sketch`: First-pass acceptance criteria.
- `license_review_required`: Whether source-copy or asset-use review is likely.
- `notes`: Additional triage notes.

## Theme Taxonomy

- `bridge-parity`
- `browser`
- `runtime-isolation`
- `cursor-integration`
- `agent-orchestration`
- `review-workflow`
- `workspace`
- `pane`
- `notifications`
- `terminal`
- `rendering`
- `performance`
- `settings`
- `packaging`
- `security`

## Current Recommended PRD Queue

1. Browser bridge parity plus domain allowlist.
2. Scalable agent sidebar state and notification correctness.
3. Restore/session correctness pack: cwd inheritance, split autosave,
   recently closed surfaces, and Git optional-lock discipline.
4. Render sizing and fractional-scale correctness from upstream Limux PRs
   #83/#100.
5. IME/dead-key input correctness from upstream Limux PR #90.
