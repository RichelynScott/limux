# Orca -> Limux extraction and migration assessment

**Author:** gula subagent
**Date:** 2026-08-12
**Purpose:** read-only evidence for the Limux manager's independent decision
**Preliminary disposition:** **CONSIDER selective extraction; HOLD a wholesale switch unless cross-platform/mobile/SSH becomes the controlling product objective.**

## Scope, snapshots, and confidence

- Local Limux snapshot: branch `main`, commit `ffafacb74e403964205be4ce29440f4eb22dc6ab`, workspace version `0.2.3`, MIT (`Cargo.toml:1-16`).
- Orca snapshot: `stablyai/orca` `main` at commit `09ec516ae50b7b83fa65343d9ad96159e3fe71fc` (GitHub commit API, committed 2026-08-12T10:24:31Z). Every Orca source read below was fetched with `gh api repos/stablyai/orca/contents/<path>?ref=<sha>`; no clone, install, build, or remote code execution occurred.
- The only filesystem mutation in this sublane is this report.
- **DeepWiki/Firecrawl status:** not performed in this sublane because the delegated scope explicitly restricted Orca inspection to `gh` API/source URLs. The parent owns the separately requested DeepWiki/Firecrawl lanes. Consequently this is a source extraction report, not the final `$evaluate-repo` verdict.
- Labels used below:
  - **VERIFIED-SOURCE**: supported by the pinned/local source cited.
  - **INFERENCE**: architecture conclusion from source, not demonstrated at runtime.
  - **UNVERIFIED-RUNTIME**: no build, benchmark, UI smoke, security test, or migration prototype was run.
  - **NOT-ASSESSED**: explicitly outside this sublane.

## Executive answer

There is useful Orca structure to bring into Limux, but most of it should be **translated into Rust/GTK/Ghostty-native modules**, not copied wholesale. The highest-value candidates are:

1. ownership-safe daemon/socket endpoint publication;
2. detached terminal lifetime plus bounded checkpoint/scrollback recovery;
3. field/record-level session salvage instead of whole-session reset;
4. capability-negotiated remote protocols with mixed-version tests;
5. provider-normalized agent lifecycle reporting;
6. declarative CLI command specifications; and
7. optional browser snapshot/design-mode contracts.

A full switch to an Orca fork is legally possible because both repositories use MIT, but it is a **product replatform**, not an incremental migration. It exchanges Limux's Linux-native GTK/libadwaita + embedded Ghostty + local Unix-socket + hcom-specific identity model for Electron/React/Xterm, Node/native dependencies, remote/mobile/SSH surfaces, telemetry, packaging/signing, and a fast-moving upstream. Orca's feature set is much broader, but a private fork would inherit a substantial integration and security-maintenance obligation.

If the project forks Orca, the Limux features most worth carrying forward are its explicit hcom bus boundary, durable team/review artifacts, per-terminal `LIMUX_*` targeting semantics, resumable hcom session rebinding, stable/preview lanes composed with session profiles, archive-not-delete profile handling, pane flags/attention, and live directory-manager visibility.

## What is uniquely valuable in Limux

