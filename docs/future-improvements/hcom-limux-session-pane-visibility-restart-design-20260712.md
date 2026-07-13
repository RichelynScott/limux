# HCOM-Limux Session Visibility and Restart Contract v0.4

**Status:** Operator-approved consolidated design; implementation and runtime activation are not authorized
**Owners:** dino (`HCOM_MGR`) and lifo (`LIMUX_MGR`)
**Canonical joint artifact:** this Limux document
**HCOM source input:** PR #32 at
`cfb8318957109dc1052902175d6cfe7f15a996d0`, path
`docs/design/HCOM_INPUT_FOR_LIMUX_SESSION_VISIBILITY_RESTART_CONTRACT_V0_4.md`,
SHA-256 `dac08f8fb6d853e1d609c87591ae63576e1ab97c980bed18f3e53e5313753d01`
**Joint tracker:** Limux TaskMaster 23 lineage
**Evidence freeze:** 2026-07-12 D4 evidence bundle in
`/home/riche/Proj/CODEX_CLAUDE_CODE/tasks/global-config-reorg-20260712/REORG_D4_VISIBILITY_EVIDENCE_2026-07-12.md`

**Review provenance:** v0.1 was frozen at 436 lines with SHA-256
`74961bc556039f690fda3c5ca18e85b01b264d0e5ec343dea2a6c0448171947d`.
Lifo reviewed that exact artifact in HCOM event `#439256`; Lita independently
reviewed it in event `#439306` and confirmed the frozen input in `#439343`.
The 651-line v0.2 was frozen at SHA-256
`05cbda03a565083c1b9234bd42b743fc2b7f1ff6efd12fe8a1259ae2ce2000e6`.
Lita closed her findings in event `#439793`; Lifo accepted all but two exact
Limux schema corrections in event `#439817`, incorporated in this v0.3.
Kuma's Claude-family Phase B critique independently verified v0.3 at SHA-256
`8f8ed5118fb2173470fc3ae0bc1db693dd721400e114153c027ddc9411ba07bb`
and returned `APPROVE_WITH_IMPROVEMENTS`. Its three mandatory usability and
freshness corrections are incorporated in this v0.4.

## 1. Purpose

HCOM and Limux currently describe overlapping parts of one operator-visible
agent runtime:

- HCOM knows agent identity, native session identity, delivery endpoints,
  process bindings, lifecycle events, and durable coordination metadata.
- Limux knows host/runtime, workspace, Surface/Pane, pane, tab, terminal,
  visibility, and restore state.

Neither plane alone can prove which process a human-visible pane represents.
This canonical joint contract defines the shared instance identity and state
model that HCOM and Limux must implement so managers can
answer:

1. What is this session or pane actually doing?
2. Is the reported activity current, inferred, or stale?
3. Are multiple runtimes claiming the same HCOM name or native session?
4. Can a resume, exit, send, cleanup, or restore safely target exactly one
   process instance?
5. Did crash recovery preserve durable manager metadata and bind the restored
   process to the current Limux runtime?

This is an operator-approved consolidated design, not an implementation
authorization. Relay trust,
credential changes, log remediation, and global-skill activation remain
separate governed work.

## 2. Evidence and Current Constraints

The contract responds to these verified incidents:

- HCOM events `#436913` and `#436990`: two live Codex runtimes shared the same
  HCOM name and native session UUID, but `hcom list`/`diagnose` rendered one
  row. Limux showed both panes. Cleanup had recorded `pre_resume_stop` while the
  old runtime remained alive.
- HCOM event `#436705`: the duplicate runtime entered another owner's active
  worktree before exact-process containment.
- HCOM event `#437169`: crash recovery retained the native session UUID but
  silently reset `owner`, `scope`, `lifetime`, `closeout_condition`,
  `authority_level`, and `durable_handoff` to null.
- HCOM events `#437029`, `#437055`, and `#437095`: delivery phase-one timeouts
  were followed by PTY stop after containment.
- Limux restore evidence: an earlier exact-surface proof later became invalid
  when restored state overwrote a pane with another session's UUID/name.
- Relay evidence `#437164`: the worker died without clearing its apparent
  status; stale temporary fixture trust records were also visible.

The present HCOM schema explains the collapse:

- `instances.name` is the primary key and `instances.session_id` is unique
  (`src/db/mod.rs`).
- `session_bindings` maps one native session ID to one instance name
  (`src/db/sessions.rs`).
