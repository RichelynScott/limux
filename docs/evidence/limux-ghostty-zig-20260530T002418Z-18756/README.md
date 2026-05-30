# Limux Ghostty/Zig Execution Evidence

Date: 2026-05-29 20:24 EDT
Run ID: `20260530T002418Z-18756`
Review artifact: `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md`
Approved artifact SHA256: `dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`

## Result

Status: complete, with execution-wrapper deviation documented in `FYI.md` and
`HANDOFF.md`.

- Built `ghostty/zig-out/lib/libghostty.so`.
- Verified Ghostty submodule commit `81ab8ffa90185221782baf785e85387321e16f8d`.
- Confirmed no nested Ghostty submodules were present.
- Ran `CARGO_NET_OFFLINE=true cargo test --locked -p limux-host-linux surface_send_text_response`.
- Focused host test result: 2 passed, 0 failed, 186 filtered out.

## Evidence Files

| File | Contents |
|---|---|
| `zig-build.log` | Ghostty Zig build output. Empty because the successful build produced no stdout/stderr through `tee`. |
| `libghostty-readelf-dynamic.log` | Dynamic section for generated `libghostty.so`. |
| `libghostty-ldd.log` | Runtime library resolution for generated `libghostty.so`; no `not found` entries were present. |
| `cargo-test-host-send-text.log` | Locked offline host test output. |
| `final-git-status.txt` | Final git status captured by the approved block. |

## Wrapper Deviation

The shell extraction wrapper captured all `bash` fences in the review document,
not only the approved v2 command block. It first ran the illustrative README
block:

```bash
git submodule update --init --recursive
(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)
```

The build line failed immediately because `zig` was not on `PATH`. The approved
v2 command block then ran successfully. Follow-up inspection found the top-level
`ghostty` submodule at the pinned commit and no nested Ghostty submodules.
