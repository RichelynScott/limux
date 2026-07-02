# Hermes MiniMax Direct Review

Slot: minimax-tests
Model: MiniMax-M3
Provider: minimax
Generated: 2026-06-30 17:25:57 EST

```text

session_id: 20260630_172558_14dd1f
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P0] `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md:235-241` — Operator workflow acceptance is prose-only. None of the four bullets has an executable check. Fix: add a `docs/reviews/limux-cursor-ide-integration-20260630/v1-acceptance.sh` that drives each (tree matches `limux --json list-workspaces`, no `command` field reachable from Cursor, folder round-trip works, no `surface.send_text` registered). Reference it from the implementation PRD as the merge gate.

2. [P0] Plan:207-213 — Extension safety tests cover request builders only, not the wire layer. A future maintainer adding `socket.request(method, params)` bypasses every builder test. Fix: add a `node --test` case that feeds a synthetic JSON envelope carrying `surface.send_text`, `pane.create.command`, and an unknown method through the actual framing path and asserts rejection; mirror as a property test in `limux-control` deserialization.

3. [P0] Plan:7 — Implementation gate references "current Limux PR cleanup is merged" but never names the PRs or defines a checkable condition. Fix: list the PR numbers/tags and require `./scripts/check.sh` + `./scripts/xvfb-smoke-test.sh LIMUX_SMOKE_PROFILE=debug` clean as the gate. Make it `pre-merge.yml`-style prose, not narrative.

4. [P1] Plan:100-102, plan:237-239 — Wayland focus is hand-waved. Operator workflow #2 silently degrades. Fix: state a hard rule — `workspace.select { present: true }` ships only after a real Wayland desktop session confirms focus; the extension must surface a "Wayland-untested" banner when the host returns `present=false`. Add one Wayland run to the acceptance script.

5. [P1] Plan:105-107 — Read-only snapshot scope is undefined ("Do not imply scrollback" is a non-testable negative). Fix: pin the contract — v1 returns current visible viewport only, fixed grid size, no scrollback/count params. Add `node --test` enforcing view-size bounding and a Rust contract test that rejects `count`/`offset` keys on `surface.read_text` for the v1 method.

6. [P1] Plan:220-222 — Multi-runtime socket differentiation has no test harness. Two-live-socket-plus-stale synthesis is hard and is left as a manual step. Fix: design a `LIMUX_SOCKET_TEST_DIR` fake-socket harness where two `dlisten` sockets reply with distinct `system.identify` payloads; cover in `node --test` and promote one assertion into `./scripts/xvfb-smoke-test.sh`.

7. [P1] Plan:96-97 — `path_source` precedence rule says "safe and known" but never defines safe. Fix: enumerate rules (same UID, dir exists, canonicalized, no traversal-after-canonicalize) and add a Rust unit test per branch plus an extension-side test that the request builder refuses cwd-derived paths when safety fails closed.

8. [P2] Plan:198 — Cursor launcher argv tests do not enumerate rejection cases. Fix: list cases — non-existent dir, file-not-dir, broken symlink, symlink-escape, whitespace, shell metacharacters — one `node --test` per case. Reuse any existing `safe_argv` helper if present.

9. [P2] Plan:207-220 — No extension-host lifecycle coverage. `node --test` runs pure JS; `activate`, command registration, TreeDataProvider wiring are unexercised and `@vscode/test-electron` is forbidden by the no-deps rule. Fix: pick `@vscode/test-cli` or a headless Xvfb `cursor --extensionDevelopmentPath` run as the activation smoke and document the choice in the plan; do not leave it implicit.

10. [P2] Plan:224-233 — V2 attach matrix lists scenarios with no test harness. Fix: keep this section behind a "v2 protocol spike" PRD that first defines the harness (recorded PTY replays, fake alternate-screen driver, two-client arbitration). Do not let it bleed into v1.

Missing Evidence:
- No CI workflow file cited — confirm whether extension CI is in scope or whether `node --test` runs only locally.
- No specification of which `./scripts/xvfb-smoke-test.sh` assertions the new behaviors add. "Where practical" is non-actionable.
- No fixture for the WSL rule (plan:43-44). Define how the extension probes `process.platform`, `WSL_DISTRO_NAME`, and socket-path shape to refuse Windows-host Cursor against a WSL Limux socket, and assert it in `node --test`.
- No review of existing test infrastructure for Ghostty snapshot capture (`surface.read_text` is new; verify the producing side has a backing test, not just the consumer).
- V2 promotion criteria absent — what measured signal reopens the v2 lane.

Recommended Plan Changes:
- Replace the "Operator workflow acceptance" bullets with a new "Acceptance Gates" subsection citing three explicit scripts: `node --test` source-level, Rust integration tests against `limux-control` for the no-command contract, and an Xvfb-driven acceptance script that loads `integrations/cursor-limux` via `--extensionDevelopmentPath` and runs each workflow bullet.
- Replace the implementation-gate paragraph (line 7-8) with a numbered checklist naming required PRs and required `./scripts/check.sh`/`xvfb-smoke-test.sh` results.
- Add a "V2 Promotion Gate" subsection listing the measured signals (resize storm observed, slow-reader backpressure verified, alternate-screen replayed, two-client arbitration tested) that unblock the v2 PRD.
- Enumerate Wayland, two-socket, and WSL-detection cases in `node --test` rather than leaving them as prose.
- Specify which Xvfb assertions land and which stay manual so the gate does not drift.

```

Exit status: 0