- `process_bindings` can hold process IDs, but `hcom diagnose` projects their
  existence to one boolean `process_bound` and reads one `instances` row
  (`src/commands/diagnose.rs`).
- notification endpoints are keyed and migrated by instance name and endpoint
  kind, which can replace a prior endpoint of the same kind
  (`src/db/sessions.rs`).
- name-based resume calls cleanup before launch, but a pidless row is stopped
  without proving every prior process or PTY has exited
  (`src/commands/resume.rs`).

The implementation must preserve backward compatibility for identity-level
messaging and stopped-session history while adding an explicit process-instance
layer. It must not overload the current singleton `instances` row with more
ephemeral process truth.

## 3. Terminology and Identifier Authority

| Term | Meaning |
|---|---|
| Stable identity | Opaque, never-reused `identity_id`. Durable metadata attaches here, not to a mutable display name. |
| HCOM name | Human-facing mutable alias such as `dino`, versioned by `alias_revision`. It is not an identity, session, or process key. |
| Native session association | Versioned link from an `identity_id` to a Claude/Codex/Hermes/Grok session or thread identifier. Multiple live processes may temporarily claim it and remain separately visible. |
| Launch instance | One launch attempt, identified before spawn by a random, never-reused `launch_instance_uuid`. |
| Process instance | One observed agent process, correlated to a launch instance by OS host-boot identity, PID, and process-start identity. |
| HCOM lease | HCOM-issued lease epoch and unforgeable capability binding an authorized launch instance to an identity. |
| Limux runtime incarnation | Opaque unique identifier allocated once per Limux host lifetime. Agent resumes inside that host do not advance it. |
| Surface/Pane | Human UI label. The contract retains precise `surface_id`, `pane_id`, and `tab_id` fields. |
| Activity observation | A timestamped fact from a named source, with freshness and confidence. It is not a synthesized claim without provenance. |
| Quarantine | Fail-closed state where ambiguous or conflicting instances remain visible but identity-level mutation/restore is suppressed. |

Identifier authority is explicit:

| Plane | Authoritative fields |
|---|---|
| HCOM | `identity_id`, alias/native-session revisions, `launch_instance_uuid`, lease epoch/capability, durable manager metadata, identity inbox, and delivery receipts |
| Limux | runtime incarnation, socket/channel, workspace, surface, pane, tab, visibility, and host-side caller provenance |
| OS observer | device boot identity, PID, parent PID, and process-start identity or equivalent pidfd proof |
| Native agent runtime | native session identifier; HCOM owns only the versioned association record |

### 3.1 Existing Limux identifier mapping

The joint contract MUST preserve the current GTK identity vocabulary rather
than inventing a second mapping:

| Current Limux field | Type / selector | Normalized contract field | Contract treatment |
|---|---|---|---|
| `workspace_id` | UUID string; `workspace:<workspace_id>` | `workspace_id` | Exact value, scoped to the verified runtime incarnation |
| workspace row `name` / `title` | string or null | `workspace_name` | Workspace display metadata; never used as an identity join |
| workspace row `cwd` | path or null | `workspace_cwd` | Workspace-directory provenance only; default UI/CLI may redact it |
| `pane_id` | internal `u32`, decimal serialization; `pane:<pane_id>` | `pane_id` | Exact value; never interchangeable with Limux-core numeric `u64` handles |
| `tab_id` | UUID string; `tab:<tab_id>` | `tab_id` | Exact value, scoped to the verified runtime incarnation |
| `surface_id` | composite `<pane_id>:<tab_id>`; `surface:<surface_id>` | `surface_id` | Exact composite, scoped to the verified runtime incarnation |
| active `SurfaceSummary.cwd` | path or null | `surface_cwd` | Active-surface provenance only; never presented as workspace directory |
| `TerminalIdentity.workspace_cwd` | path or null | `terminal_workspace_cwd` | Right-click terminal-context provenance; compare explicitly, never silently merge |
| `pid` | host process ID | `limux_host_pid` | Correlation only; pair with host boot and process-start identity |
| `channel` | runtime channel | `limux_runtime_channel` | Insufficient alone; pair with socket and incarnation |
| `socket_path` | local runtime socket | `limux_socket_path` | Required for current-runtime handshake; redacted from default display |
| `runtime_id` | current runtime identifier | `limux_runtime_id` | Correlation only; opaque incarnation is the fence |
| `system.identify.focused` | structured surface/location object or null | `limux_focused_context` | Contains workspace/pane/surface refs and related surface metadata; observation only, never mutation authority |
| `system.identify.caller` | caller-supplied structured object or null; currently clones `focused` when omitted | `limux_caller_context` | Preserve object shape and accompany it with provenance below |

