# Renderer-Owned Terminal Child Environment Seam

Date: 2026-08-12
Investigator: subagent `/root/renderer_owned_seam`
Limux branch: `gula/renderer-supervisor-v2-20260812`
Limux source HEAD inspected: `ffafacb74e403964205be4ce29440f4eb22dc6ab`
Ghostty submodule HEAD inspected: `81ab8ffa90185221782baf785e85387321e16f8d` (`origin/linux-embedded-apprt`)
Mode: bounded read-only source investigation; only this report was created. No source, peer-owned artifact, runtime, install, or `/tmp` path was modified or used.

## Contract and conclusion

Contract: identify the smallest way for a fresh-process renderer supervisor to give the Limux GUI host `GSK_RENDERER`, `GALLIUM_DRIVER`, and `LP_NUM_THREADS` without those supervisor-injected variables appearing in terminal child environments, while treating `ghostty/` as read-only from the Limux layer.

**Decisive result: strict absence is not implementable in Limux-owned Rust alone through the current embedded Ghostty C API.** The current surface API accepts only non-null key/value overlays. The embedded runtime rebuilds a child environment from the host process and then applies those overlays with `put`; it exposes neither a key-removal list nor an environment-filter callback. An empty-string overlay is possible entirely in Limux, but leaves `KEY=` present and is therefore containment, not removal.

**Recommended smallest shippable seam:** upstream a layout-preserving removal semantic in the owned `am-will/ghostty` embedded API: treat a null `ghostty_env_var_s.value` as a per-surface removal tombstone, retain the requested keys on the embedded surface, and remove them from the `EnvMap` returned by `defaultTermioEnv`. Then update only Limux's FFI typing/construction and pass tombstones for the renderer keys the supervisor actually injected. This reuses the existing `ghostty_env_var_s` layout rather than adding a new C-struct field, but it is still a Ghostty semantic change and must land in the Ghostty owner/upstream lane before Limux can depend on it.

Until that seam lands, the exact requirement remains **BLOCKED on the embedded Ghostty API**. Do not substitute an all-pane command wrapper or a process-global post-GTK `unsetenv` mutation.

## Phase 1 - root-cause trace

### 1. The renderer policy must exist in the GUI host process

- Limux establishes GTK/GDK rendering policy before toolkit/application startup: `rust/limux-host-linux/src/main.rs:520-545` appends the GTK renderer controls and initializes Ghostty before constructing the application.
- The isolated renderer runner likewise clears inherited renderer controls and injects the chosen backend only into each fresh host process: `scripts/renderer-backend-preview/renderer-backend-preview.sh:213-242` defines each backend and `scripts/renderer-backend-preview/renderer-backend-preview.sh:322-352` performs the clean host launch.
- Runtime diagnostics read those process variables directly (`rust/limux-host-linux/src/window/renderer_diagnostics.rs:10-19`, `84-116`) and capture the selected GTK renderer after the window maps (`rust/limux-host-linux/src/window.rs:213-226`; `rust/limux-host-linux/src/window/renderer_diagnostics.rs:339-358`).

Consequence: clearing the policy before GTK/GSK selects a renderer defeats the renderer selection and its diagnostics. The host needs the policy; only the terminal-child environment must be filtered.

### 2. Each embedded terminal copies the GUI host environment

- Limux creates the Ghostty surface on `GLArea::realize`, calls `make_current`, then calls `ghostty_surface_new`: `rust/limux-host-linux/src/terminal.rs:2161-2182`, `2237-2243`.
- The embedded Ghostty surface's `defaultTermioEnv` calls `internal_os.getEnvMap`, so it starts from the GUI host's live process environment: `ghostty/src/apprt/embedded.zig:1016-1019`.
- Core Ghostty asks that runtime surface for the environment and passes it to `termio.Exec`: `ghostty/src/Surface.zig:628-669`.
- `termio.Exec` uses that map as the child environment (`ghostty/src/termio/Exec.zig:622-625`) and later passes it to the spawned command (`ghostty/src/termio/Exec.zig:1004-1013`).

