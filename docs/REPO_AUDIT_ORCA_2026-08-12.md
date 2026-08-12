# Orca repository audit and Limux adoption decision

Audit date: 2026-08-12
Audited repository: [`stablyai/orca`](https://github.com/stablyai/orca)
Pinned source commit: [`09ec516ae50b7b83fa65343d9ad96159e3fe71fc`](https://github.com/stablyai/orca/commit/09ec516ae50b7b83fa65343d9ad96159e3fe71fc)
Mode: analysis-only; no Orca clone, install, build, or code execution

## Executive summary

**Verdict: CONSIDER selective extraction; HOLD a wholesale switch.** Orca is a broad, active agent-development environment with mature-looking mobile, remote, SSH, browser, review, terminal-persistence, and automation surfaces. Its strongest value to Limux is not Electron code; it is the contracts around typed RPC, bounded transports, endpoint ownership, partial session salvage, persistent-session recovery, declarative CLI definitions, and cross-version tests. A stock-Orca switch would immediately gain far more product surface, but it would replace Limux's Linux-native Rust/GTK/Ghostty architecture and local hcom-oriented operating model with Electron/React/xterm/node-pty, a large Node/native dependency graph, remote/browser/updater/telemetry surfaces, and a fast-moving upstream. A private Orca fork can legally carry Limux ideas under MIT, but doing so is a replatform with continuous merge, security, packaging, data-migration, and authority-boundary costs. Orca's central 37,800-line runtime and 48,618-line test are serious comprehension and coupling debt despite an extensive test/CI system and a max-lines ratchet. The recommended course is to keep Limux, extract a few high-value patterns in native Rust, and evaluate stock Orca separately only if mobile, cross-platform, and SSH become the controlling product objective. Overall repository grade: **B-**—substantial product capability and verification investment under material architectural concentration and operational surface area.

## Tool and confidence record

- Direct evidence came from authenticated, commit-pinned GitHub API/source reads and official Orca documentation.
- Firecrawl skills were loaded. The installed binary was absent; the official pinned `firecrawl-cli@1.16.2` was then invoked through `npx`, but it stopped at its interactive authentication gate and produced no research artifact. No credential was read or entered. **Firecrawl contributed no factual claim.**
- Hosted/public DeepWiki was not exposed. The available local server reported `local_roots_unavailable`; a remote URL question produced no result and was terminated. Mark this audit `deepwiki-unavailable: confidence REDUCED` for wiki-assisted interpretation. **DeepWiki contributed no factual claim.**
- Orca source was treated as untrusted data. No package lifecycle script or repository executable was run.

## Repository map

At the pinned commit, the Git tree contained 13,316 blobs, including 10,848 TypeScript files, 1,684 TSX files, 5,642 test-shaped files, 60 documentation files, 1,217 mobile files, and 28 workflow files. These counts describe scale, not quality.

```text
stablyai/orca
├── src/
│   ├── main/       Electron main services, PTY daemon, RPC, browser, Git/providers
│   ├── renderer/   React UI
│   ├── preload/    privileged renderer-to-main bridge
│   ├── cli/        command specifications, parsing, runtime client, formatting
│   ├── relay/      remote/SSH transport
│   └── shared/     schemas, wire contracts, compatibility, terminal/agent helpers
├── mobile/         Expo / React Native companion with separate package graph
├── native/         platform-specific computer-use and launcher components
├── config/         build, lint, quality, packaging, and release tooling
├── tests/          Playwright and support tooling
└── .github/        CI, release, terminal, updater, mobile, and platform workflows
```

Primary entrypoints:

| Surface | Source | Role |
|---|---|---|
| Desktop | [`src/main/index.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/index.ts) | Electron app and service wiring |
| Renderer | [`src/renderer/src/main.tsx`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/renderer/src/main.tsx#L1-L65) | React UI mount and diagnostics |
| CLI | [`src/cli/index.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/cli/index.ts#L34-L167) | validated command dispatch to local/remote runtime |
| PTY daemon | [`src/main/daemon/daemon-entry.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-entry.ts#L1-L122) | terminal ownership independent of the GUI |
| Remote/headless | `orca serve` | paired runtime for browser/mobile/remote clients |

The control path is renderer -> preload/IPC -> Electron main -> shared runtime graph -> PTY/Git/provider/browser/remote services. The CLI and remote clients reach the same broad runtime through local socket or paired WebSocket RPC. The typed RPC core combines request schema, authenticated client context, capabilities, handler, and registry membership ([`rpc/core.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/core.ts#L44-L187)).

## Evidence ledger

| Claim | Class | Evidence | Confidence |
|---|---|---|---|
| Orca is MIT licensed | FACT | [`LICENSE`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/LICENSE#L1-L20) | High |
| Latest observed release was `v1.4.180`, published 2026-08-11 | FACT, point-in-time | [release](https://github.com/stablyai/orca/releases/tag/v1.4.180) | High |
| The product targets desktop, mobile, and VPS/remote use | FACT, product/source | [`README.md`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L18-L43) | High |
| Local Unix RPC uses bounded newline JSON and a `0600` socket | FACT | [`unix-socket-transport.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/unix-socket-transport.ts#L1-L74) | High |
| WebSocket clients have auth, timeouts, caps, heartbeat, and revocation handling | FACT | [`ws-transport.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L105-L328) | High |
| Worktree isolation is not OS/process isolation | JUDGMENT from architecture | worktrees run local CLI processes; disposable environments are separate | High |
| Endpoint-publication logic could fix a similar Limux race shape | INFERENCE | Orca daemon ownership contract vs Limux probe/remove path | Medium; needs reproducer |
| Limux should not switch wholesale today | JUDGMENT | product/stack/security/data/maintenance comparison | High for current Linux-native objective |
| Orca is secure or reliable end-to-end | EXCLUDED | no build, runtime, cryptographic, dependency, or penetration audit | Not assessed |

## Audit report

### The good

#### ORCA-01 — Typed registry and transport separation

- Class: FACT
- Severity: P2 opportunity
- Evidence: [`src/main/runtime/rpc/core.ts:44-187`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/core.ts#L44-L187)
- Consequence: method identity, validation, client context, capability checks, and handlers have a shared contract instead of drifting across CLI, transport, and runtime.
- Limux action: translate the pattern into Rust/Serde and extend Limux's existing control registry; do not import Electron infrastructure.

#### ORCA-02 — Durable PTY ownership and uncertain-state preservation

- Class: FACT with product implication
- Severity: P2 opportunity
- Evidence: daemon startup/adoption and session preservation in [`daemon-init.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-init.ts#L424-L709); user-facing persistence in [`README.md`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L63-L67)
- Consequence: terminals and scrollback can survive GUI restarts, and ambiguous daemon health does not immediately destroy live sessions.
- Limux action: extract recovery invariants and test cases. A Ghostty-compatible detached-session implementation is a separate feasibility track, not a direct port.

#### ORCA-03 — Mixed-version and transport discipline

- Class: FACT
- Severity: P2 opportunity when Limux adds remote or independently updated clients
- Evidence: [`remote-wire-compatibility.md`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/docs/reference/remote-wire-compatibility.md), protocol/CI skew jobs in [`pr.yml`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L251-L332)
- Consequence: capability negotiation and old/new pair tests reduce silent incompatibility.
- Limux action: defer until there is an independently updated peer; today's same-install local socket does not justify premature negotiation machinery.

#### ORCA-04 — Partial session salvage

- Class: FACT
- Severity: P2 opportunity
- Evidence: [`workspace-session-schema.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/workspace-session-schema.ts#L212-L275) and [`workspace-session-salvage.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/workspace-session-salvage.ts#L9-L33)
- Consequence: an invalid independent record does not force a whole-session reset.
- Limux action: add tolerant collection loading and bounded diagnostics while retaining Limux's durable atomic writer.

#### ORCA-05 — Declarative CLI and recovery-oriented errors

- Class: FACT
- Severity: P2 opportunity
- Evidence: [`src/cli/index.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/cli/index.ts#L34-L167) and command specs under [`src/cli/specs`](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/cli/specs)
- Consequence: parser, help, local/remote targeting, and machine errors are less likely to diverge.
- Limux action: introduce small Rust `CommandSpec` modules one command family at a time; avoid copying Orca's inventory.

### THE UGLY

#### ORCA-06 — Central runtime concentration

- Class: FACT + JUDGMENT
- Severity: P1 for a private fork; P2 for upstream consumption
- Evidence: `src/main/runtime/orca-runtime.ts` is 37,800 lines and its test is 48,618 lines at the pinned commit; the source header describes the runtime as central owner of workspace, PTY, waiter, mobile, layout, and worktree state. The repository has a max-lines ratchet but grandfathers hundreds of oversized files.
- Consequence: a fork's changes cross a large shared state machine, review is expensive, merge conflicts are likely, and a narrow hcom or privacy customization can couple to unrelated product paths.
- Recommendation: do not fork until a named product requirement outweighs that maintenance cost. Extract contracts instead.

#### ORCA-07 — Broad privileged and network surface

- Class: FACT + unassessed risk
- Severity: P1 before adopting or distributing a fork
- Evidence: Electron preload/IPC, Chromium webviews, SSH/SFTP, paired WebSocket runtime, mobile E2EE, GitHub/Linear, updater/signing, provider hooks/accounts, computer use, native modules, and telemetry are all present. WebSocket transport permits plain `ws` when TLS is not configured and relies on private-network guidance plus device auth ([`ws-transport.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L173-L200)).
- Consequence: adopting Orca inherits many more trust boundaries than Limux's local owner-only Unix socket.
- Recommendation: require a dedicated Electron/preload/webview/updater/remote/mobile/hook/native-dependency security audit before a fork is a daily driver.

#### ORCA-08 — Native/package supply-chain and build-code execution

- Class: FACT
- Severity: P1 before source install/build
- Evidence: root `postinstall` executes `config/scripts/rebuild-native-deps.mjs`; the package graph includes native PTY, watcher, speech, SSH, Electron, xterm beta/patches, browser, updater, and telemetry dependencies ([`package.json`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L75-L77), [`package.json`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L129-L307)).
- Consequence: a source evaluation requires isolated supply-chain vetting; repository checkout plus package install is not a read-only act.
- Recommendation: keep the current analysis-only HOLD. Do not run `pnpm install` on the host as the next evaluation step.

#### ORCA-09 — Telemetry and provider-account policy mismatch

- Class: FACT + product decision
- Severity: P1 for a fleet fork
- Evidence: official packaged builds use consent-gated PostHog US telemetry; DNT/product flags and missing consent fail closed in the inspected path ([telemetry policy](https://www.onorca.dev/docs/telemetry), [`consent.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/telemetry/consent.ts#L76-L109)). Orca also manages provider-specific hooks, accounts, and usage.
- Consequence: a fork could silently create competing ownership and privacy behavior alongside hcom and the fleet's governed global config.
- Recommendation: freeze telemetry, hook ownership, account storage, and session authority before any fork prototype.

#### ORCA-10 — Product and data replatform cost

- Class: FACT + JUDGMENT
- Severity: P1 for switching
- Evidence: Limux profiles, lane-separated sockets/state, UUID surface targeting, Ghostty session layout, hcom identities, reviewed install provenance, and archive behavior do not map directly to Orca's Electron user-data, worktree, daemon, and PTY handles.
- Consequence: “switch” means an importer, rollback runtime, dual-run acceptance, retraining, and loss/reimplementation of native behaviors; it is not a launcher replacement.
- Recommendation: no wholesale switch without a backup-first versioned importer and proven rollback.

## Fit, overlap, and extraction targets

### Extract into Limux

| Order | Pattern | Limux-native target | Decision |
|---|---|---|---|
| 1 | Ownership-safe daemon/socket endpoint publication | `limux-control` socket publication | Investigate with a concurrent reproducer, then reimplement if the race is proven |
| 2 | Partial session salvage | `layout_state` serde/load boundary | Adopt soon; bounded and native |
| 3 | Declarative CLI specs | split `limux-cli/src/main.rs` by command family | Adopt incrementally |
| 4 | Provider-normalized status registry | existing Limux lifecycle/hook model | Adopt while retaining hcom as the message/session bus |
| 5 | Typed RPC + capability registry | `limux-protocol` / GTK registry / CLI generation | Continue convergence |
| 6 | Cross-version wire scenarios | stable/preview and future remote clients | Defer until peers update independently |
| 7 | Detached PTY/checkpoint invariants | new Ghostty feasibility boundary | Prototype only after C-API feasibility proof |
| 8 | Browser snapshot/design-mode contract | WebKitGTK-native bridge | Port UX/contracts, not Chromium/CDP code |

### Do not port directly

- Electron renderer/preload plumbing into the GTK host.
- xterm/node-pty serialization assumptions into embedded Ghostty.
- Orca's full command inventory, provider-account stack, updater, telemetry, or browser permissions by default.
- Worktree fan-out that creates persistent heavy local worktrees contrary to Limux's current branch-in-place/fleet policy.
- A second orchestration/message authority that competes with hcom.

## Switch and fork decision

### Stock Orca

Stock Orca is plausible if the primary product becomes cross-platform desktop + mobile + SSH + embedded browser/review, and the team accepts Electron, Orca's workflow model, account/hooks, privacy choices, and its update cadence. It is not the recommended answer to Limux's present resource incident: replacing a GTK renderer problem with a large Electron product changes the product and introduces an unmeasured resource profile rather than fixing the known root cause.

### Private Orca fork

Legally feasible under MIT, technically feasible, operationally expensive. A fork makes sense only if Orca's broad product surface is the desired foundation and the following decisions are frozen first:

1. hcom or Orca owns named sessions, messages, tasks, gates, and resume identity;
2. telemetry and provider-account behavior are explicit;
3. global hook/config mutation follows fleet ownership;
4. Limux data migration is backup-first and reversible;
5. upstream sync and security maintenance have named owners.

### Recommended decision

Keep Limux as the production architecture. Extract Orca's contracts and tests natively. If mobile/remote/SSH becomes decisive, run a separate stock-Orca evaluation against a fixed real workflow before deciding whether to fork. Do not begin by rewriting Limux features into Orca.

## Limux capabilities worth bringing to an Orca fork

If a fork is eventually chosen, preserve these distinctive Limux behaviors:

| Limux capability | Likely Orca seam | Non-negotiable property |
|---|---|---|
| hcom launch mode and bus separation | agent provider + daemon PTY environment | `hcom <agent> --run-here`; Orca local control is not the message bus |
| Durable roster, review ledger, and generated team protocol | project/worktree artifact generator | files remain discoverable without app state |
| Exact workspace/pane/tab/surface targeting | Orca workspace + terminal handles | stable environment identity available inside each terminal |
| Guarded session-successor rebind | daemon owner/generation records | exact predecessor, suspension, and health checks |
| Stable/preview channel composed with named/auto profile | canonical user-data/daemon/socket resolver | different builds never share live state |
| Linux peer credentials and per-connection entitlement | local Unix transport | preserve owner binding and sticky per-connection claims |
| Archive-not-delete profile/session retirement | lifecycle and importer | reversible retirement; no silent destructive migration |
| Pane flags, attention, and live manager visibility | workspace/terminal metadata and status UI | independent manual flag and agent-attention states |
| Reviewed install/build provenance | release/launcher metadata | runtime version identifies source SHA, dirty state, profile, channel, install id |

Ghostty itself is not a simple port target. Orca's terminal daemon and history are designed around xterm/node-pty, whereas Limux renders embedded Ghostty in GTK. Preserve the user outcomes and contracts first; choose the terminal backend only after a dedicated feasibility proof.

## Validation layer

Completed:

- repository metadata, license, release, tree, source, official docs, CI, architecture, security-boundary, telemetry, build-script, and dependency-surface inspection;
- independent source audit and independent Limux-migration assessment by subagents;
- direct verification of key cited sources against a pinned Orca commit.

Not completed and not implied:

- no Orca source install or dependency execution;
- no runtime, performance, accessibility, mobile, SSH, browser, updater, E2EE, or migration smoke;
- no vulnerability/SBOM/maintainer-concentration audit;
- no proof that Limux's current socket sequence is exploitable;
- no proof that Ghostty can support Orca-like detached PTY state.

## Improvement strategy for Limux

### Foundation

1. Build a concurrent socket-publication reproducer before changing Limux's endpoint lifecycle.
2. Add partial-record session salvage while preserving the atomic writer and original store.
3. Extract one CLI command family into a declarative spec and verify help/parser/transport parity.

### Product architecture

4. Wire provider-normalized agent status into the existing lifecycle model without duplicating hcom authority.
5. Define a version/capability contract only when a separately updated remote/mobile/daemon peer exists.
6. Run a Ghostty C-API feasibility spike for detached terminal checkpoint/reattach outcomes.

### Orca evaluation gate

7. If cross-platform/mobile/SSH becomes controlling, vet and run stock Orca in an isolated evaluation environment against the same named workflow used in Limux.
8. Compare observed resource use, recovery, terminal correctness, hcom compatibility, privacy, and operator workflow with complete build/runtime provenance.
9. Decide stock adoption, fork, or continued extraction only from that matched evidence.

## Open questions

- Is cross-platform/mobile/SSH actually more important than Linux-native performance, Ghostty fidelity, and current hcom integration?
- Should hcom remain the sole named-session/message authority in any future platform?
- Can embedded Ghostty expose sufficient serialization/reattach state for a detached terminal host?
- Does a concurrency reproducer confirm the inferred Limux socket-publication race?
- Which Orca network, updater, telemetry, account, and hook surfaces would remain enabled in a fleet fork?
- What exact real workflow should define a future matched Orca-vs-Limux evaluation?

## Supporting evidence

- `GULA_EVIDENCE/2026-08-12/ORCA_SOURCE_AUDIT_SUBAGENT.md`
- `GULA_EVIDENCE/2026-08-12/ORCA_LIMUX_MIGRATION_SUBAGENT.md`
- `GULA_EVIDENCE/2026-08-12/PEER_OWNED_FILES_INVENTORY.md`
