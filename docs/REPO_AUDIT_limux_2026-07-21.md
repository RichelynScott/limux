# Limux Repository Audit and Improvement Plan

**Audit date:** 2026-07-21  
**Repository:** `/home/riche/MCPs/limux`  
**Audited head:** `31a9431` (`origin/main`)  
**Authoring model:** `gpt-5.6-sol` (`high`), verified from hcom transcript binding at write time  
**Method:** `$repo-audit`, composed with `$prime`, `$hcom-project-reconciliation`, `$directory-cleanup`, `$dirty-worktree-owner-cleanup`, and `$taskmaster-reviewed`

## Executive Summary

Limux is **B-grade product code inside D-grade repository hygiene; overall grade C+**. The mainline is real, testable software rather than a prototype: the canonical gate passed on this audit run with 597 tests, clippy with warnings denied, formatting, and both packaging validators. The strongest engineering is around runtime identity, socket/request boundaries, agent integration tests, and stable/preview channel isolation. The top three risks are: a confirmed `read-screen --help` cross-pane disclosure path, release workflows that execute mutable or unverified supply-chain inputs, and a dual control architecture whose live GTK bridge intentionally remains only partially equivalent to the standalone dispatcher. Operationally, three open PRs are conflicting; two contain useful features plus unresolved exact-head P2 review findings, while the third mixes obsolete TaskMaster state with a valuable handoff rewrite. The repository also carries 88 local branches, a retained release worktree, six competing handoffs, a 113 KiB FYI journal, and an 18 GiB checkout dominated by a 16 GiB ignored build directory. The best near-term opportunities are to close the CLI disclosure, reconcile rather than blindly merge the three PRs, pin the release supply chain, and establish one Limu-owned current-state surface while preserving historical handoffs. Large-file decomposition should follow only after these safety and governance items, because the current tests provide a credible refactoring net.

## Repo Map

### Purpose and maturity

Limux is a Linux terminal workspace manager with GPU-rendered Ghostty terminals, GTK4/libadwaita UI, WebKitGTK browser panes, stable/preview runtime channels, a Unix-socket automation API, and agent orchestration for Codex, Claude, Gemini, OpenCode, Hermes, and hcom (`README.md:1-24`). It is an actively dogfooded pre-1.0 desktop product: the stable channel and diagnostic surface are mature enough for daily use, while bridge parity, browser automation, and crash/resource work remain explicitly partial (`docs/cmux-parity-plan.md:46-108`).

### Stack and architecture

| Area | Role | Evidence |
|---|---|---|
| `rust/limux-protocol` | Shared JSON envelopes and restricted-method manifest | `rust/limux-protocol/src/lib.rs:100-190` |
| `rust/limux-core` | In-process state model and standalone dispatcher | `docs/cmux-parity-plan.md:46-60` |
| `rust/limux-control` | Unix socket resolution, peer authorization, framing, server | `rust/limux-control/src/auth.rs:13-68`; `rust/limux-control/src/request_io.rs:7-47` |
| `rust/limux-host-linux` | GTK host, panes, terminal FFI, live control bridge, persistence | `AGENTS.md:15-24`; `docs/cmux-parity-plan.md:54-65` |
| `rust/limux-cli` | User CLI, diagnostics, hooks, agent-team and review workflows | `README.md:76-112` |
| `ghostty/` | Read-only vendored submodule and embedded renderer source | `AGENTS.md:161-161`; `README.md:150-169` |
| `scripts/` | Local quality gate, packaging, install, Xvfb and resource validation | `scripts/check.sh:1-15`; `AGENTS.md:51-72` |
| `.taskmaster/` | Durable multi-tag project plan | `LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:33-38` |
| `docs/`, `FYI.md`, `*_HANDOFF.md` | Product plans, verification records and session continuity | `LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:24-31` |

### Main data/control flow

