# PRD-I: HCOM-Limux Visibility and Restart Integration

**Status:** Planning approved; implementation not authorized
**Owner:** lifo (`LIMUX_MGR`)
**Joint owner/dependency:** dino (`HCOM_MGR`)
**TaskMaster:** Task 23, subtask 23.3
**Canonical contract:**
`docs/future-improvements/hcom-limux-session-pane-visibility-restart-design-20260712.md`
**Source contract:** HCOM PR #32 at
`cfb8318957109dc1052902175d6cfe7f15a996d0`
**Operator decision:** Option A on 2026-07-13 authorizes this planning artifact,
not implementation, merge, installation, activation, or promotion.

## 1. Introduction

Limux can show the panes an operator sees, while HCOM knows coordination
identity, session bindings, process endpoints, and delivery history. Today those
planes can disagree after duplicate launches, stale environment inheritance,
host crashes, or native session resume. The result is unsafe targeting and an
operator UI that may show one pane while HCOM reports another process.

PRD-I defines Limux's G4 isolated-preview implementation lane for the approved
joint contract. It does not redesign HCOM's G3 registry or authorize changes to
the stable/daily-driver runtime.

## 2. Goals

- Give every Limux host lifetime an opaque runtime incarnation and expose it
  consistently through the control bridge, terminal environment, persistence,
  and diagnostics.
- Make caller identity provenance explicit while preserving backward-compatible
  read-only focus fallback and rejecting fallback for strict/mutating actions.
- Display and copy the same workspace, Surface/Pane, process-instance,
  freshness, delivery, and conflict truth available to agents.
- Integrate with the HCOM G3 contract through a durable, idempotent,
  transport-independent bind/ACK/nonce protocol.
- Suppress guessed restore and mutation when identity, runtime incarnation,
  lease, or exact target disagrees.
- Validate all work in an isolated preview runtime without replacing or
  restarting the daily driver.

## 3. Gate Dependencies

1. **G1 complete:** canonical Limux contract exists and is co-owner verified.
2. **G2 complete:** cross-family design findings are reconciled in canonical
   v0.4.
3. **G3 contract dependency:** Dino must provide the HCOM G3a PRD/task, exact
   schema/API version, capability/lease semantics, and G3b verification plan.
4. **G4 planning:** this PRD and its eventual TaskMaster expansion may proceed.
5. **G4 implementation:** may begin only after a reviewed implementation plan,
   clean owned branch, explicit dependency freeze, and normal PR gates.
6. **G5/G6:** real failure smokes and stable promotion remain separate future
   gates requiring operator approval where specified by the canonical contract.

## 4. User Stories

### US-I.1: Runtime incarnation and provenance

**Description:** As an operator or automation client, I need every Limux host
lifetime and caller claim to identify its source so stale pre-crash data cannot
masquerade as current authority.

**Acceptance Criteria:**

- [ ] RED tests prove two host launches in the same channel receive distinct,
      never-reused runtime incarnations.
- [ ] The incarnation is allocated once per host lifetime and does not advance
      when an agent resumes inside the existing host.
- [ ] `system.identify` preserves structured `focused` and `caller` objects and
      adds `caller_identity_source`, `caller_is_fallback`, and warning/reason
      fields.
- [ ] Provenance values are exactly `explicit_params`, `environment`,
      `focused_fallback`, or `unavailable`.
- [ ] Strict identity and every mutation reject `focused_fallback`; default
      read-only identify retains it with a visible warning.
- [ ] Targeted host and CLI tests pass before broader workspace tests.

### US-I.2: Canonical Surface/Pane context

**Description:** As a human or agent, I need the same exact context from the
right-click menu, CLI, and shared HCOM record so I can target the pane I see.

**Acceptance Criteria:**

- [ ] Context preserves GTK `workspace_id`, `pane_id`, `tab_id`, composite
      `surface_id`, socket, channel, and runtime incarnation without comparing
      them to standalone `limux-core` numeric handles.
- [ ] `workspace_name`, `workspace_cwd`, `surface_cwd`, and
      `terminal_workspace_cwd` retain separate provenance; a bare CWD is never
      presented as workspace directory.
- [ ] `Copy All Context` includes contract version, exact selectors, runtime
      incarnation, provenance, conflict state, and redacted HCOM instance data.
- [ ] `Copy Pane Read Command` uses explicit workspace and surface selectors.
- [ ] Sensitive socket paths, endpoint addresses, ancestry, and environment
      values are hidden unless manager-authorized diagnostics are explicitly
      requested.