Therefore a renderer value inherited by the GUI host reaches every terminal child unless the embedded surface removes it from this per-child map.

### 3. The current C API can add or overwrite but cannot remove

- `ghostty_env_var_s` contains two required pointers, `key` and `value`, and `ghostty_surface_config_s` only exposes an array plus count: `ghostty/include/ghostty.h:416-419`, `447-463`.
- Limux's raw FFI mirrors the same shape with `*const c_char` value pointers and no removal/filter member: `rust/limux-ghostty-sys/src/lib.rs:229-252`.
- Limux converts `TerminalOptions.extra_env` into C strings and forwards only key/value entries: `rust/limux-host-linux/src/terminal.rs:1928-1949`, `2203-2224`.
- Embedded Ghostty assumes every value pointer is non-null, converts it to a string, and inserts it into `config.env`: `ghostty/src/apprt/embedded.zig:421-427`, `572-583`.
- `termio.Exec` applies the resulting override map with `env.put`, never `remove`: `ghostty/src/termio/Exec.zig:807-813`.

Bounded negative search over `ghostty/include/ghostty.h`, `ghostty/src/apprt/embedded.zig`, and `rust/limux-ghostty-sys/src/lib.rs` found no environment-removal list, unset function, or child-environment filter callback. The runtime callback struct contains wakeup, action, clipboard, and close callbacks only (`ghostty/include/ghostty.h:974-1005`; `rust/limux-ghostty-sys/src/lib.rs:375-398`).

### 4. Ghostty already demonstrates the correct boundary in its native GTK runtime

Ghostty's native GTK surface gets a process environment map and removes toolkit-private variables, including `GSK_RENDERER`, before starting the terminal child: `ghostty/src/apprt/gtk/class/surface.zig:1527-1543`. The embedded runtime does not mirror that filter.

This is strong pattern evidence that the correct boundary is `defaultTermioEnv`, not the shell command and not the GUI process environment.

## Phase 2 - alternatives and compatibility analysis

### A. Empty-string overrides in `TerminalOptions.extra_env`

This is the smallest Limux-only containment. Limux already uses empty-string overlays to clear inherited HCOM identity values (`rust/limux-host-linux/src/pane.rs:1200-1217`, `1535-1556`). The same mechanism could override the three renderer values with empty strings.

It does **not** meet strict removal:

- embedded Ghostty inserts the empty string into the map rather than interpreting it as a tombstone (`ghostty/src/apprt/embedded.zig:572-583`);
- `termio.Exec` then inserts that empty value into the final child environment (`ghostty/src/termio/Exec.zig:807-813`);
- programs that distinguish absent from present-empty would still observe the variable.

Verdict: acceptable only as an explicitly named temporary containment if the operator relaxes the requirement from "absent" to "supervisor value does not survive." It must not be claimed as the final child-env removal seam.

### B. Force every terminal through an `env -u ...` command wrapper

Rejected. `ghostty_surface_config_s.command` is not a transparent launcher hook:

- setting it makes the command shell-expanded and automatically enables wait-after-command (`ghostty/src/apprt/embedded.zig:472-480`, `563-569`);
- it overrides Ghostty's configured/default command selection, which otherwise honors configured commands, `SHELL`, and the passwd shell (`ghostty/src/config/Config.zig:1131-1160`, `4504-4560`);
- shell integration wraps/detects the selected command (`ghostty/src/termio/Exec.zig:750-805`);
- shell and direct-command execution have materially different argv behavior (`ghostty/src/termio/Exec.zig:1396-1415`, `1537-1580`);
- Limux already uses this command field for restored-agent startup commands (`rust/limux-host-linux/src/pane.rs:1557-1585`; `rust/limux-host-linux/src/terminal.rs:2226-2235`).

Risks include lost custom shell/command behavior, changed wait/exit behavior, degraded shell integration, extra shell nesting, and interference with restored-agent commands. This violates the surgical-change requirement.

