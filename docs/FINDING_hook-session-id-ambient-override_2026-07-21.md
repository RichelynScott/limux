# FINDING — `hook_session_id()` prefers ambient env over explicit payload (cross-lane misattribution + non-deterministic quality gate)

**Found by:** tutu (`LIMUX_MGR`) · 2026-07-21
**Found while:** verifying the H1 `read-screen` fix (PR #82) against the full test suite
**Status:** **RESOLVED** in PR #82 (commit `38f5254`). Precedence decision recorded below.
**Severity:** Medium-High (correctness + gate integrity). **Size:** S–M.

## RESOLUTION (2026-07-21, tutu as `LIMUX_MGR`)

**Decision: payload-first.** Every explicit identity carried by the payload
(`session_id`, then the `transcript_path` stem) is consulted **before** any
ambient environment value.

Rationale: a hook payload describes an event belonging to one specific session;
the environment describes whichever session invoked the CLI. In a multi-agent
workspace — this product's core use case — those are routinely different
processes, and `limux_env_value` walks ancestor process environments as well.
Ambient-first therefore misattributes across lanes. Explicit request data must
beat ambient inference, which is the same principle the H1 fix applied.

Also extracted `hook_session_id_with_env(payload, env_lookup)` so the ordering is
tested through an injected lookup rather than inheriting the ambient environment
of whoever runs the suite. The single production call site is unchanged.

**Gate impact:** the full workspace suite now reports **602 passed / 0 failed**
from inside a Claude session. Previously the outcome depended on the runtime.

**Remaining follow-up (not done here):** sweep other `limux_env_value` call sites
for the same explicit-vs-ambient inversion.

## Two symptoms, one root cause

### 1. The quality gate is non-deterministic *by runtime*

`cli_arg_tests::hook_session_id_falls_back_to_transcript_stem` (`rust/limux-cli/src/main.rs`) fails when the suite is run from **inside a Claude session** and passes under **Codex**:

```
assertion `left == right` failed
  left:  Some("cd1a39d7-e1f4-4607-84bf-5f29f6e4c66e")   <- the RUNNING session's id (tutu)
  right: Some("268746f1-5a8f-471c-85db-dc50649c2f9c")   <- the payload's transcript stem
```

This reconciles a live contradiction: limu's audit (`docs/REPO_AUDIT_limux_2026-07-21.md`, Codex/`gpt-5.6-sol`) recorded `./scripts/check.sh` passing with **597 tests**; the repo `CLAUDE.md` warns this same test is failing. **Both are correct** — the result depends on who runs it. A gate whose outcome depends on the operator's runtime cannot be trusted as a merge signal.

### 2. The underlying behavior is a real correctness risk

```rust
fn hook_session_id(payload: &Value) -> Option<String> {
    hook_str(payload, &["session_id", "sessionId", "sessionID"])
        .map(str::to_string)
        .or_else(|| limux_env_value("CLAUDE_CODE_SESSION_ID"))   // ambient
        .or_else(|| limux_env_value("CLAUDE_SESSION_ID"))        // ambient
        .or_else(|| limux_env_value("HERMES_SESSION_ID"))        // ambient
        .or_else(|| hook_session_id_from_transcript(payload))    // explicit payload data
        .filter(|value| !value.trim().is_empty())
}
```

**Ambient environment (steps 2–4) is preferred over the payload's own `transcript_path` (step 5).** And `limux_env_value` does not only read this process's environment — it falls back to `ancestor_env_value(name)`, walking **parent process** env.

So: a hook payload that describes session **A** (carrying A's `transcript_path`) but no explicit `session_id` will be attributed to whatever session **invoked the CLI** — session **B**. In a multi-agent Limux workspace, which is the product's core use case, **hook events can be recorded against the wrong agent lane.**

## Why this matters beyond one test

This is the **same defect class as H1** (fixed in PR #82): *implicit ambient context silently overriding explicit request data, producing cross-lane effects.*

- **H1** — ambient global focus overrode absent explicit target → cross-lane **disclosure**.
- **This** — ambient session id overrides explicit payload transcript → cross-lane **misattribution**.

Worth asking whether other call sites share the pattern. `limux_env_value`'s ancestor-walk makes it a broad surface: any code preferring it over explicit request data inherits the same risk.

## Open question for the owner (do not fix blind)

What *should* the precedence be? The test encodes one answer (payload transcript beats ambient env); the implementation encodes the opposite. One of them is wrong and it is not self-evident which:

- **Payload-first** is correct if hooks are handling events for *other* sessions (multi-agent). Implies reordering so `hook_session_id_from_transcript` precedes the env lookups.
- **Ambient-first** may be deliberate if hooks were designed to self-identify when payloads are unreliable. Then the *test* is wrong and should be rewritten to control the environment rather than inherit it.

Either way the test must **not** depend on ambient environment — it should inject/clear the env vars so the gate is deterministic across runtimes.

## Recommendation

1. Decide the precedence semantics (owner call — likely payload-first for multi-agent correctness).
2. Make the test environment-independent regardless of that decision, so the gate stops depending on who runs it.
3. Sweep other `limux_env_value` call sites for the same explicit-vs-ambient inversion.

## Not claimed

- Not claimed that hook misattribution has actually occurred in production — the mechanism is verified by reading source and observing the test; no live incident is cited.
- Not claimed which precedence is intended; git history/PR archaeology was not performed.
- Not claimed to affect non-hook code paths without a call-site sweep.
