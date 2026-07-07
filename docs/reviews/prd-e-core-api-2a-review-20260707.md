# PRD-E Commit 2a Review: limux-core Public API

Reviewer: lifo
Date: 2026-07-07
Reviewed commit: `7bbd27210c64f37e00d6dc40025be7ba81394dbd`

## Scope Reviewed

- `Dispatcher::with_shared_state(Arc<Mutex<ControlState>>)`
- `Dispatcher::dispatch_sync(V2Request)`
- `ControlState::import_snapshot(ControlStateSnapshot)`
- Public snapshot DTOs and `ControlStateSnapshotError`

## Findings

No blocking findings.

The commit satisfies the PRD-E 2a gate without building on 2b behavior:

- Shared-state constructor uses an existing caller-owned `Arc<Mutex<ControlState>>` instead of allocating a new state wrapper.
- Sync dispatch reuses the existing `dispatch_request` path and the async `dispatch` method now delegates to it, preserving prior response behavior.
- Snapshot import is an explicit public API that lets GTK write a tree-shaped mirror without exposing private `ControlState` internals.
- Snapshot validation rejects empty trees and duplicate ids before mutation, avoiding ambiguous mirror reads.
- Import preserves build/runtime identity and derives next ids from imported state, so later core-created entities do not collide with mirror ids.

## Verification

```text
cargo fmt --check
cargo test -p limux-core --lib -- --nocapture
cargo clippy -p limux-core --all-targets -- -D warnings
git diff --check
```

All commands passed after commit 2a.

## Gate Verdict

Commit 2a is approved as the limux-core public API base for PRD-E.

Do not start commit 2b until this review artifact is committed and pushed with the PRD-E branch.