| Capability to preserve | Evidence | Why it is not just generic terminal functionality |
|---|---|---|
| Native Linux presentation and Ghostty rendering | `README.md:1-24`, `README.md:548-560`; `rust/limux-host-linux/src/terminal.rs:23-52` | GTK4/libadwaita and embedded `libghostty.so` provide a Linux-native UI/rendering stack. Replacing it with Electron/Xterm changes the core product, not merely its implementation. |
| Orthogonal runtime channels and session profiles | `README.md:114-161`, `README.md:244-269`; `rust/limux-control/src/socket_path.rs:29-57,104-151`; `rust/limux-control/src/session_paths.rs:1-76` | Stable/preview build lanes and named/auto session sets compose without sharing sockets or state. The single path authority is also a strong correctness pattern. |
| Recoverable profile retirement | `README.md:137-146` | `profile rm` archives instead of deleting and refuses a running profile. This is a product-level no-loss behavior worth preserving in any fork. |
| Local caller identity and narrow control boundary | `rust/limux-control/src/auth.rs:13-72`; `rust/limux-control/src/socket_path.rs:203-240`; `rust/limux-control/src/request_io.rs:7-46` | Linux peer credentials, owner-only modes, 0700/0600 path permissions, request-size, connection, and idle bounds form a compact local automation boundary. |
| Exact pane/surface targeting from spawned shells | `rust/limux-host-linux/src/pane.rs:1535-1556`; `rust/limux-cli/src/main.rs:5342-5372,5488-5566` | Every terminal gets workspace/surface/pane/tab/socket identity. `read-screen` deliberately defaults to the caller's exact surface to avoid cross-lane reads in a shared workspace. |
| hcom-native launch and explicit control-bus/message-bus separation | `rust/limux-cli/src/main.rs:2892-2941`; `README.md:399-416` | `hcom <agent> --run-here` keeps named persistent sessions inside visible Limux panes while Limux remains the GUI control bus. That division is specific and operationally useful. |
| Durable, file-first team and review coordination | `README.md:371-397,429-462`; `rust/limux-cli/src/main.rs:5074-5268` | The generated runtime protocol points to instruction sources and seeds a roster/review ledger without overwriting existing durable records. Review prepare/spawn creates durable requests and evidence pointers instead of relying on terminal history. |
| Restorable agent identity, suspension, and successor rebind | `rust/limux-host-linux/src/layout_state.rs:249-325,327-433` | Claude/Codex/OpenCode/Gemini/Hermes sessions retain launch identity, suspension reason, hcom name, and a guarded predecessor-to-successor rebind operation. |
| Operator-visible attention and manager context | `rust/limux-host-linux/src/pane.rs:232-286`; `README.md:168-203` | Manual pane flag colors coexist with pane attention, while the header exposes live process-tree resources and hcom directory-manager claims for the active workspace. |
| Durable state commit and crash classification | `rust/limux-host-linux/src/durable_atomic.rs:7-38,56-113`; `rust/limux-host-linux/src/runtime_lifecycle.rs:7-77,95-151` | Locking, no-follow regular-file checks, fsync+rename+parent-fsync, and incarnation-aware clean/unclean markers are valuable regardless of front end. |
| Method classes and mutation kill switch | `rust/limux-host-linux/src/control_registry.rs:1-19,21-218,220-282` | Native, read-only, mutation, and deferred routes live in a central registry; wired Wave-1 mutations can be disabled as a group. |

Two limits matter when comparing feature lists:

- **VERIFIED-SOURCE:** Limux's browser is opt-in via `LIMUX_ENABLE_WEBKIT_BROWSER` (`rust/limux-host-linux/src/pane.rs:160-179`), and the README says browser bridge parity remains open (`README.md:475-482`).
- **VERIFIED-SOURCE:** the PRD-G lifecycle state machine is explicitly not wired to GTK/socket/CLI yet (`rust/limux-host-linux/src/agent_state.rs:1-10`). It must not be counted as a shipped end-to-end agent-status feature.

## A. Orca components and patterns that would benefit Limux

