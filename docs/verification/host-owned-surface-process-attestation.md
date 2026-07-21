# Host-Owned Surface/Process Attestation

This document defines the evidence contract for proving that an interactive
agent process belongs to one exact Limux Surface/Pane and one exact Limux host
runtime. It is a verification contract, not launch, model, provider, approval,
promotion, or teardown authority.

Use this contract only after a separate authorization record names the target,
the allowed read-only checks, the evidence destination, and the people or
systems responsible for execution and review. Do not infer authorization from
the existence of this document.

## Definition

A process is **host-owned by a Limux Surface/Pane** only when all of the
following refer to the same live launch:

- the selected Limux runtime channel, socket, install identity, and
  `limux-host` process;
- the workspace, pane, active tab surface, and exported `LIMUX_*` tuple;
- the target process identity, including PID and process start identity;
- the complete parent chain from the target process to that exact
  `limux-host` process;
- the authoritative hcom name, native session UUID, transcript binding, and
  live control endpoints; and
- a fresh request/reply nonce delivered to that exact hcom instance.

Matching display text, a process name, a PID by itself, inherited environment
variables, the focused pane, or an hcom name by itself is not sufficient.

## Authorization Record

Before any check, freeze an authorization record with:

1. A unique attestation ID, issue time, expiry, executor, and independent
   reviewer.
2. The permitted phase: `observer-pid-visibility`, `surface-attestation`, or
   `pane-preserving-teardown`. Authorization for one phase does not authorize
   another.
3. The exact Limux launcher, runtime channel, socket, installed source SHA or
   install ID, and expected `limux-host` executable.
4. The exact workspace, pane, surface, and tab references. Focus-based
   fallback is forbidden.
5. The expected command argv, working directory, hcom name, native session
   UUID when already known, and allowed environment keys. Do not record secret
   environment values.
6. An allowlist of read-only commands or APIs and the evidence directory.
7. Capture limits and redaction rules. Default screen capture is a small,
   bounded region sufficient to identify the session; full scrollback,
   transcript content, environment dumps, and credentials are forbidden.
8. Time, CPU, memory, process-count, and retry limits.
9. A statement that mismatch, ambiguity, timeout, or missing evidence yields a
   non-PASS verdict and grants no follow-on authority.

Without this frozen record, the gate is `NOT-AUTHORIZED` and no check runs.

## Gate O: Observer Host-PID Visibility

This gate is required when process ancestry will be observed from an isolated
observer whose host PID visibility has not already been proved. It is separate
from the Limux surface attestation.

The authorization record must additionally bind:

- the exact observer runtime/engine and version;
- the reviewed absolute-path supervisor or wrapper and its source and
  executable hashes;
- an already-present image or runtime artifact by digest, with no implicit
  pull or dependency acquisition;
- an inert payload and command hash that does not invoke Limux, hcom, an agent,
  a model, a provider, a credential, or a product workload;
- network disabled, no secrets, no broad host mounts, no host home directory,
  no container-engine socket inside the target, non-root execution, dropped
  capabilities, and `no-new-privileges`; and
- evidence retention and disposition. The visibility check itself has no
  cleanup or delete authority.

PASS requires all of the following:

1. The engine reports one numeric, nonzero host PID for the inert target.
2. The observer can read only the authorized process metadata, normally
   `/proc/<pid>/status`, `cmdline`, `stat`, `cgroup`, and `ns/pid`, while the
   target is alive.
3. Engine/container identity, an inert nonce, and `/proc/<pid>/stat` start time
   correlate the observation so a stale or reused PID cannot pass.
4. A nonexistent-PID negative control fails as expected.
5. Host `/proc` is not mounted into the target to manufacture visibility.
6. Evidence binds engine/version, wrapper hashes, artifact digest, payload
   hash, target identity, PID/start time, paths read, timestamps, and exit
   statuses.

If the selected observer cannot see the engine-reported host PID, the verdict
is `WAIT-BLIND(engine_unobservable)`. It must never be converted to PASS from
container-local PID data.

## Gate L: Limux Surface/Process Ancestry

Run this gate only after launch authority, if any, has already been granted and
the expected target is live. The attestation itself remains read-only.

### 1. Freeze Runtime Identity

Record the output identity from the selected launcher using the supported
top-level CLI surfaces, including `--version`, `target-info`, and `doctor
--json` when authorized. Resolve the launcher and host executable without
assuming that `limux`, `limux-stable`, and a preview channel share a socket or
process.

Record the `limux-host` PID, executable identity, and process start time. PID
alone is not a stable identity.

### 2. Freeze Surface/Pane Identity

Resolve the exact workspace and list its panels. The live inventory must map
the expected workspace, pane, active tab, surface, and socket to the frozen
runtime. Read only the named surface; never substitute the focused surface.

The target process must expose exactly these non-secret keys with matching
values:

- `LIMUX_WORKSPACE_ID`
- `LIMUX_PANE_ID`
- `LIMUX_SURFACE_ID`
- `LIMUX_TAB_ID`
- `LIMUX_SOCKET`