- [ ] GTK tests verify menu labels and copied formatter output.

### US-I.3: Durable HCOM bind handshake

**Description:** As a session restored or launched in a Limux pane, I need a
durable exact bind so HCOM and Limux agree before either reports the session as
active.

**Acceptance Criteria:**

- [ ] A failing test demonstrates that inherited environment alone cannot prove
      a current binding.
- [ ] Limux validates the HCOM contract/schema version, launch UUID,
      lease/capability reference, current runtime incarnation, workspace,
      surface, pane, tab, socket, and channel.
- [ ] The transport preserves durable outbox, idempotency key, bind ACK,
      single-use nonce, bounded retry, and compensation requirements regardless
      of whether the final mechanism is environment bootstrap, local RPC, event
      stream, or a reviewed combination.
- [ ] Replayed nonce, stale lease, changed incarnation, and duplicate bind
      requests fail without retargeting another pane.
- [ ] Raw capabilities, nonce values, message text, and transcript content are
      never stored or displayed.
- [ ] Cross-plane protocol tests run against the exact HCOM G3 contract version.

### US-I.4: Exact targeting and restore suppression

**Description:** As an operator recovering sessions after a crash, I need Limux
to refuse guessed targeting instead of restoring the wrong process into a pane.

**Acceptance Criteria:**

- [ ] Duplicate HCOM identity/native-session candidates remain separately
      visible and enter `duplicate_quarantined` state.
- [ ] Resume, exit, kill, terminal injection, cleanup, targeted delivery, and
      automatic restore require an exact current launch UUID, authorized HCOM
      lease/capability, and matching Limux incarnation.
- [ ] Focus, display name, PID alone, restored environment, or newest-record
      heuristics never authorize mutation.
- [ ] An unproved old process remains `stopping`, `unreachable`, or
      `quarantined`; successful replacement verification does not mark it
      stopped.
- [ ] Restore disagreement produces inspectable `restore_suppressed` state and
      never launches a guessed native session.
- [ ] Selector/action race tests prove compare-and-set failure cannot reach a
      replacement process.

### US-I.5: Honest activity and delivery display

**Description:** As an operator, I need to distinguish actual agent work,
terminal activity, queued messages, and unavailable telemetry.

**Acceptance Criteria:**

- [ ] UI/CLI show orthogonal process, agent-turn, terminal, delivery, Limux-host,
      workspace, identity, and restore dimensions with source and freshness.
- [ ] Per-runtime `turn_hooks` capability is shown as `yes`, `no`, or `unknown`.
- [ ] `working` requires a fresh positive native turn observation plus a live,
      currently fenced process; terminal output or CPU cannot establish it.
- [ ] Freshness uses receiving-plane monotonic receipt time; sender wall time is
      provenance only and bounded skew yields uncertainty rather than false
      freshness.
- [ ] Delivery UI distinguishes identity enqueue, endpoint acceptance, PTY
      injection, hook presentation, model-visible ACK, timeout, and relay state.
- [ ] A zero-live-instance enqueue is displayed honestly rather than as generic
      sent/delivered success.
- [ ] The closed operator legend includes `working`, `idle`,
      `delivery degraded`, `duplicate quarantined`, `restore suppressed`, and
      `unknown`.

### US-I.6: Preview-only rollout and failure validation

**Description:** As the daily Limux operator, I need this work tested without
affecting my active stable sessions.

**Acceptance Criteria:**

- [ ] Development and integration use an isolated `preview:<id>` install root,
      socket namespace, state directory, and runtime incarnation.
- [ ] `limux-preview target-info` and `doctor --json` prove preview targeting
      before every live integration run.
- [ ] Stable and preview runtimes remain simultaneously visible and independently
      addressable when names/native UUIDs collide.
- [ ] Automated tests cover all 21 canonical acceptance cases, with Limux-owned
      fixtures for stale focus, duplicate workspace names, host restart, relay
      death, PTY stop, endpoint/process lifetime inversion, and clock skew.
- [ ] `./scripts/check.sh`, focused host/CLI tests, Ghostty resource validation,
      and Xvfb smoke pass on the exact PR head.
- [ ] No stable launcher, daily-driver install, active socket, session state, or
      global skill is changed during G4.

## 5. Functional Requirements

- **FR-I.1:** Add an opaque per-host `limux_runtime_incarnation` with explicit
  lifecycle and persistence semantics.
