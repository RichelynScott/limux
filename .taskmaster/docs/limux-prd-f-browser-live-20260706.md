# PRD-F: Live Browser Panes — Architecture Decision + Implementation

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:40 UTC
**Purpose:** Bring browser capability to the running GUI — the last open Phase-2
item of the original parity plan and the #1-priority candidate in the research
db (cmux-20260702-001) — with the architecture chosen deliberately for WSL2 and
a server-side domain allowlist + audit events from day one.

- **Priority:** P1 (Wave 1 — roadmap W1.2)
- **Dependencies:** PRD-E (method registry + fall-through). Phase F1 (spike)
  can start before PRD-E completes; Phase F2 (implementation) cannot.
- **Effort:** L — **Tier 3 / high-stakes**; F1 decision gate is mandatory
  before any F2 code.
- **Channel targeting:** preview channel only until PRD-C checklist covers browser basics

## Problem Statement

`limux-core` carries a full `browser.*` vocabulary (~40 methods: navigation,
snapshot, find/click/fill, cookies/storage, tabs/frames, console/errors,
addscript, state save/load) and the CLI exposes a complete `browser` suite —
but none of it works against the running GUI: `pane.create type=browser`
fail-closes at bridge parse time, and the parity plan's only remaining Phase-2
line is "Browser command bridge parity." Meanwhile cmux ships in-app browser
panes as a headline feature (split browser next to terminal, scriptable
a11y-tree API, auth import) and upstream's new Rust `mux` backend chose a
different architecture: CDP-driving an existing Chrome instead of embedding an
engine (cmux #7325). On WSL2 the embedded-engine tax is higher than on macOS:
WebKitGTK under WSLg runs software-GL, spawns noisy subprocesses (already
observed polluting the host log), and duplicates the browser the operator
already runs on Windows.

## Phase F1 — Architecture decision spike (timeboxed, decision doc required)

Evaluate exactly two candidates against WSL2-weighted criteria; produce
`docs/decisions/browser-pane-architecture-20260707.md` with a scored matrix,
prototype evidence for both, and a recommendation. **Gate: operator (or
delegated lifo) sign-off on the decision doc before F2 begins.**

| Candidate | Sketch |
|---|---|
| (a) WebKitGTK embedded pane | WebKitGTK widget as a pane surface type inside the GTK app; `browser.*` methods drive it in-process. Existing `scripts/appimage-webkit.sh` implies prior packaging intent. |
| (b) CDP external browser | Limux launches/attaches to a Chromium-family browser via CDP (DevTools protocol over localhost); the pane renders a live view (embedded CEF-free: screencast frames into a GTK widget) or — v1-minimal — manages the external window while Limux owns control/automation. mux precedent: cmux #7325. |

Decision criteria (weights in the doc): WSLg rendering stability (software GL,
compositor popups) · memory footprint at 3+ browser panes · auth/profile reuse
(operator's real browser state) · `browser.*` vocabulary coverage achievable ·
subprocess/log hygiene · packaging weight (AppImage/AUR/RPM) · maintenance
surface. Prototype evidence minimum: render a page + execute `browser.goto`,
`browser.snapshot`, `browser.click` end-to-end under WSLg for each candidate
(throwaway code, committed to a spike branch, not merged).

## Phase F2 — Implementation (common invariants, architecture-specific detail)

The stories below bind REGARDLESS of the F1 choice; the executing agents write
architecture-specific task decomposition into TaskMaster at import time, after
the decision doc is signed.

### US-1: As the operator, I can open a browser pane next to my agent's terminal
- [ ] `limux new-pane --type browser --url <https-url>` works against the
      running GUI (bridge no longer fail-closes; parse-gate removed only once
      the route exists).
- [ ] `limux open-browser <url>` / `browser open-split` route live.
- [ ] The pane participates in normal pane life: splits, focus, close,
      attention marking, session-restore placeholder (URL restored; page state
      restore is out of scope v1).
- [ ] Browser pane creation/teardown does not disturb terminal panes (Xvfb
      regression: create/close browser pane while agent TUI streams output).

### US-2: As an agent, the scriptable browser vocabulary works live
- [ ] Wave-1 method set live-routed (registry class `gtk-mutation` or
      `browser-native` per PRD-E registry): `browser.goto/url/wait`,
      `browser.snapshot` (a11y-tree/text), `browser.screenshot`,
      `browser.find.*`, `browser.click`, `browser.fill`, `browser.get.*`,
      `browser.console.list/errors.list`, `browser.tab.list/new/switch/close`.
- [ ] Remaining `browser.*` methods explicitly classified `deferred` in the
      registry (documented, `-32601`).
- [ ] Every live method has an Xvfb (or headless-CDP) test against a local
      fixture page served from the test harness — no external-network tests.

### US-3: As the security posture owner, browsing is allowlisted + audited
- [ ] Server-side domain allowlist enforced in the host (NOT client-side):
      default-deny for `browser.goto`/`tab.new`/redirect-follow outside the
      allowlist; config file (`~/.config/limux/browser-allowlist.json`) with
      explicit `["*"]` opt-out documented as unsafe.
- [ ] Navigation denials return a structured error (`-32009` conflict family)
      naming the blocked domain.
- [ ] Audit events: every navigation, click, fill, script-injection, and
      cookie/storage access emits a structured line to a dedicated audit log
      (`~/.local/state/limux/logs/browser-audit.log`), including requesting
      socket peer + method + target; log rotation by size.
- [ ] `browser.addscript`/`addinitscript`/`addstyle` and `browser.cookies/
      storage` mutations classified `deferred` in v1 UNLESS the F1 decision
      doc argues them in with a threat note (default: deferred — highest
      injection risk, lowest agent need).
- [ ] Redirect/JS-initiated navigation is subject to the same allowlist
      check as explicit `goto` (test with a fixture meta-refresh page).

## Functional Requirements

1. F1 decision doc + sign-off BEFORE F2 (hard gate; TaskMaster subtask).
2. Registry integration: all `browser.*` classifications land in PRD-E's
   registry — one table, no side registry.
3. Subprocess/log hygiene: browser-engine output (either candidate) is
   captured into its own log stream, never interleaved raw into
   `limux-host.log` (closes the observed June-24 Chromium-noise pollution).
4. Packaging: F2 must state its dependency posture (WebKitGTK package dep vs
   "operator provides Chromium/Chrome path via `limux.browserExecutable` /
   `LIMUX_BROWSER_BIN`" — mirroring the Cursor lane's absolute-executable-path
   convention) and update the install docs accordingly.
5. Resource limits: cap concurrent browser panes (config, default 4) with a
   clean error at the cap.

## Non-Goals

- No cookie/profile IMPORT from other browsers (cmux has it; privacy surface
  too large for v1 — CDP candidate gets profile reuse "for free" via the
  operator's own browser, which is the acceptable v1 shape).
- No page-state session restore (URL-only restore in v1).
- No DevTools UI embedding, focus mode, tab mute, per-pane process pools.
- No remote-workspace browser routing (W3 SSH lane).
- No `browser.input_touch` (core already returns `not_supported`).

## Technical Considerations

- WSLg reality check for candidate (a): WebKitGTK's GL path under Zink
  software rendering is the highest-risk unknown — the F1 spike MUST test
  scrolling + video-less page render performance, not just "it opens".
- Candidate (b) rendering options differ in effort by an order of magnitude:
  managed-external-window (cheap, weaker "pane" integration) vs CDP
  screencast-into-widget (true pane, more work). The decision doc must score
  BOTH sub-variants; a v1 managed-window + v2 screencast ladder is acceptable.
- Keep `browser.*` handler logic OUT of `window.rs` (own module, e.g.
  `rust/limux-host-linux/src/browser_pane.rs` + pure-logic crate module for
  allowlist/audit — unit-testable headless).
- The typed-PTY control-character guard is terminal-scoped; browser `fill`
  needs its own input sanitation note (no control chars into CDP/WebKit text
  entry; test with fixture).

## Success Metrics

- Operator can run an agent + docs page side-by-side in one workspace on the
  real machine (PRD-C checklist addendum).
- Zero raw browser-engine lines in `limux-host.log` after a 10-minute
  browser-pane session.
- 100% of allowlist-denied navigations visible in the audit log during tests.

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-host-linux browser_pane -- --nocapture
cargo test -p limux-core browser_allowlist -- --nocapture   # if allowlist logic lands core-side
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh      # extended: browser fixture-page suite
```

## Rollback Plan

F2 ships behind the registry: reclassifying `browser.*` to `deferred` (one
table change) restores today's fail-closed behavior; `pane.create` browser
parse-gate can be re-enabled independently. Decision-doc + spike branches are
docs-only artifacts.

## Open Questions

1. Allowlist default contents — propose: empty (deny-all) with the checklist
   walking the operator through adding their doc domains. Confirm at F1 gate.
2. Should audit events ALSO surface as `notification.create` for high-risk
   actions (script injection)? Default: log-only v1.
3. If F1 chooses CDP: attach to the operator's real Windows Chrome over the
   WSL2 boundary, or a Linux-side Chromium? (Spike must test both; Windows-
   side attach has localhost-forwarding + version-skew risks.)