The installed CLI resolves a channel-specific Unix socket, connects to either the embedded GTK bridge or standalone server, sends JSON commands, and renders JSON or human output. Same-user peer credentials are checked at connection time; request frames are capped at 1 MiB, connections at 64, and idle time at 300 seconds (`rust/limux-control/src/auth.rs:50-102`; `rust/limux-control/src/request_io.rs:7-47`). GTK-side operations are queued onto the host main loop, while the standalone path operates on `ControlState`. This split is both a deliberate compatibility layer and the repo's largest architectural source of drift (`docs/cmux-parity-plan.md:46-86`).

### Verified live snapshot

These facts were rechecked with live commands during the audit, not inferred from prose:

- `origin/main` and the audited branch began at `31a9431`; `git fsck --no-dangling` and `git diff --check` passed.
- `./scripts/check.sh` passed: 597 tests, clippy with `-D warnings`, and formatting. Both packaging validators passed.
- Stable `limux-cli 0.2.3` at source `1a26bda0` reported `doctor.ok=true`; the launcher/release state is also recorded at `LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:17-22`.
- Open PRs #58, #67 and #68 are all `CONFLICTING`/`DIRTY`, matching the succession inventory at `LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:40-44`.
- The checkout measured 18 GiB: `target/` 16 GiB, `ghostty/` 1.5 GiB, `archive/` 479 MiB, `.git/` 127 MiB, and `logs/` 8.3 MiB. `target/`, `logs/`, `archive/`, `.worktrees/`, and `WORKTREES/` are ignored.
- There are 88 local branches: 23 merged into `origin/main`, 65 not merged by graph identity. No branch was deleted during this audit.
- The sole secondary worktree, `/tmp/limux-release-0.2.3-20260719`, is clean and its head is an ancestor of current main, but it remains retained by an explicit no-loss instruction (`LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:46-49`).

### Coverage disclosure

Read deeply: operating instructions, README, manifests/lockfile shape, all CI workflows, quality/install surfaces, socket authorization/framing, control registry, runtime visibility/tick logic, CLI `read-screen` dispatch, TaskMaster active/master/resource-crash summaries, all handoffs, open-PR metadata/comments, and the three conflicting branch histories. Sampled: the large GTK window/pane implementation, core dispatcher, release scripts, Cursor integration, and detailed verification docs. Skipped: vendored Ghostty internals, exhaustive line-by-line review of all 55,200 Rust lines, all 88 local branches, full threat modeling of browser/WebKit behavior, and current CVE enumeration. Dependency vulnerability status is therefore **not asserted**.

## Audit Report

### THE UGLY — fix before new feature expansion

1. **H1 — `read-screen --help` can read another focused pane.** The CLI builds a read request without recognizing `--help`, without rejecting unknown flags, and without requiring an explicit workspace or surface; it then calls `surface.read_text` (`rust/limux-cli/src/main.rs:4984-5014`). A black-box incident independently observed the command returning another agent's focused screen (`LIMU_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md:89-100`). This is a same-user cross-lane information disclosure, not merely a help-text defect. **Severity: High. Size: S.**
2. **H2 — release workflows execute mutable or unverified supply-chain inputs.** Actions are referenced by movable tags, Blueprint Compiler is cloned by a tag rather than a commit, and a `continuous` AppImage executable is downloaded and run without a pinned digest (`.github/workflows/rust-quality.yml:14-22`; `.github/workflows/release-linux.yml:64-87`; `.github/workflows/release-linux.yml:95-106`). The RPM workflow repeats the mutable Blueprint path (`.github/workflows/release-rpm.yml:64-75`). A compromised tag or replaced continuous asset can affect signed/published deliverables. **Severity: High. Size: M.**
3. **H3 — production and test control paths are intentionally non-equivalent.** The standalone dispatcher supports the full vocabulary while the live GTK bridge supports a subset and still lacks browser parity (`docs/cmux-parity-plan.md:46-65`). Phase 2 is explicitly partial (`docs/cmux-parity-plan.md:67-108`). Tests against the standalone path can therefore pass while live user behavior remains absent or different. **Severity: High. Systemic. Size: XL; must be broken down.**
4. **H4 — all open PRs are conflicted, and the two product PRs retain exact-head P2 findings.** #58 mixes TaskMaster, multiple handoffs, FYI, skills and verification docs; #67 conflicts in TaskMaster and `window.rs`; #68 conflicts in TaskMaster, CLI doctor/main, and host main. The current succession record identifies all three as dirty (`LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:40-44`). Live review inspection found an exact-head restrictive-socket-mode P2 on #67 and an exact-head stderr-fd P2 on #68. Blind rebase/merge would combine stale task state with unresolved correctness defects. **Severity: High. Operational/systemic. Size: L.**