`terminal_id` is absent from the current GTK contract. It is optional,
proposed-only metadata and MUST NOT be a required join key. Limux core numeric
`u64` identifiers are standalone core handles and MUST NOT be serialized as or
compared with GTK workspace/pane/tab/surface identifiers. G4 must document and
test each identifier's persistence and reuse behavior; until then, all GTK
location identifiers are treated as valid only under an exact current runtime
incarnation.

A normalized boolean such as `limux_has_focused_context` may be derived for UI
display, but it is proposed metadata and MUST NOT replace or masquerade as the
current structured `focused` object. Every CWD field carries its exact source.
A bare `cwd` is invalid in the shared record and MUST NOT be joined or displayed
as a workspace directory without provenance. Missing values remain explicit
`null` plus a reason.

Caller provenance uses the TaskMaster 23 vocabulary:
`explicit_params`, `environment`, `focused_fallback`, or `unavailable`, plus
`caller_is_fallback` and a warning/uncertainty field. Read-only inspection may
use `focused_fallback` with a warning. Strict identity and every mutation MUST
reject it.

## 4. Shared Correlation Record

Each launch MUST be addressable by a random immutable
`launch_instance_uuid`, allocated durably before spawn. The key is not the HCOM
name, native session UUID, PID, or a concatenation of reusable identifiers.

```text
launch_instance_uuid = random never-reused UUID
```

The shared record is versioned and contains:

```text
contract_version
identity_id
launch_instance_uuid
device_id
device_boot_id

hcom_name
alias_revision
hcom_process_id              # legacy/correlation field, not a primary key
hcom_lease_epoch
hcom_capability_id           # fingerprint/reference only; never raw token
hcom_lease_expires_at
native_tool
native_session_id
native_association_revision
os_pid
parent_pid
process_start_identity
process_started_at
process_observed_at

limux_runtime_id
limux_runtime_channel
limux_socket_path
limux_host_pid
limux_runtime_incarnation
workspace_id
workspace_name
workspace_cwd
surface_id
surface_cwd
pane_id
tab_id
terminal_id                  # optional/proposed; never a required join
terminal_workspace_cwd       # right-click terminal context, separately sourced
limux_focused_context        # structured object or null
limux_caller_context         # structured object or null
caller_identity_source       # explicit_params | environment | focused_fallback | unavailable
caller_is_fallback
caller_warning
workspace_visibility       # visible | hidden | unknown

launch_state               # reserved | pending | bound | verified | active | aborted | quarantined
launch_idempotency_key
launch_outbox_sequence
launch_inbox_sequence

delivery_endpoints[]       # kind, address/port, lease_epoch, last_seen_at
activity_observations[]    # dimension, value, source, source_sequence/turn_id,
                           # observed_at, expires_at, confidence
lifecycle_state
lifecycle_reason
lifecycle_observed_at

owner
scope
lifetime
closeout_condition
authority_level
durable_handoff
metadata_revision          # identity-scoped

identity_provenance[]      # source, observed value, certainty, timestamp
conflicts[]                # field, competing values, source records
```

Rules:

1. A field that is unavailable is explicit `null` plus a reason; absence MUST
   NOT silently mean false, idle, stopped, or unowned.
2. HCOM and Limux identifiers are stored independently. UI may label the group
   `Surface/Pane`, but JSON retains workspace, surface, pane, and tab.
3. Stable and preview Limux runtimes remain distinct even when HCOM name and
   native session UUID match.
4. `system.identify` exposes the caller provenance vocabulary above. Focused
   fallback is always labeled and strict callers reject it.
5. Events carry HCOM lease epoch/capability identity and Limux runtime
   incarnation when known. A prior lease or incarnation cannot update a
   current record.
6. A Limux incarnation is evidence of host identity, not an HCOM fence. HCOM
   mutations require an HCOM-issued capability checked atomically with the
   current lease epoch.

## 5. Durable and Ephemeral State

Durable coordination metadata is `identity_id`-scoped and survives process
loss and display-name changes:

- `owner`
- `scope`
- `lifetime`
- `closeout_condition`
- `authority_level`
- `durable_handoff`