| Priority | Orca component/pattern | Orca evidence | Limux seam / evidence | Recommendation and caveat |
|---|---|---|---|---|
| A1 | Ownership-safe daemon endpoint publication | Orca documents private bind -> exclusive link -> liveness proof -> identity recheck -> second probe -> atomic rename, and forbids removing a name the actor did not create (`stablyai/orca:src/main/daemon/AGENTS.md:3-50`). | Limux connects to the current socket and, after a stale-class error, removes the pathname in a separate operation (`rust/limux-control/src/socket_path.rs:270-297`). | **Extract first.** Reimplement the ownership protocol in `limux-control` and add a concurrent publisher/replacement test. **INFERENCE:** Limux's probe-then-remove sequence has the same TOCTOU shape Orca retired; a live race is not proven here. |
| A2 | Detached PTY host with warm attach and bounded terminal checkpointing | Orca's `TerminalHost` owns sessions independently, attaches/detaches clients, snapshots live sessions, and final-checkpoints on shutdown (`stablyai/orca:src/main/daemon/terminal-host.ts:24-76,117-179,200-239`). History recovery freezes/fingerprints generations and supports warm reattach/reopen (`stablyai/orca:src/main/daemon/history-manager.ts:35-68,110-195`). Oversized checkpoints are replayed and binary-searched down to the largest bounded scrollback (`stablyai/orca:src/main/daemon/terminal-checkpoint-serializer.ts:211-280`). | Limux embeds Ghostty surfaces in the GTK host (`rust/limux-host-linux/src/terminal.rs:23-52`); its persisted terminal tab contains cwd and optional agent, but no terminal buffer (`rust/limux-host-linux/src/layout_state.rs:474-489`). | **Prototype as a boundary, not a port.** First define a renderer-neutral terminal session/checkpoint trait; then test whether Ghostty's C API can serialize/rehydrate sufficient state. **INFERENCE/UNVERIFIED-RUNTIME:** source shape indicates Limux does not persist scrollback; no restart smoke was run. |
| A3 | Partial session salvage | Orca validates persisted state at the read boundary and drops corrupt independent records rather than the full session (`stablyai/orca:src/shared/workspace-session-schema.ts:1-32,212-275`; `stablyai/orca:src/shared/workspace-session-salvage.ts:9-33`). | Limux deserialization failure falls back to the entire default session (`rust/limux-host-linux/src/layout_state.rs:708-745`), though successful loads are normalized and writes are durable (`rust/limux-host-linux/src/layout_state.rs:758-769`). | **Extract soon.** Implement serde-level tolerant collections plus bounded diagnostics/quarantine. Preserve Limux's atomic writer. This is smaller and lower-risk than the terminal daemon. |
| A4 | Explicit mixed-version compatibility and negotiated capabilities | Orca blocks too-old client/server pairs with pure compatibility evaluators (`stablyai/orca:src/shared/protocol-compat.ts:1-64`). Its remote-wire contract requires capability negotiation for new opcodes and runs current-vs-release skew tests in both directions (`stablyai/orca:docs/reference/remote-wire-compatibility.md:1-10,31-59,78-100`). Relay framing has a version handshake, keepalive, bounded concurrency, ack windows, and bulk-lane chunking to avoid terminal head-of-line blocking (`stablyai/orca:src/relay/protocol.ts:27-109`). | Limux has V1->V2 request adaptation and a capability method registry (`rust/limux-protocol/src/lib.rs:19-105`; `rust/limux-host-linux/src/control_registry.rs:21-218,247-259`), but the request envelope itself carries no negotiated protocol version. | **Extract when adding a detached daemon, SSH, or mobile client.** For today's same-install local socket this is lower priority; adding negotiation without an independently updated peer would be premature. |
| A5 | Worktree ownership/provenance model | Orca separates strong `orca-managed` metadata from `agent-scratch`, `external`, and `unknown-legacy`; location alone is not proof Orca created a worktree (`stablyai/orca:src/shared/worktree-ownership.ts:139-170`). Visibility then honors explicit imports and hides scratch worktrees by default (`stablyai/orca:src/shared/worktree-ownership.ts:173-233`). | Limux currently models folder workspaces/panes, not a first-class Git worktree lifecycle (`README.md:9-24`). | **Extract only if Limux adds repo/worktree UX.** Adopt the ownership/provenance classification, not Orca's worktree-first product default. Current fleet governance treats routine local worktrees as exceptional, so automatic fan-out would conflict with the operating model. |
| A6 | Browser accessibility snapshots and design-mode capture | Orca builds stable element refs from Chromium accessibility/DOM data, including cursor-interactive elements and cross-origin iframe sessions (`stablyai/orca:src/main/browser/snapshot-engine.ts:1-42,78-187`). Its product exposes element HTML/CSS/screenshot capture (`stablyai/orca:README.md:77-81`). | Limux has an opt-in WebKitGTK browser and explicitly incomplete browser bridge parity (`rust/limux-host-linux/src/pane.rs:160-179`; `README.md:475-482`). | **Port the command/result contracts and UX, not CDP code.** WebKitGTK does not provide the same Electron/CDP seam; the implementation needs a WebKit-native accessibility/JS bridge and a separate threat review. **UNVERIFIED-RUNTIME:** no WebKit feasibility spike was run. |
| A7 | Provider-normalized status and durable orchestration primitives | Orca strips/parses bounded OSC 9999 agent-status frames even without a mounted terminal (`stablyai/orca:src/shared/agent-status-osc.ts:4-85`) and centralizes install/remove/status adapters for many agent families (`stablyai/orca:src/main/agent-hooks/managed-agent-hook-registry.ts:18-92`). Its orchestration contract distinguishes mutations and provides run/message/task/worker/gate operations (`stablyai/orca:src/shared/orchestration-rpc-contract.ts:1-90`; `stablyai/orca:src/cli/specs/orchestration.ts:5-24,80-188,236-261`). | Limux already translates hooks and integrates hcom, but its in-host aggregate lifecycle model is not yet wired (`README.md:307-347`; `rust/limux-host-linux/src/agent_state.rs:1-10`). | **Extract status normalization and provider registry structure.** Do not replace hcom with a second overlapping message bus by default; keep Limux's explicit bus boundary and add an adapter to Orca-like status/run primitives only where useful. |
| A8 | Declarative CLI command specs and compatibility recovery | Orca separates command paths, flags, usage, and behavioral notes (`stablyai/orca:src/cli/specs/orchestration.ts:5-24,80-188`) and makes retired orchestration commands return a no-effects migration guide (`stablyai/orca:src/shared/orchestration-rpc-contract.ts:41-90`). | Limux's CLI behavior, parser, orchestration, docs generation, and file protocols are concentrated in `rust/limux-cli/src/main.rs` (10,707 lines at this snapshot; line-count evidence from `wc -l`). | **Extract gradually.** Introduce Rust `CommandSpec` modules and generate help/validation from them before adding commands. **INFERENCE:** this should reduce drift, but no refactor prototype or compile check was performed. |