- **FR-I.2:** Extend `system.identify` and related context surfaces with caller
  provenance without changing structured caller/focused object shapes.
- **FR-I.3:** Define one normalized contract record that retains exact GTK
  identifiers and source-specific CWD values.
- **FR-I.4:** Add version negotiation and a transport-independent HCOM bind
  interface with idempotent ACK/nonce/fencing behavior.
- **FR-I.5:** Require exact current selectors and authority for every
  cross-plane mutation or restore.
- **FR-I.6:** Render duplicate/conflicting instances separately and expose
  actionable exact selectors without leaking sensitive local details.
- **FR-I.7:** Track activity and delivery as source-tagged, expiring dimensions,
  not one synthesized status.
- **FR-I.8:** Evaluate freshness with receiver monotonic time and deterministic
  skew handling.
- **FR-I.9:** Preserve backward-compatible read-only inspection and identity
  enqueue while failing closed for unsupported cross-plane mutation.
- **FR-I.10:** Keep all implementation and live validation isolated to preview
  until G5/G6 evidence and approval exist.

## 6. Non-Goals

- Implementing HCOM's process-instance database, lease allocator, or identity
  inbox semantics in Limux.
- Auto-killing duplicate processes.
- Treating focus, terminal output, CPU, display name, or PID as authority.
- Parsing message or transcript content to infer activity.
- Changing relay trust, credentials, cryptographic authority, or stale fixture
  cleanup policy.
- Replacing the stable/daily-driver runtime, installing globally, or promoting
  global skills during G4.
- Choosing the G6 retention default or approving production activation.

## 7. Technical Considerations

- Existing GTK identifiers are string/UUID/composite values; standalone
  `limux-core` uses different numeric handle domains.
- `system.identify` currently falls back to focused context without provenance;
  implementation must preserve read compatibility while making uncertainty
  explicit.
- The production path is the GTK bridge in
  `rust/limux-host-linux/src/control_bridge.rs` and `window.rs`; standalone core
  tests alone are insufficient.
- Pane construction and host-level callbacks remain centralized in `window.rs`
  and `PaneCallbacks`; do not bypass that boundary.
- HCOM G3 must freeze schema/API/lease behavior before cross-plane G4 code can be
  merged. Use contract-version negotiation and fail closed on mismatch.
- New behavior follows RED-GREEN-REFACTOR. Each acceptance behavior requires a
  failing test observed before production code.

## 8. Verification Matrix

| Layer | Required evidence |
|---|---|
| Pure logic | Focus provenance, CWD provenance, incarnation fencing, TTL/skew, duplicate selection, and state derivation unit tests |
| GTK host | Structured identify payload, context menu/copy output, exact pane targeting, restore suppression, and visible conflict state tests |
| CLI/protocol | Schema negotiation, redaction, exact selectors, error codes, and backward-compatible read-only output tests |
| HCOM integration | Durable bind/ACK/nonce, idempotency, stale lease/incarnation, duplicate process, replay, and compensation tests against frozen G3 contract |
| Runtime isolation | Stable plus preview simultaneous launch, separate sockets/state/install roots, and duplicate-name/native-session visibility |
| Full gate | `cargo fmt --check`, workspace clippy/tests, `scripts/check.sh`, Ghostty validation, Xvfb smoke, preview `doctor --json`, exact-head review |

## 9. Success Metrics

- Every live process instance visible in Limux is independently correlatable to
  one HCOM launch record or explicitly marked incomplete/conflicting.
- No strict mutation or restore succeeds from focus fallback, stale environment,
  stale lease, stale incarnation, ambiguous name/session, or selector race.
- Operator-visible activity and delivery state always names its source,
  freshness, and achieved delivery level.
- Stable daily-driver sessions remain unaffected throughout G4 development and
  validation.
- All 21 canonical acceptance cases pass against matching HCOM/Limux contract
  versions before G5 begins.

## 10. Open Planning Questions

1. Which transport satisfies the frozen durable outbox/ACK/nonce requirements
   with the smallest GTK main-thread and HCOM coupling?
2. What additive HCOM schema/API version and compatibility negotiation will G3
   expose?
3. What source-specific TTL defaults should Limux display, and which values are
   negotiated by HCOM?
4. How should sensitive manager-only diagnostics be authorized without making
   local path or endpoint details ambient?
5. Which subset of the 21 cases belongs in fast PR CI versus retained preview
   integration and G5 real-restart suites?

These questions must be resolved in the implementation plan and linked HCOM G3
artifacts. They do not authorize code changes.