### Architecture and design

#### M1 — three god files dominate core behavior

**[FACT]** `window.rs` is 9,675 lines, CLI `main.rs` is 9,552, and `limux-core/src/lib.rs` is 8,198. They combine parsing, orchestration, state, UI wiring, I/O and large in-file test modules. This contradicts the repo's own small-domain-module and pure-logic separation rules (`docs/maintainability.md:19-37`). Concrete consequence: any PR touching shared dispatch or GTK window behavior has a high merge-conflict surface, as #67/#68 already demonstrate. **Severity: Medium. Systemic. Size: XL.**

#### M2 — partial bridge parity creates duplicated business rules

**[FACT]** The architecture requires behavior to be represented in the standalone dispatcher, GTK bridge, CLI parser, and registry; the parity plan acknowledges only read/live subsets and deferred methods (`docs/cmux-parity-plan.md:46-108`). **[JUDGMENT]** The canonical method registry is the right direction, but routing and validation remain too distributed. Done means one declarative method contract drives capabilities, validation, dispatch ownership and parity tests. **Severity: Medium, elevated to High for new automation methods. Size: XL.**

### Code quality and correctness

#### M3 — command parsing is hand-rolled and uneven

**[FACT]** `new-pane` and `close-surface` explicitly make help side-effect-free, but `read-screen` does not (`rust/limux-cli/src/main.rs:4960-4981`; `rust/limux-cli/src/main.rs:4984-5014`). The README itself warns that JSON flag placement is uneven (`README.md:103-105`). This inconsistency caused H1 and makes each new command responsible for remembering its own unknown-flag and help rules. **Severity: Medium beyond H1. Systemic. Size: L.**

#### M4 — source tests are embedded in already oversized production units

**[FACT]** The repository explicitly says to move tests out when they obscure the main codepath (`docs/maintainability.md:32-37`), yet the largest modules carry hundreds of inline tests. The tests are valuable; their placement inflates merge surfaces and navigation cost. **Severity: Medium. Size: L, incremental.**

### Security

#### Strength: sensible local-socket and request boundaries

The default socket mode is same-local-user, peer identity comes from Linux `SO_PEERCRED`, and owner-only modes require same UID (`rust/limux-control/src/auth.rs:13-68`). Request size, concurrent connections and idle duration are bounded (`rust/limux-control/src/request_io.rs:7-47`). The restricted Cursor surface has a shared method manifest and server-side enforcement. Preserve these controls.

#### M5 — the unrestricted default trusts every process under the same UID

**[FACT]** `LocalUser` is the default, and it authorizes any same-UID process (`rust/limux-control/src/auth.rs:24-42`; `rust/limux-control/src/auth.rs:62-67`). In this environment, multiple AI agents share a Unix user, so OS-user equivalence is not lane equivalence. H1 shows how easily a benign parser fallback can cross agent lanes. This is an accepted local-automation trade-off today, not proof of remote exposure, but the user-facing security contract is under-documented. Prefer the existing `LimuxOnly` boundary for contexts where it is compatible; do not invent a new credential system without separate authorization. **Severity: Medium. Size: M for policy/docs/tests.**

#### H2 — release provenance gap

See THE UGLY. There were no hardcoded credential values found in the reviewed first-party surfaces. Secret values and `.env` contents were not opened.

### Testing

#### Strength: broad, enforced behavior tests

The canonical script runs boundary lint, formatting, clippy with warnings denied, and all workspace tests (`scripts/check.sh:1-15`). CI runs this gate on pushes to main and PRs and validates packaging assets (`.github/workflows/rust-quality.yml:1-53`). This audit observed 597 passing tests, including socket round trips, restricted methods, runtime lifecycle, terminal input, agent hooks, layout restore and GTK-side pure logic.