### C. Remove the variables from the GUI process after GTK initializes

Rejected. Terminal surfaces are created during window realization and immediately start their child (`rust/limux-host-linux/src/terminal.rs:2161-2237`). The window also establishes asynchronous DBus watches before presenting (`rust/limux-host-linux/src/window.rs:3454-3461`, `3562-3566`), and the application has explicit worker-thread paths (`rust/limux-host-linux/src/window.rs:409-420`). A process-global environment mutation after toolkit initialization is a race-prone cross-thread operation and would also erase the renderer policy before the delayed diagnostics capture.

### D. Put `env = KEY=` in Ghostty configuration

Rejected for ambient removal. The config parser treats an empty value as removal from the **override map** (`ghostty/src/config/Config.zig:1268-1304`; `ghostty/src/config/RepeatableStringMap.zig:20-57`), but the child base environment is separately copied from the process and `termio.Exec` only applies surviving overrides with `put` (`ghostty/src/Surface.zig:628-669`; `ghostty/src/termio/Exec.zig:807-813`). Removing a key from the override map does not remove the same key from the ambient base map.

### E. Add a new removal array to `ghostty_surface_config_s`

Functionally sound but larger and more ABI-sensitive than necessary. A nullable-value tombstone can preserve the existing C struct layout. Prefer the tombstone unless upstream maintainers require an explicit removal array for API clarity.

## Phase 3 - recommended seam and single hypothesis

Hypothesis: if embedded Ghostty treats `ghostty_env_var_s.value == NULL` as a per-surface tombstone and applies those tombstones to its freshly copied `defaultTermioEnv`, then the GUI host can retain renderer policy and diagnostics while terminal children receive no renderer variables, without modifying command selection or mutating the process environment.

Minimum implementation shape:

1. **Ghostty owner/upstream lane**
   - Document `ghostty_env_var_s.value == NULL` as "remove this key from the terminal command environment."
   - Make embedded `EnvVar.value` nullable.
   - Duplicate and retain removal keys on the embedded surface for its lifetime.
   - In embedded `defaultTermioEnv`, remove those keys after `getEnvMap` and before returning the map.
   - Keep non-null entries' current override behavior unchanged.

2. **Limux-owned lane after the Ghostty commit exists**
   - Keep `ghostty_env_var_s` layout unchanged; add a constructor/path that deliberately emits `value = null` for removal entries.
   - Extend `TerminalOptions` with a distinct removal collection rather than encoding removals as empty strings.
   - Pass only keys selected by the renderer supervisor's validated policy, plus the internal policy marker if one is introduced. Do not accept arbitrary untrusted key names.
   - Keep the renderer variables in the GUI host process so `renderer_diagnostics` remains truthful.

3. **Provenance boundary**
   - The current preview runner injects renderer values without an origin marker (`scripts/renderer-backend-preview/renderer-backend-preview.sh:322-350`). A production supervisor must carry the exact injected-key set to the host through a validated internal policy representation; otherwise Limux cannot distinguish a supervisor value from a user's unrelated ambient value.
   - Scope must be "keys injected by this selected renderer policy," not "always erase these names," unless the operator explicitly chooses the broader product behavior.

This is the smallest design that preserves Ghostty command semantics, keeps renderer diagnostics valid, and makes the child environment claim mechanically testable.

## Public RED-test seams (define before implementation)

These are behavior seams, not private helper tests.

### RED-1: embedded C API child-environment contract

Using the public embedded surface API, start a terminal command with:

- host process containing all three renderer variables;
- one normal non-null env override;
- null-value tombstones for `GSK_RENDERER`, `GALLIUM_DRIVER`, and `LP_NUM_THREADS`.

Observe the actual launched command environment. Assert the three keys are absent, the normal override is present, and an unrelated inherited variable remains. This must fail on current Ghostty because `embedded.zig` dereferences every value pointer.

