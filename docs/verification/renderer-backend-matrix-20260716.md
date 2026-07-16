# Renderer Backend Matrix Evidence - 2026-07-16

## Scope

This record covers TaskMaster tag `limux-resource-crash-20260716`, Task 2. All
backend launches used isolated sockets, session directories, XDG directories,
and preview channel names. No daily-driver environment, install, state, or log
was mutated. The live incident log was not opened or scanned.

Source branch: `lifo/renderer-diagnostics-task2-20260716`

## Daily-Driver Baseline

Read-only process evidence for the current daily-driver host showed:

- renderer policy source disables GLES and Vulkan before GTK initialization;
- `/dev/dxg` exists and `nvidia-smi` sees an NVIDIA GeForce RTX 4060 Ti;
- `/dev/dri` does not exist under this WSL runtime;
- the host process held no `/dev/dxg` or `/dev/dri` descriptor;
- eight hot `llvmpipe` worker threads were present;
- approximately 103-105% CPU, 5.6-5.9 GiB RSS, and about 0.8 GiB swap.

This indicates software GL selection despite an available WSL GPU path.

## Preview Matrix

| Candidate | Requested policy | Selected GTK renderer | Software evidence | GPU FD | Result |
|---|---|---|---|---|---|
| WSL D3D12 GL | `GSK_RENDERER=gl`, `GALLIUM_DRIVER=d3d12` | `GskGLRenderer` | none | `/dev/dxg` | PASS |
| Desktop GL | `GSK_RENDERER=gl` | not separately forced in this round | daily driver resolves to llvmpipe | none on daily driver | BASELINE / NEEDS ISOLATED RECHECK |
| Software GL | `GSK_RENDERER=gl`, `LIBGL_ALWAYS_SOFTWARE=1`, `GALLIUM_DRIVER=llvmpipe` | `GskGLRenderer` | env and llvmpipe thread indicators | none required | PASS AS FINAL FALLBACK |

The queryable fallback order is:

1. `wsl-d3d12-gl`
2. `desktop-gl`
3. `software-gl`

The matrix is metadata and preview policy. It does not change the daily-driver
default or claim that GTK can switch a failed backend in-process after renderer
initialization.

## Measured Results

### D3D12, one workspace

- Healthy, realized terminal surface.
- `renderer_diagnostics.status = captured`.
- `selected_renderer = GskGLRenderer`.
- `is_software_fallback = false`.
- `/dev/dxg` descriptor present; no llvmpipe threads.
- Approximately 310 MiB RSS and 0-2% CPU across short samples.

### D3D12, ten-workspace Task 1 fixture

- Ten restored workspaces; active terminal healthy and realized.
- `selected_renderer = GskGLRenderer` and no software fallback indicators.
- `/dev/dxg` descriptor present; no llvmpipe workers.
- `320,728 KiB` RSS, `0 KiB` swap, and 21 threads.
- 0-3% CPU across five one-second samples.

### Explicit software fallback, one workspace

- Healthy, realized terminal surface.
- `selected_renderer = GskGLRenderer`.
- `is_software_fallback = true`.
- Indicators: `env:LIBGL_ALWAYS_SOFTWARE`,
  `env:GALLIUM_DRIVER=llvmpipe`, and `thread:llvmpipe`.
- Approximately 344 MiB RSS and 0-3% CPU with `LP_NUM_THREADS=2`.

## Default-Selection Proposal

Keep the daily-driver renderer policy unchanged during this incident stage.
For the next isolated preview round, prefer `wsl-d3d12-gl` only when `/dev/dxg`
and the Mesa D3D12 driver are present. Treat startup or health-check failure as
a process-level fallback to `desktop-gl`, then `software-gl`; never attempt to
replace GTK's renderer after initialization.

Promotion remains gated on:

- a bounded process-level preview runner or equivalent documented operator
  sequence that exercises a deliberately failing candidate and the fallback;
- a larger real-session input, resize, clipboard, and terminal-content matrix;
- Task 3 bounded logging, so backend failures cannot recreate unbounded logs;
- exact-head review and an explicit runtime promotion decision.

## Verification

- Renderer diagnostics tests: 5 passed.
- `cargo test -p limux-host-linux`: 378 passed.
- `cargo clippy -p limux-host-linux --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