#### M6 — no measured coverage threshold

**[FACT]** The canonical gate and CI contain no `llvm-cov`, tarpaulin, Codecov, or equivalent coverage measurement (`scripts/check.sh:1-15`; `.github/workflows/rust-quality.yml:42-53`). A large test count does not show which branches of the 55,200-line Rust surface remain untested. Add reporting first; gate only after establishing a realistic baseline. **Severity: Medium. Size: M.**

#### M7 — live-runtime integration is not part of normal PR CI

**[FACT]** CI runs unit/workspace tests and packaging validators, while the maintained Xvfb smoke is documented as a separate local command (`AGENTS.md:51-72`; `.github/workflows/rust-quality.yml:42-53`). This leaves live GTK/socket behavior dependent on manual or special-lane evidence. Add a bounded headless smoke job after stabilizing runtime and Ghostty caching. **Severity: Medium. Size: L.**

### Performance and resource behavior

#### Strength: hidden-surface work is deliberately bounded

The terminal visibility state coalesces map/unmap/reparent changes and applies Ghostty occlusion (`rust/limux-host-linux/src/terminal.rs:700-755`). The app tick path changes cadence based on visible surfaces rather than spinning unconditionally (`rust/limux-host-linux/src/terminal.rs:920-950`). Resource-crash TaskMaster work therefore builds on real controls rather than starting from zero.

#### M8 — repo-local build and log growth lacks an operator-friendly reclamation policy

**[FACT]** The audit measured 16 GiB in ignored `target/` and 8.3 MiB in ignored `logs/`; the largest log was `logs/chat.json` at 8.3 MiB. The repo correctly forbids committing build outputs (`docs/maintainability.md:19-25`) but does not provide a no-loss, policy-compliant disk-reclamation runbook. Because the global operating contract forbids deletion by default, ordinary `cargo clean` is operator-gated here. **Severity: Medium operational. Size: S for policy, potentially slow for execution.**

### Dependencies and CI/CD

#### Strength: compact Rust dependency surface and committed lockfile

The six-crate workspace uses mainstream Rust dependencies and a committed `Cargo.lock`. The highest-complexity external component, Ghostty, is a pinned submodule and built explicitly in CI (`.github/workflows/rust-quality.yml:14-45`). No dependency vulnerability conclusion is made because a current advisory scan was outside this audit.

#### H2 — mutable release inputs

See THE UGLY. Pin actions by commit SHA, pin downloaded tools by immutable release plus digest, and verify before execution.

### DevEx, operations and repository hygiene

#### M9 — continuity state is fragmented and current truth is not discoverable from `HANDOFF.md`

**[FACT]** The root handoff is stale Halo-era content while runtime truth lives in `HAMO_HANDOFF.md`; six session files compete for attention, and a better canonical rewrite is stranded on PR #58 (`LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md:24-31`). A zero-context successor can easily restart completed release work or miss active incidents. **Severity: Medium, high during recovery. Size: M.**

#### M10 — branch and PR inventory is too large for routine ownership reasoning

**[FACT from live Git inventory]** 88 local branches, 65 not graph-merged into current main, three conflicting open PRs, and a retained `/tmp` worktree create high operator cost even though the primary checkout is now controlled. Previous FYI entries show this has recurred (`FYI.md:1996-2018`). Do not bulk-delete; classify branches as merged, superseded, unique-unlanded, or evidence-retained, then act only with explicit disposition. **Severity: Medium. Size: L.**

### Documentation

#### M11 — README release examples are stale at 0.2.2

**[FACT]** The workspace and live stable runtime are 0.2.3, but install commands, version examples and header examples still say 0.2.2 (`README.md:26-39`; `README.md:89-97`; `README.md:114-121`). This makes users install or expect an older release and undermines the otherwise strong runtime identity story. **Severity: Medium. Quick win S.**

#### L1 — useful history has become an onboarding obstacle