### Smallest practical extraction sequence

1. A1 endpoint publication hardening.
2. A3 session salvage.
3. A8 declarative CLI specs for one narrow command family.
4. A7 wire the existing Limux lifecycle model using a provider registry/status parser while retaining hcom.
5. Decide whether A2 detached terminal persistence is worth a prototype.
6. Gate A4/A6/A5 on explicit remote/browser/worktree product goals.

This order is a judgment recommendation, not measured effort data.

## B. Wholesale switch feasibility and hidden costs

| Dimension | Switch-to-Orca finding | Evidence / status |
|---|---|---|
| Legal | **Feasible with notice preservation.** Both are MIT. An Orca fork or copied substantial source must retain Orca's copyright and permission notice. | Limux `Cargo.toml:12-16`; `stablyai/orca:LICENSE:1-20`. |
| Platform reach | Orca already targets macOS, Windows, Linux, headless Linux, iOS, and Android; this is a major gain if those are product requirements. | `stablyai/orca:README.md:210-233`; cross-platform rules at `stablyai/orca:AGENTS.md:29-49`. |
| Core UI/terminal | This is a replatform from Rust/GTK/libadwaita/Ghostty to Electron 43/React 19/Xterm WebGL. Retaining the native Limux renderer would require a second Linux client or a deep hybrid integration. | Limux `README.md:548-560`; `stablyai/orca:package.json:196-205,226-250`. **INFERENCE:** no hybrid spike was run. |
| Feature coverage | Orca already has restart-surviving terminal scrollback, worktree fan-out, Chromium design mode, native review/editing, SSH workspaces, mobile steering, and broad CLI automation. | `stablyai/orca:README.md:35-151,160-167`. These are upstream product claims corroborated selectively by source, not end-to-end tested in this sublane. |
| Agent orchestration | Orca's durable run/task/worker/message/gate primitives overlap substantially with Limux agent-team + hcom. A switch requires choosing one authority or writing a clear adapter; running both as peer authorities risks split state. | `stablyai/orca:src/shared/orchestration-rpc-contract.ts:18-60`; Limux `README.md:387-416`. **INFERENCE:** split-brain risk is architectural, not reproduced. |
| Worktree operating model | Orca is explicitly worktree-first: agents run in isolated worktrees and compare/merge results. That conflicts with this environment's branch-in-place default and exceptional-worktree governance. | `stablyai/orca:README.md:18-20,49-53`; environment policy constraint. A fork would need to disable or adapt defaults, not merely rebrand them. |
| Build/supply chain | Orca requires Node 24/pnpm, Electron, React, native `node-pty`, watcher, speech, SSH, WebSocket and telemetry dependencies, plus several patched Xterm packages and native build allowlists. Limux's primary app is a six-crate Rust workspace plus Zig-built Ghostty and Linux UI libraries. | `stablyai/orca:package.json:129-153,196-205,274-307`; Limux `Cargo.toml:1-25`, `README.md:205-224`. **NOT-ASSESSED:** dependency vulnerability/SBOM audit. |
| Release operations | Orca adds desktop multi-OS packaging/signing/updating and mobile store/APK workflows. Limux already has isolated user-local stable/preview/rollback lanes with install provenance. | `stablyai/orca:README.md:210-233,263-268`; Limux `README.md:244-269`. |
| Security boundary | Orca adds Chromium guest pages, SSH/relay/WebSocket, mobile pairing/E2EE, Git/provider integrations, updater, and telemetry. Its browser policy auto-grants clipboard read, notifications, persistent storage, fullscreen and pointer lock within managed sessions (`stablyai/orca:src/main/browser/browser-session-permission-policy.ts:1-20`). Its E2EE v2 contract validates exact handshake records, key/nonces, version and transport context (`stablyai/orca:src/shared/mobile-e2ee-v2-contract.ts:1-40,45-107`). | **NOT-ASSESSED:** cryptographic correctness, Electron hardening, updater trust, browser isolation, SSH host-key flow, and production relay infrastructure. The presence of controls is not proof of security. |
| Privacy/telemetry | Official stable/RC builds have telemetry transport compiled on, gated by official build identity/write key and effective consent; events are dropped unless consent resolves enabled. DO_NOT_TRACK/product disable/CI/user opt-out are supported. | `stablyai/orca:src/main/telemetry/client.ts:19-36,66-115,163-204`; `stablyai/orca:src/main/telemetry/consent.ts:1-16,76-110`. A Limux fork must make an explicit privacy decision rather than inheriting silently. |
| Upstream/fork cost | Orca's source spans desktop main/renderer/relay/CLI, native code, mobile, and extensive compatibility surfaces. A private downstream carrying different worktree, hcom, telemetry, and Linux-native preferences will continuously reconcile with a fast-moving upstream. | `stablyai/orca:README.md:160-167,245-247`; `stablyai/orca:pnpm-workspace.yaml:1-4`; broad source layout observed through pinned Git tree API. **INFERENCE:** no quantified maintenance estimate is offered. |
| Migration/data compatibility | Limux's profile/lane paths, session schema, pane IDs, Ghostty state, hooks, installed launchers, and hcom bindings do not map directly to Orca user data and PTY/session handles. | Limux `rust/limux-control/src/session_paths.rs:1-90`, `rust/limux-host-linux/src/layout_state.rs:12-73,203-325`; Orca `stablyai/orca:src/shared/workspace-session-schema.ts:1-12,111-165,212-275`. **UNVERIFIED-RUNTIME:** no importer was designed or tested. |