### RED-2: Limux production GTK bridge to actual terminal child

Launch a fresh isolated host through the production CLI/host path with a renderer policy, create or use a real terminal surface through the live bridge, and inspect the child through the supported `send`/`read-screen` surface. Assert:

- renderer diagnostics still records the requested policy and selected renderer;
- the terminal child reports all injected renderer keys absent, not empty;
- Limux pane identity variables remain present;
- an unrelated inherited variable is unchanged.

The production bridge is required because Limux explicitly distinguishes it from the standalone dispatcher (`AGENTS.md:160-169`).

### RED-3: supervisor-origin preservation

Run two fresh hosts:

1. supervisor injects `GALLIUM_DRIVER` and marks it as policy-owned: child must not receive it;
2. no supervisor policy owns the key, but the launching environment contains a user value: behavior must follow the operator-approved contract (recommended default: preserve it).

This test prevents accidental blanket scrubbing from silently changing user-launched workloads.

### RED-4: command semantics regression

With the removal policy active, verify through terminal behavior that:

- normal default shell startup still works;
- a configured Ghostty command still runs;
- a restored-agent `startup_command` still runs once;
- Ghostty shell integration remains active where it was active before.

No test should assert a private wrapper string; the public outcomes are the seam.

### RED-5: fresh-process fallback isolation

Using a fake host executable selected through the existing `LIMUX_HOST_BIN` host-resolution seam (`rust/limux-cli/src/main.rs:646-663`), make the first renderer candidate reject and the next accept. Assert different host PIDs, backend-specific environment per process, and no terminal/user session launched in the rejected probe. The existing host command already has an explicit spawn/wait boundary (`rust/limux-cli/src/main.rs:665-731`), so the test should observe process behavior rather than a private fallback helper.

## Semantic decisions the owner must freeze

1. **Absence versus empty:** recommended and assumed here is actual absence. Empty overlay is not equivalent.
2. **Origin:** recommended is remove only supervisor-owned renderer keys, carried by a validated internal policy. Blanket removal would also alter user-injected Mesa controls.
3. **Key set:** the immediate requested set is `GSK_RENDERER`, `GALLIUM_DRIVER`, and `LP_NUM_THREADS`. The existing diagnostics know eight renderer controls (`rust/limux-host-linux/src/window/renderer_diagnostics.rs:10-19`); expanding removal to the other five is a separate claim and must be justified by the selected supervisor policy.
4. **Rejected probe behavior:** a probe must not restore a real user session or launch agents. The current preview harness copies a session template and creates real surfaces, so it is validation tooling, not yet a production no-user-session probe (`scripts/renderer-backend-preview/renderer-backend-preview.sh:279-352`).

## Evidence integrity

Source hashes at investigation time:

| Path | SHA-256 |
|---|---|
| `ghostty/include/ghostty.h` | `1e8c7b40ec0270cf24d58e68aa8b3f3762af0fc3292fa507f880997e6bb06312` |
| `ghostty/src/apprt/embedded.zig` | `1afa5ecccbea64d10040f03fa68584f21fdca10fe55a797663b3e69071c7ca6e` |
| `rust/limux-ghostty-sys/src/lib.rs` | `25cb87730befb996ab259248732551e30d33eade2f6bde46da35cde818fd5791` |
| `rust/limux-host-linux/src/terminal.rs` | `bf715e3838d190a0c271ecc2feb1d88cd5b69834769b7e0297c9af396e957c92` |
| `rust/limux-cli/src/main.rs` | `06e9f0071bc48667b4fbcb83c4c70acc883a448f957545974d8cf9acc4048ad5` |
| `rust/limux-host-linux/src/window/renderer_diagnostics.rs` | `3337ef7f172da598dc2792ffa945c338ecb8c2033b792f767245a8d203effbdf` |

The repository rule is explicit: vendored `ghostty/` is read-only from Limux and integration must use the C API (`AGENTS.md:260-267`). No Ghostty or Limux source changes were made during this investigation.
