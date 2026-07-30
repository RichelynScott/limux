# LIMUX_MGR Decision: OMP `ask` Waiting State

- **Date:** 2026-07-30
- **Decision owner:** limu (`LIMUX_CODEX_MGR`)
- **Decision:** **A — GO, with corrected scope**
- **Request:** hcom `605271`
- **Decision response:** hcom `605290`
- **Source reviewed at:** `1c8d43bb9faf`

## Decision

Proceed with a first-class OMP-to-Limux lifecycle integration. Do not wait for
`tutu`, do not impersonate an existing agent family, and do not treat the
existing notification path as a mitigation.

The request's premise that W1.3 already exists end-to-end is not accurate.
`rust/limux-host-linux/src/agent_state.rs` contains the pure state engine, but
its module header explicitly defers GTK sidebar wiring,
`surface.agent_event`, and `agents-status` to the next PRD-G slice. The current
verification record likewise states that hooks/sidebar/socket/CLI work remains
under TaskMaster task 7.

Therefore Option A includes both:

1. Finish the native PRD-G live path: hook ingress, host state-store
   consumption, sidebar/attention rendering, socket exposure, CLI exposure,
   eviction, and end-to-end verification.
2. Add OMP as a first-class, receiver-only agent family and map its lifecycle:
   - `ask` waiting -> `NeedsInput`
   - answer/resume -> `Running`
   - cancel/end -> `Idle`

Limux must not install or mutate OMP configuration. OMP owns emission after the
Limux receiver contract is committed.

## Rejected Options

- **B — MITIGATE-FIRST:** rejected. OMP emits no Limux hooks today, so the
  notification route cannot work without OMP changes; spoofing an existing
  family would also corrupt agent identity and lifecycle semantics.
- **C — DEFER:** rejected. The work is bounded, operator-visible, and already
  belongs to the open PRD-G implementation surface.

## Owners and Ordering

1. **limu:** owns Limux TaskMaster task 7 sequencing and the receiver contract.
2. **rako:** owns OMP-side hook emission after the receiver contract lands.
3. **fire:** not assigned.
4. **tutu:** offline; not a gate.

## Acceptance

- A background OMP pane entering `ask` visibly becomes `needs-input` in the
  live Limux sidebar and raises the intended attention signal.
- Answer/resume and cancel/end clear the waiting state through explicit events.
- Completion remains distinguishable from waiting.
- Verification exercises the production host/control/sidebar path, including
  the Xvfb smoke path; pure state-engine tests alone are insufficient.
