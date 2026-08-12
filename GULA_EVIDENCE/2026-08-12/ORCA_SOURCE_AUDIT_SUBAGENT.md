# Orca source audit for Limux comparison

**Status:** Phase 1/2, read-only, source-grounded preliminary findings. This is an evidence packet for the root agent; it is not the final adopt/fork/switch verdict.

**External repository:** `stablyai/orca`
**Pinned audit commit:** [`09ec516ae50b7b83fa65343d9ad96159e3fe71fc`](https://github.com/stablyai/orca/commit/09ec516ae50b7b83fa65343d9ad96159e3fe71fc) (`main`, 2026-08-12 10:24:31Z)
**Collection time:** 2026-08-12 14:09:14Z
**Method:** direct GitHub metadata/API and commit-pinned source reads; official Orca documentation; local Limux source reads for the comparison boundary. Repository content was treated as untrusted data. Nothing was cloned, installed, built, or executed.

## 1. Scope and tool trace

The requested evidence includes architecture, stack, entry points, main data/control flow, build/install/update, permissions/security/sandboxing, telemetry/network, dependencies/provider coupling, test/CI/release health, license, maturity, capabilities Limux could extract, and the mechanics/risks of switching to or forking Orca.

Loaded methodologies in full before investigation:

- `repo-audit`
- `evaluate-repo`
- `deepwiki`
- `firecrawl`
- `firecrawl-search`
- `firecrawl-scrape`
- `github-cli`

Availability and fallback record:

- **GitHub CLI:** available and authenticated; repository-aware commands used explicit `-R stablyai/orca`, and direct API reads used `repos/stablyai/orca/...` with the pinned ref.
- **Hosted/public DeepWiki:** unavailable. `codex mcp list` exposed only `deepwiki-local`; the requested public namespace `mcp__deepwiki__` was absent. Exact probe `codex mcp get deepwiki` returned `Error: No MCP server named 'deepwiki' found.` The local/private DeepWiki server was deliberately not used because cloning/ingestion was out of scope. Mark this `deepwiki-unavailable: confidence REDUCED` for wiki-assisted architectural interpretation.
- **Firecrawl:** its skills were loaded, but no Firecrawl MCP tool was exposed and the CLI probe returned `zsh:1: command not found: firecrawl`. Targeted executable/package-location checks also found no callable Firecrawl surface. Official public pages were therefore read through the ordinary web fetcher as a supplemental fallback. No Firecrawl result is represented as having run.
- **Branch protection API:** `repos/stablyai/orca/branches/main/protection` returned HTTP 404 to the current GitHub identity. This does **not** prove that protection is absent; protection configuration is unverified.

## 2. Pinned repository facts

At collection time GitHub reported:

| Fact | Evidence |
|---|---|
| Purpose | “ADE for working with a fleet of parallel agents,” desktop/mobile/VPS; [repository](https://github.com/stablyai/orca) and [`README.md:18-26`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L18-L26) |
| Default branch / pinned head | `main` / `09ec516…`; [commit](https://github.com/stablyai/orca/commit/09ec516ae50b7b83fa65343d9ad96159e3fe71fc) |
| Popularity snapshot | 43,480 stars and 3,036 forks at 2026-08-12 14:09Z; [repository API](https://api.github.com/repos/stablyai/orca) |
| Activity snapshot | `pushed_at=2026-08-12T13:40:17Z`, `updated_at=2026-08-12T14:09:03Z`; [repository API](https://api.github.com/repos/stablyai/orca) |
| Latest release observed | `v1.4.180`, published 2026-08-11, with macOS x64/arm64, Windows installer, and Linux AppImage/deb/rpm x64/arm64 assets carrying GitHub SHA-256 digests; [release](https://github.com/stablyai/orca/releases/tag/v1.4.180) |
| License | MIT, copyright Lovecast Inc.; [`LICENSE:1-20`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/LICENSE#L1-L20) |
| Work queue snapshot | GitHub Search reported 1,742 open issues and 1,885 open PRs. These are point-in-time indexing counts, not a quality verdict; [issues query](https://api.github.com/search/issues?q=repo%3Astablyai%2Forca+is%3Aissue+is%3Aopen), [PR query](https://api.github.com/search/issues?q=repo%3Astablyai%2Forca+is%3Apr+is%3Aopen) |

The repo is a large, rapidly moving product rather than a small terminal library. The pinned tree includes desktop, mobile, native modules, browser automation, SSH/remote runtime, agent hooks, skills, editor/UI, review, GitHub/Linear, and extensive tests. This breadth is load-bearing when comparing it with Limux: adopting a few contracts and adopting the product are very different decisions.

## 3. Architecture and stack

### 3.1 Top-level shape

The repository contains `src/` (desktop, CLI, main process, preload, renderer and shared code), `mobile/` (separate Expo/React Native workspace), `native/`, `resources/`, `skills/`, `skill-guides/`, `tests/`, `config/`, and `.github/workflows/`. The root workspace explicitly excludes `mobile/` because mobile has its own workspace and lockfile: [`pnpm-workspace.yaml:1-4`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/pnpm-workspace.yaml#L1-L4).

Primary stack:

- Electron desktop shell, TypeScript, React 19, Vite/electron-vite.
- xterm packages with WebGL plus `node-pty` for terminal sessions.
- Zustand-style renderer stores and Zod schemas for runtime/RPC validation.
- Node 24 CI baseline, pnpm 10.24.
- Vitest unit/integration tests and Playwright/Electron end-to-end infrastructure.
- Expo/React Native mobile companion as a separate package graph.
- SSH2, WebSocket, filesystem/Git/provider abstractions, Electron updater, PostHog, native speech libraries, and browser automation.

Primary dependency evidence is in [`package.json:129-152`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L129-L152) and development tooling in [`package.json:154-277`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L154-L277).

### 3.2 Entrypoints

| Surface | Entry | Role |
|---|---|---|
| Electron main | `src/main/index.ts` | Creates app windows; wires settings, runtime, daemon PTY, hooks, account, usage, automation and IPC services. |
| Preload | `src/preload/index.ts` | Large privileged bridge between renderer and Electron/main capabilities. |
| Renderer | `src/renderer/src/main.tsx` | Loads localization/styles/diagnostics and mounts React under an error boundary; [`main.tsx:1-65`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/renderer/src/main.tsx#L1-L65). |
| CLI | `src/cli/index.ts` | Parses and validates command specs, lazy-loads the runtime client, resolves local/remote selection, dispatches, and emits structured output/errors; [`index.ts:34-119`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/cli/index.ts#L34-L119). |
| Persistent terminal daemon | `src/main/daemon/daemon-entry.ts` | Standalone Node process started with socket/token paths; owns PTY subprocesses and survives GUI exit for reattach; [`daemon-entry.ts:1-122`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-entry.ts#L1-L122). |
| Headless/remote runtime | `orca serve` through CLI/runtime modules | Runs the runtime without a desktop window and exposes pairing; [remote server docs](https://www.onorca.dev/docs/remote-servers). |

The package exposes `out/main/index.js` as Electron main and `out/cli/index.js` as the `orca` binary: [`package.json:1-12`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L1-L12).

### 3.3 Main control/data flow

```text
React renderer
  -> preload/Electron IPC
  -> Electron main services
  -> OrcaRuntime mutable live graph
      -> local Git/filesystem/provider contracts
      -> persistent PTY daemon
      -> SSH provider implementations
      -> browser/editor/review/account integrations

orca CLI
  -> command schema + validation
  -> RuntimeClient
  -> local Unix/named-pipe RPC or paired WebSocket runtime
  -> same runtime method registry

mobile/browser/remote client
  -> paired WebSocket RPC
  -> authenticated runtime context
  -> runtime methods and revocable client grant
```

The RPC core defines an authenticated request envelope, context (client identity/capabilities/orchestration authority), Zod-backed method definitions, and a registry: [`src/main/runtime/rpc/core.ts:44-187`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/core.ts#L44-L187). This registry/transport separation is one of the strongest reusable designs for Limux.

The persistent daemon is not a thin helper. Initialization probes and adopts an authenticated surviving daemon, preserves live sessions on uncertain health, falls back to local PTYs in degraded mode, and detaches after readiness so terminals survive Electron exit: [`daemon-init.ts:136-262`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-init.ts#L136-L262), [`daemon-init.ts:424-709`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-init.ts#L424-L709), [`daemon-init.ts:923-1037`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-init.ts#L923-L1037). The README’s user-facing result is scrollback that survives restarts: [`README.md:63-67`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L63-L67).

### 3.4 Architectural concentration

`src/main/runtime/orca-runtime.ts` is 37,800 lines / about 1.44 MB at the pinned commit; its own header disables the line-limit rule and says it centrally owns the mutable workspace graph, PTYs, waiters, mobile/layout/worktree reconciliation: [`orca-runtime.ts:1-10`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L1-L10), class beginning [`orca-runtime.ts:2694`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L2694).

Other unusually large load-bearing files include its 48,618-line test, 4,943-line preload, 3,738-line terminal RPC module, and 3,321-line main entry. The repo acknowledges this debt with a ratchet that grandfathers 347 baseline oversized files while rejecting new growth/bypasses; see [`config/max-lines-baseline.txt`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/config/max-lines-baseline.txt) and [`config/scripts/check-max-lines-ratchet.mjs:7-37`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/config/scripts/check-max-lines-ratchet.mjs#L7-L37). The ratchet is positive control evidence, but the central runtime is still a material comprehension, fork-maintenance, and change-coupling risk.

## 4. Build, installation, and update behavior

### Build and source setup

- Root scripts include lint, reliability/quality gates, max-lines ratchet, type checks, unit tests, desktop/native builds, packaging, and multiple E2E suites: [`package.json:12-127`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L12-L127).
- `postinstall` executes `node config/scripts/rebuild-native-deps.mjs`, so source installation is not metadata-only and runs repository build code: [`package.json:75-77`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L75-L77).
- The package allowlists native builds including Parcel watcher, CPU features, esbuild, `node-pty`, and sherpa; it also patches `node-pty` and beta xterm packages: [`package.json:274-307`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/package.json#L274-L307).
- `.npmrc` uses `minimum-release-age=4320` (three days) and `shamefully-hoist=true`: [`.npmrc:1-2`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.npmrc#L1-L2). The lockfile is large (about 489 KB), consistent with the broad Electron product surface.

No package manager or repository code was run during this audit.

### Distribution and updater

- Project-owned release binaries cover macOS, Windows, and Linux; Homebrew/AUR paths are documented at [`README.md:210-226`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L210-L226).
- Windows builds are documented as signed through SignPath: [`README.md:263-264`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L263-L264).
- Desktop update uses `electron-updater`, generic feeds, manual download/install state, platform recovery, and prerelease handling. The source explicitly warns never to override built-in Authenticode verification with a no-op: [`src/main/updater.ts:2182-2212`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/updater.ts#L2182-L2212); download starts at [`updater.ts:2295`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/updater.ts#L2295).
- The default headless remote updater adapter reports automatic updates unsupported/manual-required until a runtime-specific adapter is configured: [`src/main/runtime/remote-server-updater.ts:14-35`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/remote-server-updater.ts#L14-L35).

## 5. Permissions, security, isolation, telemetry, and network

### 5.1 Positive controls observed

- The main BrowserWindow enables Electron renderer sandboxing, while also enabling webviews: [`src/main/window/createMainWindow.ts:268-306`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/window/createMainWindow.ts#L268-L306).
- Privileged windows deny new-window/navigation inheritance and normalize external destinations: [`src/main/window/privileged-window-navigation.ts:5-31`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/window/privileged-window-navigation.ts#L5-L31).
- Local Unix RPC uses newline-delimited JSON, connection/message/idle caps, and chmod `0600` on the socket: [`src/main/runtime/rpc/unix-socket-transport.ts:1-74`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/unix-socket-transport.ts#L1-L74).
- WebSocket RPC includes pre-auth timeouts, message and connection caps, heartbeats, per-device authentication, and immediate termination on grant revocation: [`src/main/runtime/rpc/ws-transport.ts:1-43`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L1-L43), [`ws-transport.ts:105-118`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L105-L118), [`ws-transport.ts:202-328`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L202-L328).
- Pairing input is schema/length/scheme/host/base64 validated: [`src/shared/pairing.ts:14-98`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/pairing.ts#L14-L98).
- The remote-server guide says each client receives a revocable token and warns not to expose the port directly to the public Internet: [remote-server docs, access/security](https://www.onorca.dev/docs/remote-servers#access-and-security).

### 5.2 Important boundaries and review surfaces

1. **Local worktrees are not execution sandboxes.** The main parallel-agent claim is branch/worktree isolation: [`README.md:49-53`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L49-L53). Agents still run with the local/remote host’s user permissions and credentials. Per-workspace disposable VM/container isolation exists only through the separate experimental/BYO environment mechanism: [ways-to-run docs, Cloud VMs](https://www.onorca.dev/docs/ways-to-run#4-cloud-vms-per-workspace-environments). Do not equate “isolated worktree” with OS/process isolation.

2. **Transport encryption is conditional.** WebSocket source supports `ws` or `wss`; TLS is used when certificate and key are configured, otherwise the server is HTTP/plain WebSocket. Its comment states device authentication is independent of transport encryption: [`ws-transport.ts:1-14`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L1-L14), [`ws-transport.ts:173-200`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L173-L200). The docs compensate operationally by requiring a private path such as Tailscale/LAN and treating pairing links like passwords: [remote-server docs](https://www.onorca.dev/docs/remote-servers#access-and-security). Any Limux adaptation should preserve an explicit private-network or TLS boundary, not copy only token auth.

3. **Embedded browser is a broad surface.** The main privileged window enables `webviewTag`; the guest preference helper observed in this pass is small and only sets a fullscreen-resize preference: [`src/shared/browser-guest-web-preferences.ts:1-3`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/browser-guest-web-preferences.ts#L1-L3). This is a review surface, not a demonstrated vulnerability.

4. **Agent hooks change user-level agent configuration.** Orca’s Codex hook service reads and rewrites managed/system hook configuration and generates a loopback event hook authenticated with a token file: [`src/main/codex/hook-service.ts:174-203`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/codex/hook-service.ts#L174-L203), [`hook-service.ts:331-362`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/codex/hook-service.ts#L331-L362), [`hook-service.ts:635-668`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/codex/hook-service.ts#L635-L668), [`hook-service.ts:772-830`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/codex/hook-service.ts#L772-L830). Claude and other agents have parallel services. This overlaps Limux’s governed hook/runtime-owner boundary and would need an explicit integration policy in any fork.

5. **Account/usage features couple to provider credentials.** Remote docs explicitly require provider CLIs/accounts on the runtime-owning server: [remote-server docs, what runs where](https://www.onorca.dev/docs/remote-servers#what-runs-where). No secret values were accessed in this audit.

### 5.3 Telemetry and outbound services

The official policy says packaged builds send anonymous, enumerated lifecycle/workspace/agent/error/settings events to PostHog US; it denies sending prompts, terminal output, file/repo/branch/path/URL/commit content and provides both UI opt-out and `DO_NOT_TRACK=1` / `ORCA_TELEMETRY_DISABLED=1`: [telemetry policy](https://www.onorca.dev/docs/telemetry).

Source corroborates a consent gate: DNT/product kill switch/CI disable transmission, and missing consent fails closed pending the banner: [`src/main/telemetry/consent.ts:76-109`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/telemetry/consent.ts#L76-L109). The client requires an official build and write key, sends to PostHog US, validates event schemas, and applies consent before capture: [`src/main/telemetry/client.ts:19-36`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/telemetry/client.ts#L19-L36), [`client.ts:66-116`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/telemetry/client.ts#L66-L116), [`client.ts:163-204`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/telemetry/client.ts#L163-L204).

Other expected network surfaces include GitHub releases/update feeds, GitHub and Linear integrations, remote/WebSocket runtime, SSH/SFTP, mobile pairing, browser content, and the user’s agent/provider CLIs. A full endpoint allowlist was not derived in this phase and remains unverified.

## 6. Dependency and provider coupling

The marketing-level provider boundary is intentionally broad: any terminal CLI agent can run, with named support for Claude Code, Codex, Grok, Cursor, Copilot, OpenCode, Pi/OMP, Hermes, Kimi, Qwen and many more: [`README.md:171-205`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L171-L205).

That broad launch compatibility should not be mistaken for zero coupling:

- Account switching, usage/rate-limit tracking, session lifecycle recognition and managed hooks contain agent-specific adapters.
- Remote execution requires the chosen CLI, its authentication and its filesystem/runtime dependencies on the remote owner machine.
- Local and SSH implementations share explicit PTY/filesystem/Git provider contracts, which is a reusable boundary: [`src/main/providers/pty-provider-contract.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/providers/pty-provider-contract.ts), [`filesystem-provider-contract.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/providers/filesystem-provider-contract.ts), [`git-provider-contract.ts`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/providers/git-provider-contract.ts).
- Direct dependencies include the Linear SDK, browser automation, Electron updater, PostHog, SSH2, WebSocket and native PTY/speech modules. A Limux fork/adoption would inherit their update, native-build and privacy surfaces unless deliberately removed.

## 7. Tests, CI, release health, and maturity

### Positive evidence

- PR CI uses read-only contents permission and checkout with credential persistence disabled: [`.github/workflows/pr.yml:15-28`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L15-L28).
- Static gates cover lint, type-aware quality/reliability, max-lines, skills and localization: [`pr.yml:32-94`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L32-L94).
- Type checking and Git compatibility jobs are explicit: [`pr.yml:113-175`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L113-L175).
- Shell contracts exercise bash/zsh/fish and real PTY behavior: [`pr.yml:176-249`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L176-L249).
- The test matrix spans Node 24/26 and 16 shards, with separate cross-version wire and Node 18 managed-hook compatibility jobs: [`pr.yml:251-332`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L251-L332).
- The tree contains dedicated Windows/Linux/macOS E2E/update workflows, daemon/PTY recovery tests, transport tests, mobile tests and benchmark/fuzz suites.
- Releases are frequent and multi-platform. `v1.4.180` was published one day before this snapshot and assets carry GitHub-calculated SHA-256 digests: [release](https://github.com/stablyai/orca/releases/tag/v1.4.180).

### Cautions

- No local build or test execution was authorized; this packet validates source/CI structure, not the pinned commit’s runtime behavior.
- `action_required` on external-contributor GitHub runs was observed, but that state is commonly an approval gate and was not treated as a test failure.
- The 1,742/1,885 open issue/PR counts show exceptional queue/activity scale but do not independently establish defect rate, responsiveness or merge quality.
- Branch protection/signature requirements could not be read with current permissions.
- Daily release cadence is strong activity evidence and also implies high ongoing rebase/cherry-pick cost for a long-lived fork.

**Preliminary maturity characterization:** production-scale, actively shipped and unusually well tested, with substantial operational hardening around PTY persistence, updates and cross-platform behavior. Its dominant maintenance risk is breadth plus concentrated runtime complexity, not lack of activity.

## 8. Concrete Orca capabilities Limux could extract

These are source-grounded candidates, ordered by likely value-to-disruption rather than a final roadmap.

| Candidate | What to extract (not necessarily code-copy) | Limux fit | Main caveat | Evidence |
|---|---|---|---|---|
| Typed RPC registry | Co-locate method name, Zod/typed params, handler, auth/capability context; generate CLI/help/compat surfaces from it. | High: Limux already has protocol/core/control/CLI crates and partial bridge registry. | Translate to Rust/Serde; do not import Electron runtime. | [`rpc/core.ts:44-187`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/core.ts#L44-L187) |
| Cross-version wire compatibility | Explicit protocol versions/capabilities plus CI exercising old/new wire combinations. | High for stable/preview lanes and future remote clients. | Needs a declared Limux compatibility contract first. | [`AGENTS.md:47-49`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/AGENTS.md#L47-L49), [`pr.yml:288-309`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml#L288-L309) |
| Transport hardening | Size/connection/idle limits, pre-auth deadline, heartbeats, revocation disconnect, local socket `0600`. | High as contracts/tests even if Limux stays local-only. | Remote transport adds security/product scope; keep local design smaller until authorized. | [`unix-socket-transport.ts:1-74`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/unix-socket-transport.ts#L1-L74), [`ws-transport.ts:1-43`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/rpc/ws-transport.ts#L1-L43) |
| CLI command-schema discipline | Parse/validate before connection, lazy runtime loading, explicit local/remote selection and JSON errors. | High; Limux’s CLI already drives a socket and has known flag-placement drift. | Avoid copying Orca’s huge command inventory. | [`src/cli/index.ts:34-167`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/cli/index.ts#L34-L167), [CLI reference](https://www.onorca.dev/docs/cli/reference) |
| Persistent PTY host and scrollback checkpoints | A UI-independent daemon owns PTYs/history, can be adopted across GUI restarts, and preserves live sessions under uncertainty. | Potentially high; directly addresses durable agent sessions. | Ghostty surface lifecycle differs from xterm/node-pty; this is a substantial architecture project. | [`daemon-init.ts:424-709`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-init.ts#L424-L709), [`README.md:63-67`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L63-L67) |
| Provider contracts for local/SSH | Filesystem/Git/PTY interfaces shared across local and SSH implementations. | Medium-high if Limux pursues remote execution. | SSH credentials, reconnection and remote source delivery expand threat/support surface. | [`src/main/providers`](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/providers), [`README.md:105-109`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L105-L109) |
| Revocable paired clients | Per-client grants, pairing validation and immediate revocation disconnect. | Medium if Limux adds mobile/remote. | Requires TLS/private-network policy, secure grant storage and protocol lifecycle. | [`pairing.ts:14-98`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/pairing.ts#L14-L98), [remote security docs](https://www.onorca.dev/docs/remote-servers#access-and-security) |
| Agent-driven diff review | Line annotations routed back to the agent, review/edit/commit loop. | Medium-high product value; complements Limux’s durable review ledger. | Requires a real Git diff/editor UX, not just a terminal protocol. | [`README.md:119-127`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L119-L127) |
| Design Mode | Capture selected DOM/CSS/screenshot from a real browser into agent context. | Medium; Limux has WebKitGTK browser plans/partial support. | Large browser security and context-sanitization surface; concept first, not source copy. | [`README.md:77-85`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L77-L85) |
| Mobile/remote steering | Notifications, follow-ups and shared persistent sessions from phone/browser. | Long-term differentiator. | Requires remote runtime, pairing, transport, lifecycle and mobile product; not a small feature. | [`README.md:35-43`](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/README.md#L35-L43), [ways to run](https://www.onorca.dev/docs/ways-to-run) |
| Test scenarios | Cross-shell real PTY checks, restart/update survival, backpressure, Unicode, history restore, cross-wire and remote reconnect cases. | High and low-risk: adapt scenarios to Rust/Ghostty. | Avoid assuming xterm/node-pty expected bytes exactly match Ghostty. | [PR workflow](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/.github/workflows/pr.yml), [`src/main/daemon`](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon) |

The most defensible first extractions are **contracts and test ideas**—typed registry, wire compatibility, transport bounds, CLI schema discipline—because they strengthen Limux without replacing its native architecture. Persistent PTY and remote/mobile designs are potentially valuable but are full product/architecture tracks.

## 9. What a switch to Orca would mean

This section records consequences, not a final recommendation.

### Stock Orca instead of Limux

- Users would gain a much broader integrated product immediately: parallel worktrees, editor/diff/review, embedded Chromium Design Mode, GitHub/Linear, SSH, persistent PTY daemon, mobile, remote runtime and a large automation CLI.
- They would leave Limux’s Rust/GTK/libadwaita/Ghostty Linux-native stack for Electron/React/xterm/node-pty and accept the associated dependency, memory, native-module, updater, telemetry and browser surfaces.
- It is not a drop-in engine substitution. Limux’s control protocol, Ghostty surface IDs, stable/preview profile isolation, hcom bus boundary and Linux-native UI would not automatically exist in stock Orca.
- The simplest empirical product evaluation would be running an official Orca build alongside Limux in a non-sensitive test environment. This audit did not install or run it.

### A maintained Orca fork

- MIT permits use/modification/distribution subject to retaining the license/copyright notice; third-party dependencies/assets still require their own inventory.
- A fork can add Limux-specific behavior, but the upstream is extremely fast-moving and centrally coupled. Regular rebasing/cherry-picking against daily releases and a 37.8k-line runtime would be a standing engineering lane, not a one-time migration.
- Removing rather than merely disabling Orca’s telemetry, account management, remote server, mobile, browser, updater or managed-hook surfaces would touch many modules and tests.
- Porting Ghostty into the Electron product is not a straightforward copy: Orca’s terminal persistence/serialization is designed around node-pty/xterm and its daemon/headless emulator. A Ghostty terminal backend would cross daemon protocol, renderer, preload, native packaging and restore logic.
- A narrower integration is structurally possible: preserve Orca’s UI/review/runtime and add a Limux/hcom adapter or external runtime provider. Whether its provider contracts are sufficient for a Ghostty/GTK external host was not established in this phase.

## 10. Limux capabilities worth carrying into an Orca fork

These Limux facts come from the current local checkout and show what would otherwise be lost.

| Limux capability | Why it is distinctive/useful in an Orca-derived product | Current evidence |
|---|---|---|
| Native Linux terminal stack | GTK4/libadwaita plus embedded `libghostty.so` provides a Linux-native UI and Ghostty renderer instead of Chromium/xterm. | `Cargo.toml:1-25`; `README.md:548-560` |
| Orthogonal runtime channel and session profile | Stable/preview lanes and named profiles compose into separate socket/state paths, preventing two builds from sharing live state. | `rust/limux-control/src/socket_path.rs:35-151`; `rust/limux-control/src/session_paths.rs:1-57` |
| Build/install provenance and doctor | Installed version includes source SHA/profile/channel/install ID; `doctor` checks launcher, process, socket and Ghostty-resource drift; stable promotion uses a recorded checklist. | `README.md:76-112`, `README.md:250-305` |
| Explicit Limux/hcom boundary | Limux owns local GUI panes/notifications/screen reads; hcom owns named agent messaging, transcript, resume/fork and multi-project coordination. | `README.md:399-422` |
| Visible in-pane agent teams | Agents can be launched directly or through `hcom --run-here` while inheriting exact workspace/pane/surface/tab/socket identity. | `README.md:307-369` |
| Durable, no-clobber collaboration artifacts | Generated `LIMUX_AGENTS.md` points to authoritative instructions with hashes, and roster/ledger outputs refuse unsafe overwrite/symlink shapes. | `README.md:371-397`; `docs/cmux-parity-plan.md:123-184` |
| Explicit terminal-input boundary | Control text and control keys use separate routes, and generated bootstrap text has control/display-spoofing guards. | `docs/cmux-parity-plan.md:194-219` |
| Runtime/channel installer isolation | Stable, preview, named preview and rollback aliases carry `install-info.json`, so testing cannot silently replace the daily driver. | `README.md:250-269` |

Potential transplant shapes, from least to most invasive:

1. Add an Orca runtime/CLI adapter that preserves the **hcom vs local GUI bus** distinction and carries stable hcom identity plus pane/surface context.
2. Bring Limux’s build identity, `doctor`, stable/preview channel and per-profile namespace contracts into an Orca fork’s updater/runtime selection.
3. Preserve Limux’s no-clobber generated protocol/roster/review-ledger semantics as an alternative orchestration mode.
4. Adapt Limux’s input-channel split and spoofing guard to Orca’s agent automation APIs.
5. Treat Ghostty/GTK as a separate Linux runtime provider or companion host before considering an in-process Electron terminal-backend rewrite.

The local evidence also shows overlap: Limux already has hook installation and an embedded browser, so “bring unique things over” should focus on the governed hcom boundary, channel/profile isolation, provenance/doctor, no-clobber coordination files and Ghostty-native host—not hooks/browser in the abstract.

## 11. Preliminary source-grounded comparison

| Dimension | Orca | Limux implication |
|---|---|---|
| Product breadth | Full agent development environment: worktrees, editor/diff, browser, integrations, SSH/remote/mobile. | Source of high-value UX and protocol ideas; wholesale parity would radically expand scope. |
| Terminal | xterm/WebGL + node-pty + persistent daemon/history. | Daemon/restore/test contracts are valuable; renderer/backend code is not directly portable to Ghostty. |
| Host/UI | Electron/React, cross-platform. | Switching sacrifices the current native GTK/libadwaita architecture. |
| Control API | Large typed RPC registry over local and paired remote transports; broad CLI. | Strongest design source for consolidating Limux bridge/CLI metadata. |
| Agent coordination | Worktree-centric fleet UI, provider-specific hooks/accounts, mobile/remote. | Complementary to Limux+hcom; must avoid competing ownership/session buses. |
| Isolation | Git worktree isolation by default; experimental BYO VM/container mode for stronger isolation. | Do not import “isolated” wording as a security guarantee. |
| Security/privacy | Renderer sandbox, validation/caps, revocable clients, optional TLS/private-network guidance, consented PostHog telemetry. | More exposed surfaces than local Unix-socket Limux; each requires an explicit policy decision. |
| Maintenance | Very active, broad CI/release machinery, large contributor/work queues, large central runtime. | Easy to learn from; expensive to carry as a diverging fork. |
| Licensing | MIT. | Code reuse is legally plausible with notices, subject to separate dependency/asset review. |

## 12. Open questions and unverified claims

These should remain open rather than being silently inferred:

- Runtime performance/resource use versus Limux; neither binary was run or benchmarked.
- Whether official Orca Linux builds fully satisfy this machine’s GTK/Ghostty workflows, accessibility, keyboard and WSL requirements.
- Exact endpoint/domain allowlist for every GitHub, Linear, update, telemetry, mobile, SSH and browser path.
- Complete threat model and independent security history; no advisory/code-audit wave was performed.
- Current branch-protection/review/signature enforcement, because the API was not readable.
- Whether an external terminal/runtime provider can preserve Orca UI while delegating PTYs/rendering to Limux without a major fork.
- Third-party license/notice compatibility for copied assets, patched xterm/node-pty code or mobile dependencies. MIT at repository root is not a substitute for a dependency/asset inventory.
- Whether the very high open issue/PR counts reflect deliberate automation/community throughput, backlog, mirrors or spam; the counts alone decide nothing.
- Full parity between documented telemetry fields and every packaged build path. Source inspected supports the main consent/client path, not every possible outbound integration.

## 13. Evidence integrity

- External source links are commit-pinned wherever GitHub line evidence is used.
- Official documentation links are live pages and may drift after 2026-08-12.
- GitHub stars/forks/issues/PRs/releases are point-in-time values and will drift.
- No execution evidence is claimed. Findings labeled architecture, maturity or fit are derived from source/document structure, not empirical runtime acceptance.
- This packet intentionally stops at preliminary findings. The root agent owns the final Limux recommendation, prioritization and any authorized implementation plan.