Process, endpoint, activity, and pane attachment are launch/lease-scoped and may
expire. Restart/reclaim allocates a launch UUID and advances the HCOM lease; it does not
replace durable metadata with nulls. Metadata changes use a revision or
compare-and-set rule so stale process events cannot overwrite newer values.

Rename, adoption, alias collision, and native-session reassociation obey these
rules:

- Rename appends an alias revision while preserving `identity_id` and durable
  metadata. Historical aliases remain discoverable but cannot resolve to two
  current identities.
- Adoption reuses an identity only after current authority and native-session
  proof. Otherwise it creates a new `identity_id` and an explicit relationship
  to the prior identity.
- A current-alias collision is rejected and quarantined; it is never resolved
  by last-writer-wins.
- Native-session reassociation appends a revision. The prior association is
  retained as history and cannot silently follow a renamed or forked session.

Crash recovery MUST report `metadata_unavailable` with the missing fields and
recovery source if durable metadata cannot be recovered. Silent null reset is a
contract violation.

## 6. State Model

### 6.1 Orthogonal dimensions

One status word is insufficient. Every process instance reports these
dimensions separately:

| Dimension | Required values |
|---|---|
| Process | `starting`, `alive`, `exited`, `unreachable`, `unknown` |
| Agent turn | `prompt_active`, `tool_active`, `model_wait`, `turn_idle`, `unknown` |
| Terminal | `attached`, `output_recent`, `quiet`, `pty_stopped`, `unreachable`, `unknown` |
| Delivery | `live`, `hook_safe`, `queued`, `phase1_timeout`, `endpoint_stale`, `relay_degraded`, `unavailable` |
| Limux host | `live`, `stale`, `dead`, `unknown` |
| Workspace | `visible`, `hidden`, `not_realized`, `unknown` |
| Identity | `consistent`, `incomplete`, `conflicting`, `duplicate_quarantined` |
| Restore | `not_needed`, `candidate`, `suppressed`, `launching`, `verified`, `failed` |

Each value includes source, observation timestamp, freshness threshold, and
whether it is observed or inferred. Recent terminal output alone MUST NOT be
reported as a running agent task.

Activity is mechanically decidable:

1. Every observation carries a source sequence or native turn ID when the
   source exposes one. Lower or duplicate sequence values cannot overwrite a
   newer observation.
2. Source-specific TTLs are part of the negotiated contract. Expiry changes a
   dimension to `unknown`; it never implies idle, stopped, or healthy.
3. Native prompt/tool/turn hooks outrank terminal output and CPU observations
   for agent-turn state. `hook_safe` describes delivery capability, never turn
   state.
4. `working` requires a fresh positive native turn observation and a live,
   currently fenced process. Terminal output or CPU may corroborate but cannot
   establish it.
5. Missing or out-of-order Stop/turn-idle hooks expire to `unknown`. They do
   not leave a permanent working or idle claim.
6. Freshness is evaluated by the receiving plane against a monotonic receipt
   clock, not by comparing untrusted sender wall-clock time directly. Sender
   timestamps remain provenance only. A negotiated bounded skew allowance may
   classify an observation as uncertain, but cannot make an expired receipt
   fresh or a fresh receipt expired.

### 6.2 Derived summaries

Human summaries may be derived only from the dimensions above. Examples:

- `working (hook: tool_active, 3s fresh; pane hidden)`
- `idle (hook: turn_idle, 8s fresh; PTY attached)`
- `delivery degraded (phase1 timeout; process alive)`
- `duplicate quarantined (2 live instances; exact target required)`
- `restore suppressed (persisted/native/Limux identity conflict)`

Derived summaries never erase the underlying observations.
The canonical operator view defines a small closed legend for these summaries
(`working`, `idle`, `delivery degraded`, `duplicate quarantined`,
`restore suppressed`, and `unknown`) so managers do not need to decode all
dimensions before acting. New labels require a contract-version change.

## 7. Required Invariants

### I1. Stable identity and multi-instance truth

HCOM MUST retain and render every live process instance under a stable
`identity_id`. Two processes sharing
an HCOM name or native session UUID appear as two rows/children, with their
process IDs, OS PIDs, lease epochs, runtime incarnations, endpoints, Limux
locations, and freshness.
The singleton identity row may summarize but cannot collapse them.

### I2. Lease and incarnation fencing

Every HCOM launch receives a new random `launch_instance_uuid` and reserved
lease epoch/capability. Each Limux host lifetime receives one opaque runtime
incarnation; resuming an agent within that host does not change it. Status,
hook, endpoint, stop, and restore writes condition atomically on the same
current HCOM lease and exact Limux incarnation where cross-plane state is
involved. Old evidence remains inspectable but cannot mutate current bindings.