### Switch decision boundary

- **Keep Limux and extract patterns** when Linux-native GTK/Ghostty, local-first control, hcom visibility/recovery, and stable/preview profile isolation are load-bearing.
- **Fork Orca** when cross-platform desktop + mobile + SSH + rich Git/editor/browser capability is the controlling objective and the project accepts Electron/Node, worktree-model adaptation, upstream merge duty, and a new security/release program.
- **Do not call a fork a Limux upgrade.** It would be a downstream Orca distribution that ports selected Limux behaviors.

## C. If Orca is forked, Limux features to port and likely seams

| Limux feature | Orca seam | Port shape | Main risk |
|---|---|---|---|
| hcom launch mode | Agent launch/provider catalog plus terminal session env. Orca passes `env` into daemon PTY creation and retains `ORCA_TERMINAL_HANDLE` (`stablyai/orca:src/main/daemon/terminal-host-session-create.ts:74-113`). | Add an explicit hcom provider/launch wrapper that invokes `hcom <agent> --run-here`; bind hcom name/session to Orca terminal handle and workspace key. | Global hook/config ownership and competing Orca orchestration authority. |
| Exact per-pane callback identity | Orca already resolves `ORCA_TERMINAL_HANDLE` and `ORCA_PANE_KEY`, including stale-handle validation/reminting (`stablyai/orca:src/cli/handlers/orchestration.ts:205-277`). | Map `LIMUX_WORKSPACE_ID/SURFACE_ID/PANE_ID/TAB_ID/SOCKET` semantics to stable Orca workspace/tab/pane/terminal handles; preserve explicit-target reads. | Identifier lifetimes differ; a superficial env rename can misroute after restore/remint. |
| File-first team protocol, roster, ledger, and review requests | Orca command spec/handler modules and bundled orchestration skill (`stablyai/orca:src/cli/specs/orchestration.ts:5-24,80-188`; `stablyai/orca:src/shared/orchestration-rpc-contract.ts:63-90`). | Add `hcom`/`limux-compat` subcommands that create marked, non-clobbering artifacts and link them to Orca runs/tasks without duplicating state. | Two durable task/message systems can disagree unless one is declared canonical. |
| Session successor rebind | Orca terminal host already tracks claimed agent owners and generations (`stablyai/orca:src/main/daemon/terminal-host.ts:26-76`) and session creation refuses ambiguous generations (`stablyai/orca:src/main/daemon/terminal-host-session-create.ts:33-71`). | Store hcom identity alongside Orca's terminal/session generation and add guarded predecessor->successor rebinding. | Rebinding a live/stale wrong generation. Require exact old identity and suspension/health gates. |
| Stable/preview channels composed with named/auto profiles | Orca build channels and `ORCA_USER_DATA_PATH` exist, but this sublane did not find a source-equivalent of Limux's orthogonal profile/lane path authority. | Namespace Electron `userData`, daemon endpoint, relay credentials and session store by `(channel, profile)` in one canonical resolver; add archive-not-delete profile retirement. | Every process and launcher must agree; missing one path creates cross-build state/socket mixing. **INFERENCE.** |
| Pane flags, attention, and directory-manager header | Orca renderer sidebar/workspace/tab state and agent status projections. | Add durable manual flag colors separate from transient needs-input/unread, plus a read-only hcom manager/resource header provider. | Renderer feature churn and polling/process-cost. |
| Linux peer-credential socket mode | Orca daemon hello currently uses protocol version/token in health checks (`stablyai/orca:src/main/daemon/daemon-health.ts:50-53,106-151`). | Add optional Linux `SO_PEERCRED` defense-in-depth without replacing the cross-platform token/credential handshake. | Cannot be the cross-platform auth primitive; WSL/SSH/Windows semantics differ. |
| Native Ghostty/GTK experience | No direct Orca seam; Orca uses Electron/Xterm. | Keep Limux as a Linux-native client against a shared protocol, or accept that this capability is retired. | A dual-client architecture is a separate product and protocol program. **UNVERIFIED-RUNTIME.** |