**[FACT]** `FYI.md` is over 2,000 lines and still appends current changes after multiple prior reconciliation episodes (`FYI.md:1967-2042`). Historical content is valuable, but a current index and archive boundary are needed so successors do not have to read the journal linearly. **Severity: Low. Size: M under archive-first condensation.**

### Runtime/product validation

#### M12 — the new-pane incident is unresolved for the current stable build

**[FACT]** The incident reproduced inert newly created terminal panes and ambiguous timeouts twice on legacy 0.2.2 (`LIMU_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md:13-74`). The reporter explicitly did not claim it affects other builds (`LIMU_INBOX/INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md:95-100`). Main has pane readiness/timeout tests, so the correct action is a bounded v0.2.3 retest before opening a new implementation lane. **Severity: Medium, flagged-not-asserted on v0.2.3. Size: S verification.**

#### M13 — active OMP output reportedly yanks scrollback and flashes

**[FACT]** The operator reports that an OMP session inside Limux repeatedly jumps the viewport to the bottom and flashes, preventing scrollback reading (`LIMU_INBOX/BUG_FROM_tutu_2026-07-21_omp-pane-scroll-yank-flash.md:5-12`). The adjustment callback directly turns GTK value changes into Ghostty `scroll_to_row` actions, while scrollbar synchronization has a separate redundant-update guard (`rust/limux-host-linux/src/terminal.rs:1089-1141`; `rust/limux-host-linux/src/terminal.rs:1856-1862`). The header samples every second and gives hcom queries a two-second timeout (`rust/limux-host-linux/src/header_status.rs:12-13`), but a causal header-to-scrollbar link is **flagged, not asserted**. Git history shows scrollbar commit `fc23ac2` predates local header PR #59; reproduction must determine whether terminal output, adjustment feedback, layout, or header refresh is the trigger. **Severity: High operator impact, root cause unverified. Size: M investigation/fix.**

### Strengths to preserve

- Mainline is green under a meaningful local and CI gate, with strict clippy and broad behavior tests (`scripts/check.sh:1-15`; `.github/workflows/rust-quality.yml:42-53`).
- Runtime identity and stable/preview isolation are user-visible and documented (`README.md:76-112`; `README.md:189-214`).
- Socket authorization, request limits, retry-safety metadata and restricted-method enforcement show strong boundary thinking (`rust/limux-control/src/auth.rs:13-68`; `rust/limux-control/src/request_io.rs:7-47`).
- The codebase has real durability work: atomic lifecycle/session state, unclean-restore suspension and no-follow protections are covered by tests.
- Maintainers document known limitations instead of overstating parity (`docs/cmux-parity-plan.md:46-108`).
- The inherited Task 29 work was a proper TDD RED checkpoint rather than an unverified patch, and is now durably preserved on `limu/pane-reflow-task29-20260721` at `7e0eb07`.

## Improvement Strategy

### Theme 1 — make command safety structural, not per-command memory

**Target state:** one parser/command metadata layer handles help, unknown flags, explicit-target requirements and side-effect classification before any socket contact.  
**Principle:** informational flags must fail closed and never perform a read or mutation.  
**Done signals:** every command has a side-effect-free help test; `read-screen --help` cannot contact a host; read commands have explicit and documented focus-fallback policy.

### Theme 2 — reconcile active work before expanding the roadmap

**Target state:** zero conflicted open PRs; every useful unlanded change is either ported to a fresh `limu/` branch from current main or explicitly closed as superseded.  
**Principle:** preserve content, not stale branch topology or obsolete TaskMaster bytes.  
**Done signals:** #58/#67/#68 each have a final disposition; exact-head review findings are fixed or recorded as accepted risk; TaskMaster status matches merged/runtime evidence.

### Theme 3 — secure release provenance with existing boundaries

**Target state:** all third-party actions and executable tools are immutable and hash-verified before execution.  
**Principle:** release builds should consume reviewed identities, not mutable names.  
**Done signals:** actions pinned by SHA with version comments; Blueprint Compiler source pinned by commit; appimagetool pinned to a release and digest; release smoke proves the same artifacts.

### Theme 4 — reduce the monolith and parity tax incrementally