### I3. Durable fenced launch saga

HCOM and Limux do not rely on an impossible cross-database transaction. A
launch is a durable idempotent saga: reserve lease and launch UUID by CAS,
write a pending launch and outbox record, obtain Limux bind ACK, spawn, obtain
HCOM bind ACK, verify a nonce, then activate by CAS against the same capability.
Retries replay the same idempotency key. Lease expiry and compensation leave
the launch aborted or quarantined; they never silently activate another PID.

### I4. Proven process closure

A resume cleanup may mark the old instance `stopping`, but MUST NOT finalize it
as stopped or authorize a replacement as unique until it proves one of:

- exact child exit was observed;
- OS PID plus process start identity no longer exists;
- an HCOM lease transfer was completed after equivalent closure proof.

A database lifecycle event or verified replacement alone is not process-exit
proof. If closure is unproven, the default is to block replacement. An explicit
operator recovery may create a second quarantined launch, but neither launch
may be called unique or receive name-level mutation authority until resolved.

### I5. Exact targeting, authorization, and TOCTOU resistance

When more than one live/current candidate matches a name or session UUID,
mutating `resume`, `exit`, `kill`, `term inject`, targeted `send`, cleanup, and
automatic restore fail closed. The command lists candidates and requires an
exact `launch_instance_uuid` plus current HCOM lease/capability or an
unambiguous Limux selector bound to that same capability. Selector knowledge is
not authority. The mutation condition checks caller authorization,
`authority_level`, launch UUID, lease epoch/capability, and Limux incarnation in
the same transaction or capability validation that performs the mutation. A
receipt records the selector and fence. Focused-pane fallback is forbidden.

### I6. Endpoint ownership

Each endpoint belongs to one launch UUID and lease epoch. Endpoint expiry or
PTY stop degrades only that instance. It cannot delete or overwrite a newer
instance's endpoint because the HCOM name and endpoint kind happen to match.

### I7. Restore agreement and current-runtime handshake

Before launch, restore reconciles persisted Limux location, HCOM identity,
native session UUID, process instance, HCOM lease, and Limux incarnation. Disagreement or
multiple candidates yields `restore_suppressed` and an inspectable diagnostic
shell/state. It never launches the guessed session.

After launch, the process receives current Limux workspace/surface/pane/tab and
runtime-incarnation values. A nonce-bound handshake proves the current socket,
channel, incarnation, and launch UUID. Restored or inherited environment values
alone are never proof and replayed nonces are rejected.

### I8. Identity messaging and direct delivery are separate

`hcom send @name` remains a backward-compatible durable identity enqueue. It
stores one inbox event even when zero, one, or many process instances are live;
duplicates do not make identity messaging fail. Endpoint acceptance, exact
instance delivery, PTY injection, hook presentation, and model-visible receipt
are separate operations with independent idempotency, retry, and receipts.

### I9. Delivery honesty

Delivery reports distinguish enqueue, endpoint acceptance, model-visible hook
delivery, PTY injection, timeout, and relay transport. `live` is cleared or
expired when its owning PTY, process, relay worker, or endpoint dies. A
phase-one timeout followed by PTY stop remains visible as a transition, not a
success-looking final state.

### I10. Host, relay, and fixture classification

Host/process identity includes device boot identity and OS process-start
identity or pidfd-equivalent evidence so PID reuse cannot revive an old record.
Relay worker health is derived from live PID identity plus heartbeat/lease expiry,
not an uncleared status flag. Dead worker state becomes `relay_degraded` and
cannot continue advertising connected delivery.

Temporary fixture hook-trust records are classified as `fixture_stale` when the
source path is temporary/missing, the recorded Codex version is obsolete, or a
test hash no longer maps to a live HCOM hook. Classification is read-only in
this contract; removal is a separately reviewed remediation.
G3a must define the authoritative installed/expected Codex-version source used
for the obsolete-version comparison; display text or ambient PATH order is not
authority.

## 8. Command and UI Contract

### 8.1 Read-only inspection

HCOM provides a manager-grade view, provisional command shape:

```text
hcom inspect <name-or-session> [--instances] [--json]
hcom diagnose <name-or-session> --instances [--json]
```

Required output:

