# Browser Pane Architecture Decision - 2026-07-07

Status: PROVISIONAL SKELETON - evidence pending
TaskMaster: `cmux-parity-20260707` task #6
PRD: `.taskmaster/docs/limux-prd-f-browser-live-20260706.md`

## Gate

No Phase F2 browser implementation may begin until this decision document has
measured evidence, a recommendation, and explicit operator or delegated-owner
ratification.

This file is intentionally a skeleton. It records the decision frame for the
morning review and leaves measurement cells empty. No F1 spike code has been
started from this document.

## Decision Question

Which browser-pane architecture should Limux use for live GUI browser control
on WSL2?

## Candidate Architectures

### Candidate A - WebKitGTK Embedded Pane

Use the existing compiled-by-default WebKitGTK pane as the live browser engine.
The pane is currently runtime-gated behind `LIMUX_ENABLE_WEBKIT_BROWSER`.

Known starting points:

- `rust/limux-host-linux/Cargo.toml` includes the `webkit` default feature.
- `pane.rs` contains `BrowserHandles`, WebKit settings, URL entry, find, and
  inspector/console plumbing.
- `window.rs` already has a human-facing open-browser path.
- `scripts/appimage-webkit.sh` bundles the WebKitWebProcess runtime.

Required evidence:

- Enable with `LIMUX_ENABLE_WEBKIT_BROWSER=1` in a preview harness.
- Measure memory at 3+ browser panes.
- Test WSLg rendering stability under Wayland and X11.
- Test DMABUF/compositing mitigations:
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` and
  `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
- Verify clipboard, IME, keyboard input, scrolling, and log hygiene.

### Candidate B1 - CDP External Browser, Managed Window

Launch or attach to a Chromium-family browser via Chrome DevTools Protocol
while Limux owns automation and lifecycle state. The browser remains an
external managed window for v1.

Required evidence:

- Demonstrate `browser.navigate`, `browser.snapshot`, and `browser.click`
  end-to-end against a local fixture page.
- Test Linux-side Chromium under WSL2.
- Test Windows-side Chrome over the WSL2 localhost boundary.
- Measure attach latency, version-skew risk, memory, and log hygiene.
- Validate operator profile/auth reuse when attaching to the operator browser.

### Candidate B2 - CDP External Browser, Screencast Into GTK Widget

Use CDP screencast frames as the visual pane content and forward input events
through CDP. This is closer to a true pane but higher effort than B1.

Required evidence:

- All Candidate B1 evidence.
- Measure screencast FPS under WSLg software rendering.
- Measure click/key input latency through the screencast path.
- Assess maintenance cost for frame handling and input translation.

## WSLg-Weighted Criteria

| Criterion | Weight | Candidate A Evidence | Candidate A Score | Candidate B1 Evidence | Candidate B1 Score | Candidate B2 Evidence | Candidate B2 Score |
|---|---:|---|---:|---|---:|---|---:|
| WSLg rendering stability | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Clipboard, IME, and keyboard input | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Memory footprint at 3+ panes | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Auth/profile reuse | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| `browser.*` vocabulary coverage | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Subprocess and log hygiene | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Packaging weight | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Maintenance surface | TBD | TBD | TBD | TBD | TBD | TBD | TBD |
| Screencast FPS and input latency | TBD | N/A | N/A | N/A | N/A | TBD | TBD |
| WSL2 localhost forwarding and version skew | TBD | N/A | N/A | TBD | TBD | TBD | TBD |
| Weighted total | 100 | TBD | TBD | TBD | TBD | TBD | TBD |

## Measurement Plan

### Candidate A Measurements

| Measurement | Command or Harness | Evidence Cell |
|---|---|---|
| Enable preview WebKit pane | `LIMUX_ENABLE_WEBKIT_BROWSER=1` preview run | TBD |
| Memory with 0 browser panes baseline | host RSS/PSS capture | TBD |
| Memory with 3+ browser panes | host + WebKitWebProcess RSS/PSS capture | TBD |
| Wayland stability | 10-minute fixture-page run under WSLg Wayland | TBD |
| X11 stability | 10-minute fixture-page run under WSLg X11 | TBD |
| DMABUF mitigation | run with `WEBKIT_DISABLE_DMABUF_RENDERER=1` | TBD |
| Compositing mitigation | run with `WEBKIT_DISABLE_COMPOSITING_MODE=1` | TBD |
| Clipboard and IME | fixture form copy/paste and typing test | TBD |
| Log hygiene | confirm WebKit output is isolated from `limux-host.log` | TBD |

### Candidate B Measurements

| Measurement | Command or Harness | Evidence Cell |
|---|---|---|
| Linux-side Chromium CDP attach | throwaway spike branch | TBD |
| Windows-side Chrome CDP attach | throwaway spike branch | TBD |
| `browser.navigate` fixture | CDP prototype | TBD |
| `browser.snapshot` fixture | CDP prototype | TBD |
| `browser.click` fixture | CDP prototype | TBD |
| Managed-window UX fit | manual WSLg run | TBD |
| Screencast FPS | B2 prototype only | TBD |
| Screencast input latency | B2 prototype only | TBD |
| Version-skew risk | CDP protocol/version notes | TBD |
| Log hygiene | Chromium stderr capture review | TBD |

## Browser Method Routing Constraints

- No `browser.*` method may be classified as `fallthrough-read`.
- Existing `limux-core` browser handlers are in-memory mock behavior and must
  not be treated as live browser evidence.
- Live browser methods must route to the chosen engine binding.
- Deferred browser methods should return a structured unsupported/deferred
  result through the PRD-E registry shape.

## Security And Audit Frame

The implementation decision must preserve these v1 requirements:

- Engine-layer navigation allowlist, not socket-only method gating.
- Exact host matching plus explicit `*.example.com` wildcard semantics.
- GUI URL bar and restore URL replay must pass through the same allowlist.
- Subresource blocking is out of scope v1 and must be documented as residual.
- Browser audit log path: `~/.local/state/limux/logs/browser-audit.log`.
- Do not log secret-bearing values: `browser.fill` values and script bodies
  are byte-length/selector-only in audit lines.
- Default classification for script/style/cookie/storage mutation methods is
  deferred unless this decision document argues otherwise with a threat note.

## Existing WebKit Pane Disposition

Required if Candidate B1 or B2 wins. Select exactly one after evidence exists:

| Option | Decision | Rationale | Follow-up Work |
|---|---|---|---|
| Remove WebKit default feature | TBD | TBD | TBD |
| Coexist as GUI-only pane while CDP powers agents | TBD | TBD | TBD |
| Fully migrate browser UX to CDP | TBD | TBD | TBD |

## Open Questions To Resolve At Ratification

| Question | Provisional Default | Evidence Needed | Final Answer |
|---|---|---|---|
| Allowlist default contents | Empty deny-all | Operator domain workflow review | TBD |
| High-risk audit events as notifications | Log-only v1 | Operator preference + noise check | TBD |
| CDP attach target if CDP wins | Test Linux Chromium and Windows Chrome | Prototype evidence | TBD |
| Script/style/cookie/storage methods | Deferred v1 | Threat note if not deferred | TBD |

## Recommendation

TBD after measurements. No recommendation is made in this skeleton.

## Ratification

| Field | Value |
|---|---|
| Evidence complete | No |
| Recommendation selected | No |
| Operator/delegated-owner sign-off | Pending |
| F2 implementation allowed | No |