**Target state:** command contracts, pure validation, GTK routing and rendering concerns live in domain modules; the live bridge and standalone dispatcher share one registry/contract.  
**Principle:** use the current tests to carve seams, not rewrite.  
**Done signals:** no production Rust module over an agreed threshold (initially 5,000 lines); new commands require one metadata/validation declaration; parity tests cover every advertised method.

### Theme 5 — make current truth obvious and history cheap

**Target state:** `LIMU_HANDOFF.md` is the manager-owned resume source, `LIMU_INBOX/` is the active inbound queue, root `HANDOFF.md` is either an owned pointer or clearly historical, and old session files remain preserved/indexed.  
**Principle:** one current entry point, many immutable historical records.  
**Done signals:** a cold session reaches current runtime, open PRs, active TaskMaster tag, dirty paths and next action in under five minutes.

### Deliberate non-goals

- Do not rewrite GTK, the CLI, or the dispatcher wholesale; regression risk exceeds near-term payoff.
- Do not delete 65 unmerged branches based only on graph state; several are evidence or superseded-squash branches.
- Do not remove the retained release worktree or 16 GiB build cache without the explicit no-loss/destructive gate.
- Do not create a new token/credential architecture for local socket separation; first evaluate the existing `LimuxOnly` mode and explicit-target rules.
- Do not merge PR #67/#68 merely to make the PR list clean. Useful code must be ported with current-main tests and exact-head findings resolved.

## Task Plan

### Quick wins

| ID | Task | Effort | Impact |
|---|---|---:|---|
| Q1 | Make `read-screen --help` informational and reject unknown flags before socket contact | S | Closes H1 immediately |
| Q2 | Update README 0.2.2 examples to 0.2.3/current-release placeholders | S | Removes visible doc drift |
| Q3 | Publish Limu-owned handoff/inbox index without rewriting historical handoffs | S | Makes succession durable |
| Q4 | Close #58 after porting only the still-valid canonical handoff/attestation content | S-M | Removes the mixed obsolete PR |

### Milestone 0 — Safety net and reconciliation

| ID | Task | Files/areas | Acceptance criteria | Effort | Change risk | Dependencies |
|---|---|---|---|---:|---|---|
| T0.1 | Fix help/unknown-flag safety for read commands | `rust/limux-cli/src/main.rs`, CLI tests | RED test proves no socket contact; `read-screen --help` and `capture-pane --help` print help; unknown flags fail; canonical gate passes | S | Low | None |
| T0.2 | Complete or explicitly park Task 29 | `terminal.rs`, TaskMaster master #29, branch `limu/pane-reflow-task29-20260721` | RED helper tests turn green; root cause documented; targeted and full gates pass; live resize smoke recorded before merge | M-L | Medium | T0.1 optional |
| T0.3 | Reconcile PR #58 | Handoff/attestation/skills docs only | Current content compared path-by-path; obsolete tasks/FYI excluded; valid content ported on fresh branch; #58 closed as superseded with pointer | M | Low-Medium | Manager handoff decision |
| T0.4 | Reconcile PR #67 | Renderer diagnostics, preview script, TaskMaster resource task 2 | Port from current main; fix restrictive socket-mode P2; run script tests/canonical gate/preview matrix; exact-head review clean | L | Medium-High | Operator confirms feature remains wanted |
| T0.5 | Reconcile PR #68 | Host logging, doctor log reader, TaskMaster resource task 3 | Port from current main; fix fd-2 P2; bounded log tests and real preview pass; incident log preserved; exact-head review clean | L | High | Operator confirms feature remains wanted |
| T0.6 | Retest Reve new-pane incident on stable 0.2.3 | Stable runtime only, no source initially | Two explicit-target creates execute command, become writable, return non-ambiguous result; evidence recorded; failure opens focused defect | S | Medium runtime | Operator-approved live pane test window |
| T0.7 | Reproduce and fix OMP scroll-yank/flash | `terminal.rs`, header/layout interaction, TaskMaster master #32 | Cadence and trigger proven; RED regression added; operator can scroll up during active output without losing follow-output behavior; canonical and live gates pass | M | Medium-High | Live repro window |