- identity summary and durable manager metadata;
- one block/object per process instance;
- exact Limux runtime/workspace/surface/pane/tab;
- HCOM lease, Limux incarnation, and identity provenance;
- dimensioned activity/delivery/restore states;
- age/freshness per observation;
- conflicts and the exact selector needed for mutation.
- per-source turn-signal capability for each native runtime, reported as
  `turn_hooks: yes | no | unknown`, so `unknown` activity distinguishes a stale
  observation from a runtime that cannot emit positive turn evidence.

Limux UI and CLI expose the same record. Context menus provide canonical
`Copy All Context` and `Copy Pane Read Command` actions. Managers can read pane
contents with explicit workspace plus surface targeting and see the same
identity, lease, and incarnation shown in UI.

### 8.2 Mutation

Identity messaging remains available regardless of process ambiguity:

```text
hcom send @<name> -- <message>                 # one durable identity enqueue
```

Process mutation remains convenient when exactly one current instance is
proven. Otherwise it fails with a candidate table and accepts an exact selector,
provisional shape:

```text
hcom r <name> --instance <launch_instance_uuid>
hcom kill <name> --instance <launch_instance_uuid>
hcom term inject <name> --instance <launch_instance_uuid> <text>
hcom deliver --instance <launch_instance_uuid> -- <message>
```

Selectors are not authorization. The exact selector, caller authority, HCOM
lease/capability, and Limux incarnation are checked atomically with mutation.
A stale selector or changed lease fails; it is never redirected to the newest
or focused pane. The receipt distinguishes identity enqueue, endpoint
acceptance, PTY injection, hook presentation, and model-visible ACK.

The default non-JSON `hcom send` success line MUST state the highest delivery
level actually achieved and the live-instance count. It must say, for example,
`enqueued; 0 live instances` or `enqueued; 2 live instances, 1 hook-delivered`;
plain `sent`/`delivered` wording is forbidden when only identity enqueue is
proved. JSON exposes the same counts and receipt levels as separate fields.

### 8.3 Recovery planning

Recovery first produces a read-only plan. It identifies dead hosts, stale
endpoints, metadata gaps, conflicts, and exact process candidates. Execution is
separately explicit and preserves a before-state evidence bundle. Automatic
bulk recovery cannot mutate quarantined or ambiguous identities.

## 9. Restart Protocol

1. **Freeze:** capture HCOM identity/session/process/endpoints/metadata, host
   boot/process-start identity, and Limux runtime/workspace/surface/pane/tab.
2. **Detect:** expire dead host/process/relay observations to `unknown` or a
   proved terminal state without deleting durable identity/history.
3. **Reconcile:** join candidates by exact authority-owned IDs and provenance;
   never by display name, PID alone, restored environment, or focus.
4. **Resolve closure:** prove the prior process closed. If unproven, block the
   replacement by default. Explicit operator recovery may reserve a second
   quarantined launch, but it receives no uniqueness or name-mutation claim.
5. **Reserve:** HCOM allocates `launch_instance_uuid`, idempotency key, lease
   epoch/capability, and pending-launch/outbox record in one CAS transaction.
6. **Bind Limux:** Limux reads its current host incarnation, validates caller
   provenance, records the idempotent request, and ACKs exact
   workspace/surface/pane/tab plus socket/channel/incarnation.
7. **Spawn:** launch once from the durable request. Record host boot, PID, and
   process-start identity; never infer identity from PID alone.
8. **Bind HCOM:** the child proves possession of the reserved capability and
   binds its native session and endpoints to the launch UUID.
9. **Verify:** challenge the exact current Limux channel with a single-use
   nonce and verify HCOM binding, process ancestry, pane identity, native
   session association, endpoint ownership, and a real HCOM round trip.
10. **Activate:** CAS the same HCOM lease/capability from `verified` to
    `active`. A selector/action race fails this CAS rather than retargeting.
11. **Finalize prior state:** mark the old process stopped only with I4 closure
    proof. Replacement verification alone is insufficient; unresolved prior
    state remains `stopping`, `unreachable`, or `quarantined`.
12. **Compensate/replay:** spawn success followed by bind, verify, or DB failure
    becomes aborted/quarantined with retained evidence. Retry uses the same
    idempotency key and cannot duplicate side effects. Leases expire to a
    recoverable non-active state; retries are bounded.

## 10. Acceptance Test Matrix

Minimum automated and real-environment cases:

1. Same HCOM name and native UUID across two OS processes and two Limux
   runtimes: both render, identity quarantines, mutation requires exact target.
