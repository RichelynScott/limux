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

(Codex-revised — corrected starting point) Limux ALREADY SHIPS an interactive
WebKitGTK browser pane as a **default cargo feature**
(`rust/limux-host-linux/Cargo.toml:13-14,22` — `default = ["webkit"]`,
`webkit6 = "0.6"`): `BrowserHandles` in `pane.rs:~2871` (webview, url_entry,
search bar, find controller, inspector/console handlers), GUI-wired via
`window.rs:5221 on_open_browser_here`, with WSLg GL mitigations already
applied (`configure_browser_settings` sets `HardwareAccelerationPolicy`,
pane.rs:~3195). `scripts/appimage-webkit.sh` bundles the WebKitWebProcess
runtime.

What does NOT work is the **scriptable/agent surface**: `limux-core` carries
an 84-method `browser.*` vocabulary and the CLI exposes a complete `browser`
suite, but the core handlers drive an **in-memory mock** (the CLI itself says
"linux mock", `limux-cli/src/main.rs:5457`) — not the shipped WebKit pane —
and `pane.create type=browser` fail-closes at bridge parse time
(control_bridge.rs:83-99, :413-428). So agents cannot open, drive, or read a
real browser pane; only a human can. The parity plan's remaining Phase-2 line
is "browser command bridge parity."