### Milestone 1 — Critical fixes

| ID | Task | Files/areas | Acceptance criteria | Effort | Change risk | Dependencies |
|---|---|---|---|---:|---|---|
| T1.1 | Pin release supply chain | All `.github/workflows/*.yml`, release docs | Actions use commit SHAs with version comments; Blueprint pinned by commit; appimagetool immutable and digest-verified; workflow syntax and package smoke pass | M | Medium | None |
| T1.2 | Document and test socket trust modes | `auth.rs`, README/security docs, integration tests | Same-user and descendant-only threat models documented; mode behavior covered end-to-end; no compatibility change without operator decision | M | Medium | T0.1 |
| T1.3 | Add bounded coverage reporting | CI, scripts, docs | Coverage artifact produced for every PR; baseline recorded; initial threshold agreed before enforcement | M | Low-Medium | Canonical gate stable |

### Milestone 2 — High-leverage improvements

| ID | Task | Files/areas | Acceptance criteria | Effort | Change risk | Dependencies |
|---|---|---|---|---:|---|---|
| T2.1 | Extract CLI command metadata and parsing domains | `limux-cli/src/main.rs` into domain modules | Help, flags, targeting and output metadata have one source; no behavior change; all CLI tests pass | XL | High | T0.1, T1.3 |
| T2.2 | Extract GTK window domains | `window.rs` into workspace, notification, control, header and DnD modules | `window.rs` under 5,000 lines; pure logic separated; canonical and Xvfb gates pass | XL | High | T1.3, PR reconciliation |
| T2.3 | Unify control contract/parity tests | protocol/core/control bridge/registry | Every advertised method has one declared owner and parity test; deferred methods cannot be advertised; browser status explicit | XL | High | T2.1, T2.2 |
| T2.4 | Add bounded Xvfb PR smoke | CI and smoke script | Cached/headless job completes within agreed budget; no daily-driver mutation; failure artifacts retained | L | Medium | T1.1 |

### Milestone 3 — Quality and polish

| ID | Task | Files/areas | Acceptance criteria | Effort | Change risk | Dependencies |
|---|---|---|---|---:|---|---|
| T3.1 | Condense FYI archive-first | `FYI.md`, project `archive/` | Current index remains concise; full historical bytes preserved in unique archive; commit/PR/task pointers retained | M | Low | T0.3 |
| T3.2 | Classify branches and retained worktree | local refs, `/tmp/limux-release-*` | Owner/disposition matrix exists; only approved, clean, pushed/merged branches/worktrees removed; no local-only evidence | L | Medium | PR reconciliation |
| T3.3 | Define disk reclamation policy | `target/`, `logs/`, archive docs | Operator selects deletion exception or external archive; exact paths and reclaimed bytes verified; no source/evidence loss | S policy + execution time | Medium | Operator decision |
| T3.4 | Refresh release/onboarding docs | README, handoff index, verification docs | 0.2.3/current placeholders consistent; current manager/runtime/next action discoverable; doc-check passes | M | Low | T0.3 |

### Top-three implementation sketches

#### T0.1 — read-command safety

1. Add RED tests using a missing socket to prove `read-screen --help`, `capture-pane --help`, and unknown flags return before `Client::call`.
2. Introduce a small pure parser that enumerates consumed flags and returns `Help`, validated parameters, or an unknown-flag error.
3. Preserve the current no-target focused-surface behavior only for normal invocation; document that this is a same-user convenience boundary.
4. Run focused CLI tests, then `./scripts/check.sh` and `git diff --check`.

Gotcha: `tmux capture-pane` compatibility calls the same helper; help and unknown-flag behavior must not break supported tmux aliases.

#### T0.3–T0.5 — PR reconciliation

1. Freeze each old head and inventory commits/files against `origin/main`; never merge the stale TaskMaster JSON wholesale.
2. For #58, port current-state documentation only. For #67/#68, create fresh `limu/` branches from current main and replay coherent feature commits or exact patches.
3. Reproduce each latest P2 as a failing test before applying the fix.
4. Run branch-specific tests, the canonical gate, relevant preview/Xvfb evidence, and exact-head review.
5. Close old PRs as superseded only after durable replacement pointers exist.