2. Same name with different native UUIDs; same UUID with different names.
3. Old-lease or old-incarnation hook/status/endpoint event after host restart cannot rewrite
   current state.
4. Missing/stale Limux environment triggers labeled focus inference; strict
   identity rejects it.
5. Duplicate workspace names with different workspace UUIDs remain distinct.
6. Limux host crash, HCOM relay death, WSL/Linux restart, PTY stop, process
   outliving endpoint, and endpoint outliving process.
7. Name resume where lifecycle cleanup succeeds in DB but old OS process
   ignores termination: replacement is quarantined or launch is blocked.
8. Manager metadata survives crash/reclaim; unavailable metadata reports
   explicit reason and source.
9. Phase-one delivery timeout followed by PTY stop preserves both transitions
   and never reports live delivery afterward.
10. Exact pane-content read and direct per-instance delivery do not use focus
    fallback.
11. Restore launches once into the intended Surface/Pane with current
   environment and nonce-verified HCOM delivery.
12. Stable daily-driver Limux and isolated preview runtime remain independently
   visible throughout the test.
13. Simultaneous resume requests contend on one lease CAS; exactly one becomes
    active and every side effect is idempotently attributable.
14. Selector validation succeeds, then the target lease changes before action:
    mutation fails without reaching the replacement.
15. Host reboot reuses a PID; device boot/process-start identity prevents stale
    process resurrection.
16. Rename, adoption, alias collision, and native-session reassociation during
    launch preserve stable identity and revision history or quarantine.
17. Spawn succeeds then HCOM DB commit fails; Limux binds then HCOM dies; both
    compensate to non-active inspectable state.
18. Replayed nonce, delayed old-endpoint renewal, partitioned relay, and
    duplicate notification cannot renew authority or duplicate identity inbox
    events.
19. Old client/new server and new client/old server combinations preserve
    identity messaging while refusing unsupported cross-plane mutation.
20. Partial schema migration recovery preserves legacy rows and reports a
    machine-readable compatibility state.
21. A skewed sender wall clock cannot make a stale observation fresh or a fresh
    receipt stale; receiving-plane monotonic TTL and bounded-skew uncertainty
    produce deterministic results across relay/device boundaries.

## 11. Gate Map

| Gate | Owner | Exit condition |
|---|---|---|
| G0 Evidence freeze | Dino + Lifo | Evidence pointers and current source constraints are frozen; no remediation mixed into design. |
| G1 Shared contract | Dino + Lifo | This canonical Limux contract is frozen with version-pinned HCOM input; identifiers, state dimensions, invariants, commands, and tests agree. |
| G2 Adversarial design review | Cross-family reviewers | Duplicate, stale-lease/incarnation, false-activity, restore, delivery, and compatibility failure modes are reviewed and findings reconciled. |
| G3a HCOM implementation PR | Dino/HCOM owner | Process-instance registry, fenced lifecycle/endpoints, metadata persistence, diagnostics, and exact targeting pass HCOM tests. |
| G3b HCOM independent verification | Independent HCOM reviewer | Current-head source review and adversarial runtime tests pass; no live binary activation. |
| G4 Limux preview implementation | Lifo/Limux owner | Runtime incarnation, caller provenance, exact pane targeting, restore suppression, UI/CLI parity, and automated cross-plane integration against the matching contract pass in isolated preview. |
| G5 Real-environment verification | Dino + Lifo | Real crash/restart, duplicate, delivery, PTY stop, relay death, stale endpoint, and nonce smokes pass in isolated preview alongside an unaffected daily driver. |
| G6 Promotion/activation | Operator | Operator explicitly approves promotion after reviewing G5 evidence; only then may global guidance/skill promotion occur. |

Global guidance or skill promotion occurs only after G6. No repository may
claim joint compatibility unless both sides report the same contract version.

Limux TaskMaster 23 remains the joint tracker through contract freeze and Limux
implementation/verification. HCOM G3a/G3b uses a separate HCOM PR/tracker
pointer linked into TaskMaster 23. The source-input PR does not complete this
task; this canonical incorporation completes G1 only.

## 12. Compatibility and Migration

- Existing HCOM names migrate to current aliases of generated stable
  `identity_id` values; migration records provenance and collision state.
- Existing `instances`, stopped events, and transcripts remain preserved.
- New process-instance storage is additive and migration-safe; schema migration
  requires normal HCOM DB backup and release gates.