Capture only those allowlisted keys. Do not dump the full process environment.

### 3. Freeze Target Process Identity

Record the target PID, process start time, executable, and argv. Compare argv
to the frozen expected command byte-for-byte after accounting for an explicitly
reviewed launcher transformation. Ambient arguments, provider/model fallback,
profile changes, or an unexpected home/config source are mismatches.

### 4. Prove Ancestry

Walk the parent chain using PID plus process start identity at every hop. PASS
requires the chain to reach the exact frozen `limux-host` process and
executable for the selected channel.

Fail when:

- a PID disappears, changes start identity, or is reused during capture;
- ancestry reaches an unrelated terminal host or bypasses `limux-host`;
- the chain reaches a different Limux channel or host process;
- Windows Terminal or another external host owns a concurrent attachment; or
- the observer cannot see enough of the chain to prove ownership.

An unobservable chain is `WAIT-BLIND`, not PASS.

### 5. Prove hcom Binding

Use the hcom JSON diagnostics for the exact name/instance. Require:

- one nonempty authoritative native session UUID;
- the expected name and working directory;
- a bound transcript path without reading transcript content;
- `process_bound`, `live_delivery_available`, and `term_available` true;
- no unexpected control or binding warnings; and
- exactly one live native client for the authoritative UUID.

Send one fresh, single-use request nonce to the exact hcom instance. Retain the
request and reply event IDs, timestamps, target identity, and a hash or
fingerprint of the nonce; do not retain the raw nonce in long-lived reports.

### 6. Correlate And Decide

PASS is conjunctive. The runtime, surface, process, ancestry, environment,
hcom, and nonce evidence must all identify the same live launch. A PASS report
must bind the evidence manifest hash and the frozen authorization record.

Use one of these non-PASS verdicts when appropriate:

| Verdict | Meaning |
|---|---|
| `NOT-AUTHORIZED` | The required phase-specific authorization is absent or expired. |
| `TARGET-AMBIGUOUS` | The exact runtime, surface, process, or hcom instance cannot be selected uniquely. |
| `WAIT-BLIND` | Required host PID or ancestry evidence is not observable. |
| `PID-IDENTITY-MISMATCH` | PID/start-time correlation changed or indicates reuse. |
| `ANCESTRY-MISMATCH` | The chain does not reach the frozen `limux-host`. |
| `BINDING-INCOMPLETE` | hcom identity, transcript, process, terminal, or live delivery is incomplete. |
| `DUPLICATE-CLIENT` | More than one native client is attached to the authoritative UUID. |
| `NONCE-FAILED` | The exact-instance round trip did not complete or cannot be correlated. |
| `EVIDENCE-INCOMPLETE` | A required field, hash, timestamp, or negative control is missing. |

No verdict from this gate grants model execution, packet/GO use, promotion,
merge, provider/config mutation, or global activation.

## Gate T: Pane-Preserving Teardown

Teardown requires a separate authorization record after Gate L evidence is
frozen. Before stopping anything, capture the current host, workspace, pane,
surface, tab, target PID/start time, and hcom instance again. Stop only the
attested worker through its exact control surface. Do not kill or restart the
Limux host and do not close the pane.

PASS requires:

1. The attested worker and its hcom live endpoints stop.
2. No different process or newer PID/start-time identity is stopped.
3. The same Limux host, workspace, pane, and surface remain present.
4. A bounded screen read shows the surviving shell or explicitly expected
   post-worker state in that same surface.
5. Post-teardown inventory and evidence manifest hashes are recorded.

If the worker cannot be selected uniquely, stop and preserve evidence. Do not
use broad name-based kill, host restart, workspace close, or pane close as a
fallback.

## Evidence Manifest

The durable report should contain metadata and hashes, not sensitive content:

- authorization ID/hash, phase, executor, reviewer, and timestamps;
- Limux launcher, version, source/install identity, channel, socket, host
  executable identity, PID, and process start time;
- workspace, pane, surface, and tab references;
- target executable identity, argv hash, PID, process start time, and the five
  allowlisted `LIMUX_*` values;
- ancestry hops as PID, start time, executable identity, and parent relation;
- hcom name, native UUID, transcript-path hash, binding booleans, client count,
  and request/reply event IDs;
- observer visibility evidence when Gate O applies;
- before/after inventory when Gate T applies;
- per-check exit status, negative-control result, final verdict, and manifest
  hash.

Never store credentials, full environment output, raw transcript text, full
scrollback, raw nonce values, or unrelated process command lines.

## Related Limux Mechanics

- `skills/limux-a2a/SKILL.md` covers exact Surface/Pane selection, hcom resume,
  one-client verification, nonce round trips, and wrong-session handling.
- `skills/limux-use-guide/SKILL.md` covers runtime channels, diagnostics, and
  operator-visible Limux/hcom recovery.
- `docs/future-improvements/hcom-limux-session-pane-visibility-restart-design-20260712.md`
  records the broader future lifecycle design. Planned lifecycle APIs in that
  document must not be treated as currently implemented attestation evidence.