## D. Blockers and non-negotiable gates

1. **Language/toolkit blocker:** Rust/GTK/libadwaita/Ghostty source cannot be dropped into Electron/TypeScript/React/Xterm. Algorithms and contracts can be translated; UI/terminal code generally cannot (`README.md:548-560`; `stablyai/orca:package.json:196-205,226-250`).
2. **Terminal state blocker:** a Ghostty-compatible checkpoint/reattach mechanism must be empirically proven before promising Orca-style restart-surviving scrollback in Limux. **UNVERIFIED-RUNTIME.**
3. **Platform blocker:** keeping Orca's Windows/macOS/mobile reach makes Linux-only assumptions, `/proc`, `SO_PEERCRED`, GTK, and Unix socket paths optional extensions rather than global invariants (`stablyai/orca:AGENTS.md:29-49`).
4. **Operating-model blocker:** Orca's worktree-first UX must be reconciled with the fleet's branch-in-place and exceptional-worktree policy before adoption. This is a policy/product decision, not a code merge.
5. **Security gate:** before adopting or distributing Orca code, run a dedicated audit of Electron main/preload/renderer boundaries, guest permissions, updater/signing, agent hook installers, remote runtime/SSH, relay/mobile credentials, E2EE key schedule/framing, telemetry schema, native dependencies, and package lifecycle scripts. **NOT-ASSESSED here.**
6. **Supply-chain gate:** produce a pinned dependency risk map/SBOM and review Orca's native builds and patched dependencies before executing package code. `package.json` names built native dependencies and five patched packages (`stablyai/orca:package.json:274-307`).
7. **License gate:** preserve Orca's MIT copyright and permission notice in copied substantial portions and fork distributions (`stablyai/orca:LICENSE:1-20`).
8. **Data migration gate:** design a versioned, backup-first importer for Limux profiles/session data and prove rollback. Do not reinterpret Limux IDs or delete the original stores. **UNVERIFIED-RUNTIME.**
9. **Authority gate:** decide whether hcom, Orca orchestration, or a defined adapter owns named sessions, messages, tasks, gates, and resume identity. Do not ship dual ambiguous authorities.
10. **Upstream strategy gate:** choose upstream-following fork vs one-time source extraction, including notice policy, patch budget, security-update SLA, and criteria for dropping/rebasing local deltas. **Operator decision.**