- Existing `hcom list --json` fields remain, but ambiguous identities add a
  multi-instance collection and `duplicate_quarantined` warning.
- Existing `hcom send @name` remains an identity-level durable enqueue even
  with duplicate or zero live instances.
- Existing identity-level process mutations retain behavior during a staged
  compatibility period only for exactly one proven current instance. Before G6
  they expose plans/warnings; fail-closed mutation becomes default only after
  explicit operator activation.
- Limux stable behavior is not replaced during development; G4 uses preview.
- Contract version negotiation and JSON `schema_version` fail closed only for
  cross-plane mutation when versions are incompatible. Unrelated identity
  messaging and read-only raw inspection continue.

Machine-readable errors are stable across JSON and CLI clients:

| Error code | Exit class | Meaning |
|---|---:|---|
| `HCOM_TARGET_AMBIGUOUS` | 2 | Recoverable state conflict; exact current target required |
| `HCOM_STALE_SELECTOR` | 2 | Selector or lease changed before mutation |
| `HCOM_CONTRACT_MISMATCH` | 2 | Cross-plane mutation unsupported by negotiated version |
| `HCOM_AUTH_DENIED` | 1 | Caller/capability/authority check failed |
| `HCOM_PARTIAL_LAUNCH` | 3 | Side effect occurred but activation failed; reconciliation required |

Exit `0` means the requested operation succeeded (including identity enqueue);
exit `1` is a hard invalid/unsupported/authorization failure; exit `2` is a
recoverable state conflict; exit `3` signals a retained partial side effect.
An old client against a new server cannot bypass fencing and receives the
appropriate named error. A new client against an old server refuses exact
cross-plane mutation when capability negotiation is absent.
Automation SHOULD branch on the named machine-readable error code, not only the
coarser process exit class shared by several conditions.

## 13. Retention and Sensitive-Field Policy

- Raw HCOM capability tokens, relay credentials, message text, transcript
  content, and nonce values are never stored in the correlation view. Only a
  non-secret capability reference/fingerprint and receipt are retained.
- Default CLI/UI output exposes endpoint kind, freshness, and status, not local
  socket paths, endpoint addresses, process ancestry, or environment values.
  Sensitive local detail requires explicit manager-authorized diagnostic mode
  and is never relayed cross-device by default.
- Active launch/endpoint/pane details remain while needed for fencing. Stopped
  process details and conflict evidence have a configurable bounded retention;
  the proposed default is 30 days, after which identifiers and sensitive local
  paths are redacted into a minimal lifecycle summary.
- Identity-scoped owner/scope/lifetime/closeout/authority/handoff metadata
  remains until explicitly revised or the identity is archived.
- Activity observations retain metadata only: source, sequence/turn ID,
  dimension, timestamps, expiry, and confidence. They never inspect message or
  transcript content.

## 14. Explicit Non-Goals

- Changing relay trust or cryptographic authority.
- Deleting stale hook-trust records as part of the design task.
- Parsing message or transcript content to infer work activity.
- Treating CPU use or terminal output as proof of model work.
- Auto-killing duplicate processes without exact selection and policy.
- Global skill activation before implementation and real-environment gates.
- Replacing existing Limux stable runtime during preview validation.

## 15. Implementation Planning Decisions

1. Exact additive persistence tables, indexes, and migration/rollback sequence
   for stable identities, associations, launch instances, leases, and outbox.
2. Source-specific observation TTL values and the negotiation/default policy.
3. Contract transport between HCOM and Limux: environment bootstrap plus local
   RPC, event stream, or both. Restored environment alone is already excluded.
4. Final exact-instance CLI names and selector ergonomics after usability and
   compatibility testing; identity-level `hcom send` semantics and honest
   default delivery-level output are resolved requirements.
5. Whether the proposed 30-day detail retention is accepted or configured to a
   different bounded operator default.

These are inputs to linked G3/G4 PRDs and tasks. Operator Option A authorizes
planning, not implementation.

## 16. Operator Decision

On 2026-07-13, the operator selected **Option A - Approve consolidation**.
This authorizes:

1. this canonical Limux incorporation;
2. linked HCOM G3 and Limux G4 implementation planning tasks;
3. continued review and estimation inside the existing G0-G6 gate map.

It does **not** authorize implementation, merge, installation, runtime
activation, stable/daily-driver replacement, global guidance promotion, or G6
promotion. Those actions retain their independent owner, review, preview,
real-environment, and operator gates.