Meanwhile upstream's new Rust `mux` backend chose CDP-driving an existing
Chrome instead of an embedded engine (cmux #7325). On WSL2 the
embedded-engine tax is real: WebKitGTK under WSLg runs software-GL, spawns
noisy subprocesses (already observed polluting the host log), and duplicates
the browser the operator already runs on Windows — but it is also already
built and shipped, which the decision must weigh honestly.

## Phase F1 — Architecture decision spike (timeboxed, decision doc required)

Evaluate exactly two candidates against WSL2-weighted criteria; produce
`docs/decisions/browser-pane-architecture-20260707.md` with a scored matrix,
prototype evidence for both, and a recommendation. **Gate: operator (or
delegated lifo) sign-off on the decision doc before F2 begins.**

| Candidate | Sketch |
|---|---|
| (a) WebKitGTK embedded pane (SHIPPED today) | The existing default-feature WebKit pane becomes the engine behind `browser.*`. Evidence for (a) is **measured on the existing production pane**, not prototyped: memory at 3+ panes, subprocess/log hygiene, WSLg stability under the existing `HardwareAccelerationPolicy` mitigation. New work = engine bindings from `browser.*` methods to the live webview. |
| (b) CDP external browser | Limux launches/attaches to a Chromium-family browser via CDP (DevTools protocol over localhost); the pane renders a live view (screencast frames into a GTK widget) or — v1-minimal — manages the external window while Limux owns control/automation. mux precedent: cmux #7325. Prototype required (throwaway, spike branch, not merged). |

Decision criteria (weights in the doc): WSLg rendering stability — including
the WebKitGTK DMABUF/compositing crash class
(`WEBKIT_DISABLE_DMABUF_RENDERER` / `WEBKIT_DISABLE_COMPOSITING_MODE`) and
GDK backend choice (Wayland vs X11) · WSLg clipboard integration + IME/
keyboard input into the webview · memory footprint at 3+ browser panes ·
auth/profile reuse (operator's real browser state) · `browser.*` vocabulary
coverage achievable · subprocess/log hygiene · packaging weight
(AppImage/AUR/RPM; appimage-webkit.sh bundles WebKitWebProcess today) ·
maintenance surface · for (b)'s screencast sub-variant: measured FPS +
input-latency targets under software rendering · for (b)'s Windows-Chrome
attach: localhost forwarding across the WSL2 boundary + version-skew risk
(graded criteria, not open questions). Prototype evidence minimum for (b):
render a page + execute `browser.navigate`, `browser.snapshot`,
`browser.click` end-to-end under WSLg.

**Mandatory decision-doc section — existing-pane disposition:** if (b) wins,
the doc MUST specify what happens to the shipped WebKit pane (remove the
default feature / coexist as GUI-only / migrate) — silence on this is a
review-blocking gap.

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
- [ ] Wave-1 method set live-routed, using PRD-E's existing enum (registry
      class `gtk-mutation`; no new class): `browser.navigate` (exact core
      method name — `goto` is a CLI verb only, main.rs:5044) /
      `browser.url.get`, `browser.wait`, `browser.snapshot` (a11y-tree/text),
      `browser.screenshot`, `browser.find.*`, `browser.click`,
      `browser.fill`, `browser.get.*`, `browser.console.list`, `browser.errors.list`,
      `browser.tab.list/new/switch/close`. Registry classifications use
      EXACT core method names throughout.
- [ ] (Codex-required — binding invariant against PRD-E) **No `browser.*`
      method is EVER classified `fallthrough-read`**: the core `browser.*`
      handlers drive an in-memory MOCK (`handle_browser_extended_command`
      mutates counters/field-maps; the CLI calls it the "linux mock",
      main.rs:5457) — fall-through would return fabricated data against the
      live GUI. Every live browser method is a NEW ENGINE BINDING
      (`gtk-mutation` class routing to the real webview/CDP), which is the
      actual shape of F2's effort.
- [ ] Remaining `browser.*` methods (84 total in core) explicitly classified
      `deferred` in the registry (documented, `-32601`).
- [ ] Every live method has an Xvfb (or headless-CDP) test against a local
      fixture page served from the test harness — no external-network tests.

### US-3: As the security posture owner, browsing is allowlisted + audited
- [ ] (Codex-revised — method-gating alone is bypassable) The allowlist is
      enforced at the ENGINE POLICY LAYER, not only on socket methods: for
      WebKit, the navigation-policy-decision (`decide-policy`) + new-window
      (`create`) signal layer; for CDP, Fetch/Page navigation interception.
      Coverage: top-level navigations, subframe/iframe navigations,
      `window.open`/new-window, server 3xx redirects, JS-initiated
      navigation, AND the GUI URL bar (the shipped pane's `url_entry` goes
      through the same check). Subresource loads (fetch/img/script from an
      allowlisted page to non-allowlisted hosts) are OUT of scope v1 —
      documented explicitly as a residual.
- [ ] Fixture tests per bypass vector: link-click, `window.open`, HTTP 302,
      iframe navigation, meta-refresh — each denied when off-allowlist.
- [ ] Allowlist matching rules (normative): exact-host match, plus explicit
      `*.example.com` wildcard form for subdomains; NO suffix matching
      (`evil-example.com` never matches `example.com`); https default —
      an http entry requires the explicit scheme in the config; non-default
      ports must be listed to match; IP literals match exactly; hosts are
      punycode-normalized (A-label) before comparison. Config file
      `~/.config/limux/browser-allowlist.json`; explicit `["*"]` opt-out
      documented as unsafe.
- [ ] Test-harness carve-out is explicit: the Xvfb suite injects a harness
      allowlist containing its `127.0.0.1:<port>` fixture server — tests
      never weaken the default config.
- [ ] Session-restore URL replay is itself a navigation and re-checks the
      (possibly changed) allowlist.
- [ ] Navigation denials return a structured error (`-32009` conflict family)
      naming the blocked domain; the pane-cap error (FR-5) uses the same
      family.
- [ ] Audit events: every navigation, click, fill, script-injection, and
      cookie/storage access emits a structured line to a dedicated audit log
      (`~/.local/state/limux/logs/browser-audit.log`, created `0600`),
      including requesting socket peer + method + target. (Codex-required —
      no secrets in the audit trail) `browser.fill` VALUES and any script
      bodies are NEVER logged — field/selector names and byte-lengths only.
      Rotation by size, retention 5 files.
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