## Recommended decision

**Default recommendation: keep Limux and extract A1/A3/A8 first; prototype A2 only after a Ghostty checkpoint feasibility test.** Those changes address concrete reliability/maintainability gaps without surrendering the product's native Linux and hcom advantages.

Use an Orca fork only if the operator explicitly chooses a broader cross-platform ADE product. In that case:

1. treat the fork as a new product lane;
2. begin from a pinned Orca commit with an upstream-sync contract;
3. disable or decide telemetry explicitly;
4. adapt worktree defaults to fleet policy;
5. port hcom identity + durable artifacts before migration;
6. add a backup-first Limux importer; and
7. preserve Limux as the rollback runtime until real workflows pass.

## Unverified claims and open evidence

- **UNVERIFIED-RUNTIME:** Limux's current session restart does not preserve terminal scrollback. Source state contains no buffer, but no live restart smoke was run.
- **UNVERIFIED-RUNTIME:** Ghostty's embedded C API can support an Orca-like detached daemon/checkpoint architecture without losing fidelity or violating threading/lifetime rules.
- **INFERENCE:** Limux's stale-socket liveness-check/remove sequence is exposed to the same publication race Orca documents. A targeted concurrent reproducer is still required.
- **UNVERIFIED-RUNTIME:** WebKitGTK can provide equivalent accessibility/DOM/screenshot/ref semantics to Orca's CDP snapshot/design-mode path.
- **NOT-ASSESSED:** Orca's security, privacy, cryptographic correctness, performance, accessibility, Linux packaging quality, and real-world feature reliability.
- **NOT-ASSESSED:** dependency advisories, maintainer concentration, contribution/review responsiveness, and hidden external services.
- **DEFERRED:** DeepWiki and Firecrawl corroboration, owned by the parent lane.
- **UNVERIFIED-RUNTIME:** exact migration effort, schedule, ongoing fork cost, and performance/resource comparison. No estimates are manufactured in this report.

## Methodology note

Loaded and followed `$repo-audit`, `$evaluate-repo`, `$limux-use-guide`, `$github-cli`, and `$karpathy-guidelines`. The audit emphasized source evidence and explicit uncertainty; the repository evaluation is intentionally incomplete pending the parent-owned DeepWiki/Firecrawl corroboration. Local source was read at the recorded Limux HEAD. Orca was inspected only through pinned `gh api` content/tree/repository endpoints. No remote code was executed and no conclusion here substitutes for the manager's independent verification.