Gotcha: squash merges make `git branch --no-merged` overcount unique work; use path/patch identity, not graph status alone.

#### T1.1 — release provenance

1. Inventory every `uses:` and executable download in all four workflows.
2. Resolve official immutable commit SHAs/digests and annotate each with the human-readable version.
3. Replace the Blueprint tag clone with an exact commit checkout and verify repository identity.
4. Replace the continuous AppImage URL with a versioned asset and published digest; fail closed on mismatch before `chmod`/execution.
5. Validate workflow syntax and run the package build in the existing reviewed CI boundary.

Gotcha: pinning improves provenance only if update cadence is owned; add a deliberate dependency-update process rather than silently returning to floating tags.

## Owner and Disposition Matrix

| Surface | Current owner | Verified state | Recommended disposition |
|---|---|---|---|
| Task 29 branch | `limu` | Pushed `7e0eb07`; deliberate RED tests, no implementation | Continue on same branch under T0.2; do not merge until green/live-smoked |
| TaskMaster master #31 | `limu` | `in-progress`; current active tag restored to resource-crash | Commit with audit/handoff lane; close only after every artifact is routed |
| PR #58 | Historical `lifo`/tutu content, manager decision now `limu` | Conflicting; mixed obsolete and useful docs | Port valid HANDOFF/attestation content, then close superseded |
| PR #67 | Historical `lifo`; manager decision `limu` | Conflicting; useful renderer work; exact-head P2 remains | Fresh current-main replacement, fix P2, reverify; do not merge old PR |
| PR #68 | Historical `bulo`; no live owner found; manager decision `limu` | Conflicting; useful logging work; exact-head P2 remains | Fresh current-main replacement, fix P2, reverify; do not merge old PR |
| `/tmp/limux-release-0.2.3-20260719` | Historical `hamo`; operator gate | Clean; head ancestor of main; retained by handoff | Keep until operator explicitly releases no-loss hold |
| Reve incident | Incoming from `reve`, now `limu` intake | Untracked; legacy 0.2.2 evidence | Move byte-identically into `LIMU_INBOX/`; run T0.6 |
| Nava design question | Incoming via `tutu`, now `limu` design queue | Untracked; exploratory only | Move byte-identically into `LIMU_INBOX/`; acknowledge; defer until hcom TUI sequencing clears |
| OMP scroll-yank bug | Incoming via `tutu`, now `limu`; TaskMaster master #32 | Operator-impact HIGH; causal theory unproven | Reproduce under current stable/preview, then execute T0.7 before feature expansion |
| Lifo HTML closeout packet | Operator/historical | Untracked, explicitly protected | Preserve byte-identically; do not stage/rename/archive without explicit decision |
| Historical handoffs | Named historical owners | Tracked, fragmented | Preserve; index from new `LIMU_HANDOFF.md`; no bulk rename/rewrite |

## Open Questions / Operator Decisions

1. **PR #67/#68 product intent:** recommended default is to port both because their TaskMaster resource/crash goals remain active, but neither old PR should be merged. Approve porting, or retire either feature?
2. **Retained release worktree:** it passes the present clean/ancestor checks, but the handoff explicitly retains it. Recommended default is keep until Hamo's release evidence is indexed, then remove with the no-loss gate.
3. **16 GiB `target/`:** repository policy forbids deletion by default, and moving it into `archive/` would not reclaim space. Recommended default is an explicit one-time approval for `cargo clean`/equivalent regenerable-artifact deletion after confirming no active build depends on it.
4. **Root `HANDOFF.md`:** ownership rules prevent Limu from overwriting Halo's file implicitly. Recommended default is create `LIMU_HANDOFF.md` now, then later convert root `HANDOFF.md` to a one-line manager-owned pointer only with explicit canonical-ownership approval.
5. **Socket trust posture:** should same-user processes remain the default automation boundary, or should operator-launched environments prefer existing `limuxOnly` descendant mode? Recommended default is document and test first; do not change the default in the cleanup lane.
