# FYI.md

Append-only journal for significant Limux session decisions and implementation notes.

## 2026-06-10 - SCS Mutation-Wave Draft Dirty State
### What:
Recorded that SCS is dirty again after `e8ef33a` with a new untracked `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md` and tracked docs pointer edits.

### Why:
The previous Limux restart pointer correctly recorded the pushed `e8ef33a` closeout, but a successor must not mistake that commit for the current frozen packet if gumo's mutation-wave packet draft is still uncommitted.

### How:
Verified Limux remained clean, checked SCS dirty state, hash-checked the new mutation-wave packet plus related SCS artifacts, sent gumo a corrected read-only hcom review in `#28584`, recorded gumo's `#28606` ack that he is patching additional Claude-side findings, and patched only Limux-owned restart docs. No SCS files, host, VM, WSL, Docker, HNS, WinNAT, network, package, ISO, SCRIM, global-config, or runtime mutation was performed by Halo.

### Impact:
Successors should first check whether gumo has committed the mutation-wave packet draft. If not, continue from hcom `#28584` and gumo ack `#28606`; if yes, verify the new commit and hashes before updating Limux pointers. The next recommended formal scope is Wave A ISO intake packet review/freeze; Wave B offline Hyper-V baseline and Wave C network stage remain gated.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | hcom `#28584`

## 2026-06-10 - SCS PRD Acceptance Gate Commit Pushed
### What:
Recorded the pushed SCS docs-only closeout at `e8ef33a`, including PRD acceptance gates, tightened Hyper-V packet command guards, PRD acceptance review, and refreshed final hashes.

### Why:
The previous Limux handoff correctly warned that SCS was dirty, but gumo has now committed and pushed the SCS-owned docs. A zero-context successor needs the durable commit and verified hashes, not the transient dirty-draft caveat.

### How:
Verified SCS `main` aligned with `origin/main`, checked final SHA256 values for the active goal, Hyper-V packet, NAT plan, ISO intake plan, review record, PRD acceptance review, PRD-001, and HTML decision packet, then patched Limux-owned restart docs only. No SCS files, host, VM, WSL, Docker, HNS, WinNAT, network, package, ISO, SCRIM, global-config, or runtime mutation was performed by Halo.

### Impact:
Successors should start from SCS commit `e8ef33a`. The lab remains `WAIT`: formal `$mutation-script-wave`, Docker/HNS/WSL2 NAT decision before WinNAT, a frozen ISO download/use packet, exact execution-packet freeze, and explicit operator approval remain required before any host/VM/network/artifact mutation.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | SCS commit `e8ef33a` | hcom `#28386`

## 2026-06-10 - SCS Packet Hashes Stale During Gumo PRD Update
### What:
Recorded that the previous SCS `8ff345f` packet/hash checkpoint is no longer the current live state from Halo's view because gumo owns new uncommitted PRD/docs edits.

### Why:
A restarted Limux session must not mistake the earlier SCS hashes for a frozen packet set while gumo is actively patching the next review blockers.

### How:
Verified Limux remained clean, checked SCS dirty state, received gumo's hcom `#28096` ack, and updated Limux-owned restart docs only. No SCS files, host, VM, WSL, Docker, HNS, WinNAT, network, package, ISO, SCRIM, global-config, or runtime mutation was performed.

### Impact:
Successors should wait for or verify gumo's next SCS commit before updating hashes. Current open findings are ISO fail-closed checks before `Add-VMDvdDrive`, GUI NAT preflight scoping, network-only in-VM acceptance checks, and `project_isolation_lab/evidence/` tracking policy.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | hcom `#28096` | hcom thread `project-isolation-lab-goal`

## 2026-06-10 - SCS NAT/ISO Closeout Pushed
### What:
Recorded the pushed SCS docs-only WAIT closeout for the Hyper-V packet, NAT reconciliation plan, ISO artifact-intake plan, review record, and HTML decision packet.

### Why:
The operator is working toward a PC restart. A zero-context Limux successor needs the durable SCS commit and final hashes, not the earlier `937158e` state or transient uncommitted draft hashes.

### How:
Patched only Limux-owned restart surfaces after verifying SCS `main` at `8ff345f`, checking the final packet/NAT/ISO/review-record/HTML/ACTIVE_GOAL hashes, and confirming SCS is clean/aligned except unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`.

### Impact:
A resumed Limux session should start from SCS commit `8ff345f`. Packet/NAT/ISO remain `WAIT`; the next real gate is formal `$mutation-script-wave` on the exact frozen SCS packet set, with no execution approval implied.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | SCS commit `8ff345f` | hcom thread `project-isolation-lab-goal`

## 2026-06-10 - SCS Hyper-V Packet Review Gate Pushed
### What:
Recorded the pushed SCS Hyper-V packet review state in Limux-local restart docs.

### Why:
Gumo landed the SCS-owned follow-up commit after Halo's read-only packet review. A resumed Limux session needs the durable SCS commit and final hashes rather than the earlier `62440b6` baseline.

### How:
Updated `HANDOFF.md` and `docs/project-isolation-lab-goal.md` with SCS commit `937158e`, packet/review-record/HTML hashes, the review-record path, and the remaining WAIT gates. Halo did not edit SCS and did not run any host, VM, WSL, network, package, or runtime mutation.

### Impact:
A successor should start from SCS commit `937158e`, packet hash `3fc1404e8e5a0bcfa31fabc549a83bbb3b96bdd0f4191d561347d56c14e7c220`, review-record hash `31c216c85af3ce3580b3e7a616e82ef505ab78e4c271c39ca20819f3fa005d0e`, and HTML packet hash `11f5bf9afe48b78e4970077f780345d8fd961444a39ac3af1c735e63f4b1cf04`.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | hcom thread `project-isolation-lab-goal`

## 2026-06-10 - Project Isolation Lab Restart Pointers
### What:
Refreshed the Limux-local handoff and goal note with the current SCS Project Isolation Lab restart pointers, including the canonical active-goal file and the Hyper-V host mutation packet draft.

### Why:
The operator is preparing for a PC restart and needs zero-context continuity around the real active goal: full isolated Linux VM baseline, disposable full Linux VM factory, and later Firecracker microVM quarantine, with Limux only as a tool/acceptance case.

### How:
Kept SCS ownership intact and patched only Limux-owned docs. Recorded that the Hyper-V packet remains `WAIT`, that the WinNAT guest-network gap is now addressed in the draft, and that formal mutation review, packet hash, ISO provenance/hash, and explicit approval remain required.

### Impact:
A resumed Limux session should verify gumo's SCS commit status first, then continue coordination on the Project Isolation Lab instead of drifting back into Limux feature work.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`

## 2026-06-10 - SCS Hyper-V Packet Commit And WAIT Review
### What:
Recorded that gumo closed and pushed the SCS Project Isolation Lab docs lane at commit `62440b6`, including the canonical `ACTIVE_GOAL.md` and Hyper-V host mutation packet draft.

### Why:
The Limux session needs an accurate restart pointer for the real active goal and must not rely on stale "verify whether SCS committed" wording after gumo's closeout.

### How:
Verified the SCS commit/hash state from the Limux session and sent gumo a Codex single-reviewer system-mutation pre-exec review. The review decision remains `WAIT`, not formal `$mutation-script-wave`, with concrete must-fix findings around staged PowerShell flow, fail-closed preflight checks, evidence overwrite risk, placeholder validation, and deterministic first-boot network posture.

### Impact:
A resumed session should start from SCS commit `62440b6` and packet SHA256 `badc15cf44a8a6be8f56231b09899ee3c2e756ff0308dd6758b70ccd9d3ca678`, then continue toward formal mutation-script-wave review and ISO provenance rather than redoing the docs closeout.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | hcom thread `project-isolation-lab-goal`

## 2026-06-10 - Read-Only Lab Placeholder Discovery
### What:
Ran read-only WSL and hcom discovery for the Hyper-V packet placeholders.

### Why:
The packet still needs operator-specific values for the control-plane WSL distro and hcom sender-name check before it can move toward formal review. Resolving candidates without mutation reduces ambiguity while preserving the host-mutation gate.

### How:
Used `wsl.exe --list --verbose`, `wsl.exe --status`, `hcom --version --name halo`, and `hcom list --name halo`. No Hyper-V, VM, network, package, installer, or runtime mutation was run. A prior hcom message with shell-backtick expansion was corrected immediately in a follow-up message to gumo.

### Impact:
Candidate values are `CONTROL_PLANE_WSL_DISTRO=Ubuntu` and `HCOM_CHECK_NAME=halo`, with the caveat that `docker-desktop` is also running on WSL2 and therefore Docker/HNS/WSL2 NAT reconciliation remains a live stop condition before WinNAT creation.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | hcom thread `project-isolation-lab-goal`

## 2026-06-10 - Ubuntu ISO Artifact-Intake Candidate
### What:
Recorded a read-only candidate Ubuntu ISO provenance and checksum for the Project Isolation Lab persistent VM baseline.

### Why:
The Hyper-V packet requires exact Ubuntu ISO provenance and SHA256 before formal mutation review or execution approval can be considered.

### How:
Checked official Ubuntu release sources and fetched only `SHA256SUMS` plus `SHA256SUMS.gpg` into `/tmp`. Verified the checksum file signature in an isolated `/tmp` GnuPG home using Ubuntu CD Image Automatic Signing Key (2012), key ID `D94AA3F0EFE21092`, fingerprint `8439 38DF 228D 22F7 B374 2BC0 D94A A3F0 EFE2 1092`. The ISO itself was not downloaded.

### Impact:
Candidate artifact is `ubuntu-24.04.4-desktop-amd64.iso` with SHA256 `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e`. This is artifact-intake evidence only, not approval to download, boot, install, or execute the ISO.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md` | `https://releases.ubuntu.com/24.04/` | hcom thread `project-isolation-lab-goal`

## 2026-05-29 - Limux Agent-Team Protocol Safety And Resume Plan
### What:
Fixed the highest-risk `agent-team` behavior by moving generated protocol output from `AGENTS.md` to `LIMUX_AGENTS.md` by default, added `--protocol-path`, and documented the next zero-friction protocol discovery phase.

### Why:
The operator workflow depends on one visible Codex session plus one Claude Code session per project, often across 4+ projects with subagents, adversarial review, and hcom cross-team communication. Generated runtime protocol files must not clobber authoritative repo instructions.

### How:
Implemented and pushed `cec067f fix(cli): protect agent-team protocol output`; then ran a five-subagent brainstorm that recommended explicit instruction-source references instead of silent inheritance or copying. Created `HANDOFF.md` and refreshed the Limux vs Multica and Limux+hcom docs for morning resumption.

### Impact:
Existing repo `AGENTS.md` files are protected from default `agent-team` protocol generation. The next safe step is to make `LIMUX_AGENTS.md` easier for agents to discover by adding generated markers, detected instruction-source pointers, no-overwrite semantics, and a local extension file.

### Related:
`cec067f` | `HANDOFF.md` | `docs/cmux-parity-plan.md` | `docs/limux-hcom-workflow.md` | `docs/limux-vs-multica-decision-guide.md`

## 2026-05-29 - Next Steps Decision Packet
### What:
Created a dark-mode HTML decision packet for the operator to review current Limux status, next-step options, execution mode, skills, and acceptance criteria.

### Why:
The operator asked for an easier-to-read status update with selectable choices and a copy-back response before continuing implementation.

### How:
Used the `$html-decision-packet` pattern and current repo evidence from `HANDOFF.md`, `docs/cmux-parity-plan.md`, recent commits, and the install/security report. The packet defaults to the recommended path: implement Phase 5A zero-friction protocol discovery in the current session.

### Impact:
The next session can ask the operator to open `docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html`, copy selections back, and proceed without reconstructing the prior discussion.

### Related:
`docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html` | `a1447e7` | `HANDOFF.md`

## 2026-05-29 - Phase 5A Agent-Team Protocol Discovery
### What:
Implemented Phase 5A for `limux agent-team`: generated `LIMUX_AGENTS.md` files now include a stable generated marker, an `Instruction Sources` section for detected `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`, metadata for path/modified time/deterministic hash, and a documented `LIMUX_AGENTS.local.md` local policy sidecar.

### Why:
The operator selected Limux + hcom as the primary orchestration path and needed near-zero-friction discovery without hidden prompt inheritance, copying, or clobbering authoritative repo instruction files.

### How:
Used TDD. Added RED tests for marker output, instruction-source references without content copying, unmarked sidecar refusal, explicit force overwrite, and symlink refusal. Hardened protocol writes with preflight validation, atomic temp-file replacement, no-overwrite semantics for unmarked files, and `--force-protocol-overwrite`.

### Impact:
`limux agent-team --dry-run` and live generation preserve existing repo `AGENTS.md` files, refuse unmarked `LIMUX_AGENTS.md` sidecars unless forced, refuse symlink protocol paths, and give agents direct pointers to authoritative instruction files. Verification passed for `cargo test -p limux-cli agent_team`, `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check`. The full `./scripts/check.sh` gate remains blocked until `ghostty/zig-out/lib/libghostty.so` is present. Claude plugin review timed out after 120 seconds and is not counted as passed.

### Related:
`rust/limux-cli/src/main.rs` | `README.md` | `docs/cmux-parity-plan.md` | `HANDOFF.md`

## 2026-05-29 - GTK Surface Send Text Readiness
### What:
Updated the live GTK bridge `surface.send_text` path so `TerminalHandle::send_text == false` returns a conflict error instead of a successful payload with `ok: true`.

### Why:
Automatic agent bootstrap depends on reliable send failure semantics. A resolved terminal surface that is not yet writable must not look successful to `limux-cli`, `agent-team`, or future bootstrap/adapters.

### How:
Added a small `surface_send_text_response` helper in `rust/limux-host-linux/src/window.rs`, wired `ControlCommand::SendText` through it, and added focused unit tests for writable and not-ready responses.

### Impact:
The GTK bridge now preserves the distinction between “surface found” and “surface writable.” `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. Host-crate test execution is blocked in this environment because `pkg-config` is missing, causing GTK sys crates to fail before Rust test compilation.

### Related:
`rust/limux-host-linux/src/window.rs` | `rust/limux-host-linux/src/terminal.rs` | `docs/cmux-parity-plan.md` | `HANDOFF.md`

## 2026-05-29 - Install Posture Decision Packet
### What:
Created a dark-mode decision packet for whether to allow a fuller normal install posture after Phase 5A and GTK send-text hardening.

### Why:
The operator is close to allowing a regular install, but the safer next step is a bounded host prerequisite install/build lane rather than a full system Limux install. The decision packet makes the tradeoff explicit and gives a paste-back response.

### How:
Used the `$html-decision-packet` pattern. The packet recommends installing only the host build/test prerequisites after a mutation-script review gate, then running host tests, `scripts/check.sh`, and the Xvfb smoke path before automatic bootstrap work.

### Impact:
The next session can open `docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html`, copy back the selected install posture, and proceed without reconstructing the package/security discussion.

### Related:
`docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html` | `docs/install-security-report-2026-05-29.md` | `d60d2a3` | `edd781e`

## 2026-05-29 - Host Prerequisite Mutation Review
### What:
Prepared a draft-only mutation review for the bounded host prerequisite install/build lane.

### Why:
The operator selected a minimal host build/test prerequisite lane, with `$mutation-script-wave` before any `sudo apt install`, rather than a full Limux system install.

### How:
Ran read-only recon for OS, installed tools, package status, apt candidates, dependency simulation, Ghostty submodule/lib state, README prerequisites, `scripts/check.sh`, and the Xvfb smoke harness. Wrote the exact draft command block and review synthesis to `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md`.

### Impact:
Mutation wave decision is `WAIT`: the apt prerequisite lane is bounded and reviewable, but it still needs explicit human approval before execution. Zig acquisition and Ghostty build remain a separate follow-up gate.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | SHA256 `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec`

## 2026-05-29 - Host Prerequisite Execution Gate Stop
### What:
Attempted to execute the approved bounded host prerequisite command block from `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md`.

### Why:
The operator explicitly approved the exact apt prerequisite command block with SHA256 `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec` so the previously blocked host GTK test could move past the missing `pkg-config` prerequisite.

### How:
Recomputed the SHA256 and confirmed it matched. Ran the frozen block until the first privileged mutation command. The pre-mutation evidence and apt simulation ran; the transaction remained the reviewed `2 upgraded, 160 newly installed, 0 to remove and 94 not upgraded` lane. Execution then stopped at `sudo apt-get update` because sudo required a password. The run was cancelled instead of collecting or handling a password in chat.

### Impact:
No apt package install occurred. `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and `libwebkitgtk-6.0-dev` remain absent. The next continuation point is to make sudo credentials available outside chat, re-verify the same review artifact SHA, and rerun the approved prerequisite block. Zig acquisition, Ghostty submodule initialization, and Ghostty build remain separate gates.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`

## 2026-05-29 - Sudo Cache Did Not Carry Into Codex PTY
### What:
Retried the approved prerequisite lane after the operator ran `sudo -v` locally, but did not execute the apt install.

### Why:
Before rerunning the frozen apt block, the session checked whether cached sudo credentials were visible inside Codex with `sudo -n true`.

### How:
Verified the mutation review artifact SHA still matched `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec` and confirmed the repo was clean. `sudo -n true` returned `sudo: a password is required`, so the Codex execution context still cannot run privileged commands without prompting for a password.

### Impact:
No OS package mutation occurred. The apt prerequisite lane remains blocked from inside Codex unless sudo credentials are made available to the same execution context, or the operator runs the approved command block manually in their own terminal. The approved review artifact was not edited, preserving its SHA.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`

## 2026-05-29 - Host Prerequisites Installed, Ghostty Gate Reached
### What:
The operator manually ran the approved host prerequisite apt lane in a trusted terminal. `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and `libwebkitgtk-6.0-dev` are now installed.

### Why:
The previous Codex execution context could not access cached sudo credentials, so the operator completed the bounded apt prerequisite install manually while preserving the reviewed package scope.

### How:
Verified post-install state with `dpkg-query` and `pkg-config --modversion gtk4 libadwaita-1 webkitgtk-6.0`. Versions are `pkg-config 1.8.1-2build1`, `pkgconf 1.8.1-2build1`, GTK `4.14.5`, libadwaita `1.5.0`, and WebKitGTK `2.52.3`.

### Impact:
The host test moved past the prior GTK/pkg-config sys-crate blocker and now fails at the expected next gate: `limux-ghostty-sys` cannot find `ghostty/zig-out/lib/libghostty.so`. The `ghostty/` submodule is still uninitialized and `zig` is still not on `PATH`. Zig acquisition, Ghostty submodule initialization, and Ghostty build remain a separate reviewed gate.

### Related:
`rust/limux-ghostty-sys/build.rs` | `HANDOFF.md`

## 2026-05-29 - Ghostty/Zig Mutation Review Prepared
### What:
Prepared a draft-only mutation review for the next Ghostty/Zig build gate.

### Why:
The apt prerequisite lane is complete, and the active host-test blocker is now missing `ghostty/zig-out/lib/libghostty.so`. Resolving that requires fetching the pinned Ghostty submodule and acquiring/building with Zig, which is a separate external-code and native-build supply-chain lane.

### How:
Reviewed README build instructions, `.gitmodules`, current submodule state, `scripts/package.sh`, the pinned Ghostty `build.zig.zon`, official Zig release metadata, and local package/tool availability. Wrote an exact draft command block that uses a project-scoped Zig `0.15.2` tarball from `ziglang.org`, verifies SHA256 `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`, initializes only the pinned `ghostty` submodule, builds `libghostty.so`, and reruns the host readiness test.

### Impact:
Mutation wave decision is `WAIT`: the next lane is bounded and reviewable, but it still needs explicit human approval before execution. No Ghostty/Zig commands were executed.

### Related:
`docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md`

## 2026-05-29 - Ghostty/Zig Security Consensus Gate
### What:
Ran a multi-session security consensus gate on the Ghostty/Zig mutation review using `kazu`, `zori`, `niru`, and the local Claude plugin adversarial review.

### Why:
The next Limux blocker requires downloading Zig, initializing the pinned Ghostty submodule, and building native external code. The operator asked for a consensus security gate before proceeding.

### How:
Sent a durable hcom review brief to the named reviewers, collected v1 `WAIT` findings, patched the mutation review to v2, and ran a narrow v2 re-review. V2 added execution-time Zig metadata cross-checks, fresh per-run extraction, archive containment checks, non-recursive submodule init, explicit `am-will/ghostty` trust-anchor documentation, offline locked Cargo test, and durable evidence logs.

### Impact:
Consensus result is `GO for explicit operator approval; WAIT for execution`. The frozen v2 artifact SHA is `dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`. No Ghostty/Zig command block was executed.

### Related:
`docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md` | `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md` | hcom thread `limux-ghostty-zig-gate`

## 2026-05-29 - Approved Ghostty/Zig Build Gate Executed
### What:
Executed the approved Ghostty/Zig v2 build gate after verifying artifact SHA256 `dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`.

### Why:
The host GTK test was blocked because `limux-ghostty-sys` could not find `ghostty/zig-out/lib/libghostty.so`.

### How:
Verified the frozen review artifact hash, command syntax, repo status, Zig metadata from official `index.json`, Zig archive SHA256 and byte size, archive containment, pinned Ghostty commit `81ab8ffa90185221782baf785e85387321e16f8d`, and absence of nested Ghostty submodules. Built `libghostty.so` with project-scoped Zig `0.15.2`, captured dynamic-link evidence, and ran `CARGO_NET_OFFLINE=true cargo test --locked -p limux-host-linux surface_send_text_response`.

Execution wrapper note: the shell extraction command accidentally captured an earlier illustrative README bash fence before the approved v2 block. That first fence initialized the top-level `ghostty` submodule and attempted `zig build`, which failed immediately because `zig` was not on `PATH`; the approved v2 block then executed successfully. Follow-up inspection found the submodule at the pinned commit, no nested submodules, and no extra system mutation.

### Impact:
`ghostty/zig-out/lib/libghostty.so` now exists locally, and the focused host test passed offline with 2 tests passing. Evidence is stored under `docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/`. The focused host test exposed an existing `unused_mut` warning in `rust/limux-host-linux/src/window.rs`; full clippy/check work should address that before claiming the complete workspace gate.

### Related:
`docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/` | `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`

## 2026-05-29 - Full Gate And Xvfb Smoke Restored
### What:
Cleared the remaining host warning and restored the Xvfb live smoke harness after the Ghostty/Zig gate.

### Why:
The approved Ghostty build made host verification possible again, but the focused host test still emitted an `unused_mut` warning and the smoke harness still carried old Mesa assumptions that prevented Ghostty surfaces from realizing under Xvfb.

### How:
Removed the unnecessary `mut` binding in `rust/limux-host-linux/src/window.rs`. Debugged the smoke failure with `GHOSTTY_LOG=stderr`, which showed `error.OpenGLOutdated`: the script forced `softpipe` plus OpenGL `3.3`, while the pinned Ghostty requires OpenGL `4.3`. Updated the smoke harness to use `llvmpipe` and OpenGL `4.3` by default, with `LIMUX_SMOKE_GALLIUM_DRIVER` available for local Mesa debugging. Also updated stage 6 to accept the current `new-pane --json` ref-shaped response and compare it with raw `LIMUX_*` child env values.

### Impact:
`cargo fmt --check`, `git diff --check`, `./scripts/check.sh`, and `./scripts/xvfb-smoke-test.sh` pass with the local Ghostty library on `LD_LIBRARY_PATH`. The live smoke now verifies `agent-team --dry-run`, live `agent-team --no-launch`, workspace listing, peer surface send, workspace notify, self-split `new-pane` command execution with fresh `LIMUX_*` env, and hook translation.

### Related:
`scripts/xvfb-smoke-test.sh` | `rust/limux-host-linux/src/window.rs` | `HANDOFF.md`

## 2026-05-29 - Shell-Quoted Launch Snippet Hardening
### What:
Hardened generated `limux new-pane --command ...` shell snippets and removed unsafe nested-prompt examples from workflow docs.

### Why:
Automatic agent bootstrap must not be built on ad hoc shell strings. Generated snippets need to preserve launch commands as one caller-shell argv, avoid command-substitution/semicolon side effects, and make arbitrary prompt text a post-readiness `limux send` concern instead of a launch-shell concern.

### How:
Added central `shell_command_arg` / `new_pane_shell_command` helpers, changed generated `LIMUX_AGENTS.md` scratch-pane output to quote `bash`, and made `new-pane` fail fast on unexpected positional tokens such as unquoted extra prompt text. Added regression tests for metacharacter round trips, exact JSON command preservation, leading-hyphen command values, single-argv parsing, and outer-shell side-effect inertness. Updated README, cmux parity, hcom workflow, and Limux-vs-Multica decision docs.

### Impact:
The current manual/generated-snippet path is green. Full automatic bootstrap remains deferred until typed-PTY paths such as `limux send` / respawn / host-spawn have an explicit control-character and newline policy plus live metacharacter smoke coverage.

### Verification:
`cargo test -p limux-cli agent_team_tests::`, `cargo test -p limux-cli new_pane_tests::`, `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, `git diff --check`, `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh`, and `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/xvfb-smoke-test.sh` passed. hcom reviewers `niru`, `zori`, and `kazu` converged on GO for the manual snippet path and deferred typed-PTY control-character handling before auto-bootstrap. Claude plugin adversarial review timed out after 180 seconds; a `--bare` retry failed because Claude was not logged in under bare mode, so it is not counted as a passed plugin review.

### Related:
`rust/limux-cli/src/main.rs` | `docs/cmux-parity-plan.md` | `docs/limux-hcom-workflow.md` | hcom thread `limux-shell-quoting`

## 2026-05-29 - Typed-PTY Control-Character Guard
### What:
Added a shared typed-terminal-text safety policy for Limux control paths that inject text into terminal panes.

### Why:
Automatic agent bootstrap should send arbitrary prompt text only after pane readiness, and that typed-text route needs a clear boundary between printable/multiline messages and terminal control sequences. ESC, BEL, C1 CSI/OSC, NUL, DEL, and similar controls should not be injectable through `limux send`, paste, respawn, or host-spawn text paths.

### How:
Added `validate_terminal_text_payload` in `limux-protocol`, allowing printable Unicode plus tab, LF, and CR while rejecting other `char::is_control()` values with byte offset and codepoint diagnostics. Enforced it in `limux-cli`, `limux-core`, `limux-host-linux` control parsing, and the GTK host send sink before `TerminalHandle::send_text`. Kept `limux send-key` / `surface.send_key` as the explicit route for control keys, and left OSC/output parsing separate from typed input. Expanded the Xvfb smoke harness to reject ESC/BEL/C1 payloads across `send`, `new-pane --command`, `respawn-pane`, `paste-buffer`, and `new-workspace --command`.

### Impact:
The typed-PTY safety gate is complete for the current control surface. Full automatic bootstrap can now proceed as a two-phase implementation: launch the agent binary first, wait for surface readiness, then send prompt text through the guarded `limux send` path. Residual policy choices are explicit: CR remains allowed, and Unicode format / zero-width characters are not blocked by this guard. Kazu's closeout classified those as accepted/deferred display-spoofing risks, not execution risks, and asked that they be tracked in the Phase 5B automatic-bootstrap threat model.

### Verification:
Red tests were observed before implementation for protocol, CLI, core, and host bridge behavior. After implementation, `cargo test -p limux-protocol terminal_text_policy`, `cargo test -p limux-cli terminal_control`, `cargo test -p limux-core terminal_control`, `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" cargo test -p limux-host-linux terminal_control`, `cargo fmt --check`, `git diff --check`, `bash -n scripts/xvfb-smoke-test.sh`, `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" cargo test -p limux-cli`, `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh`, and `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/xvfb-smoke-test.sh` passed. Claude plugin adversarial review timed out after 240 seconds and is not counted as a passed plugin review; hcom reviewers `niru`, `zori`, and `kazu` had already converged on the shared-validator / CLI+host+core enforcement shape.

### Related:
`rust/limux-protocol/src/lib.rs` | `rust/limux-cli/src/main.rs` | `rust/limux-core/src/lib.rs` | `rust/limux-host-linux/src/control_bridge.rs` | `rust/limux-host-linux/src/window.rs` | `scripts/xvfb-smoke-test.sh` | `HANDOFF.md` | hcom thread `limux-typed-pty-policy`

## 2026-05-29 - Phase 5B Agent-Team Automatic Bootstrap
### What:
Implemented Phase 5B for `limux agent-team`: live runs now launch peer panes with bare agent commands, write `LIMUX_AGENTS.md` first, then send each peer a short bootstrap prompt that points to the generated protocol and authoritative instruction sources.

### Why:
The operator workflow needs near-zero-friction Codex/Claude team startup without putting arbitrary prompt text inside launch-shell command strings or silently copying repo instructions.

### How:
Added `--no-bootstrap`, top-level/per-peer bootstrap status reporting, strict generated-prompt validation, post-write `surface.send_text` delivery, explicit `surface.send_key enter` submission, and failure reporting that names the peer and surface. Fixed host command-launch Enter semantics for Ghostty by sending text and Enter separately, and widened the command-launch readiness budget for slower hosts. Expanded CLI tests and the Xvfb smoke harness with fake `codex`/`claude` binaries that prove the prompt was received after the protocol file exists.

### Impact:
`agent-team` can now start a paired local agent team and orient peers automatically while preserving `--dry-run`, `--no-launch`, and `--no-bootstrap` safety paths. The next Limux workflow work is project/team roster plus durable review and consensus ledger support.

### Verification:
`cargo fmt --check`, `bash -n scripts/xvfb-smoke-test.sh`, `git diff --check`, `cargo test -p limux-cli agent_team`, `cargo test -p limux-cli`, `cargo test -p limux-host-linux fallback_enter_key_values_match_ghostty_key_encoding`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, `LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh`, `LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh`, and `./scripts/xvfb-smoke-test.sh` passed after the final Claude review fixes. Pre-exec hcom reviewers `kazu` and `zori` returned GO with blockers that were implemented; `niru` acked the gate and no late blocking verdict was observed before closeout. Claude plugin adversarial review found no security-blocking defect and flagged medium reliability issues; follow-up removed trailing-LF double submission, made fail-fast partial-side-effect behavior explicit in the error path, and widened the command-launch readiness budget. Residual: the smoke proves fake-agent ordering, not real Codex/Claude TUI readiness under slow cold starts.

### Related:
`rust/limux-cli/src/main.rs` | `rust/limux-host-linux/src/window.rs` | `rust/limux-host-linux/src/terminal.rs` | `scripts/xvfb-smoke-test.sh` | `docs/cmux-parity-plan.md` | `docs/limux-hcom-workflow.md` | `HANDOFF.md` | hcom thread `limux-phase5b-bootstrap`

## 2026-05-29 - Phase 5C Agent-Team Durable Roster And Review Ledger
### What:
Implemented Phase 5C for `limux agent-team`: runs now seed `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` when missing and point generated protocol/bootstrap instructions at both durable coordination files.

### Why:
The operator workflow spans multiple projects and agent teams. Team ownership, related teams, reviewer findings, consensus decisions, accepted risks, and cross-team notifications need durable files instead of terminal scrollback.

### How:
Added `--roster-path`, `--ledger-path`, and `--force-roster-overwrite`; created a durable Markdown roster template and append-oriented review ledger template; preserved existing roster/ledger files by default; kept live surface/pane/workspace IDs in the regenerated `LIMUX_AGENTS.md` protocol instead of the durable roster; refused unmarked force replacement, symlink, non-regular, and overlapping roster/ledger/protocol targets; updated generated `LIMUX_AGENTS.md`, bootstrap prompts, README, roadmap, workflow, decision, and handoff docs. Expanded CLI tests and Xvfb fake-agent smoke proof so peers see protocol, roster, and ledger files before bootstrap.

### Impact:
`agent-team` now gives new Codex/Claude/Gemini/OpenCode panes a low-friction, file-backed place to find project/team routing and record review consensus. The next practical lane is a reviewer/capture wrapper plus consensus/cross-team broadcast conventions.

### Related:
`rust/limux-cli/src/main.rs` | `scripts/xvfb-smoke-test.sh` | `docs/cmux-parity-plan.md` | `docs/limux-hcom-workflow.md` | `docs/limux-vs-multica-decision-guide.md` | `HANDOFF.md`

## 2026-05-29 - Expanded Phase 5C Next Steps Packet
### What:
Added a clearer Markdown and dark-mode HTML decision packet for the post-Phase-5C next steps.

### Why:
The one-line recommendation, "reviewer/capture wrapper plus consensus conventions," was too compressed. The operator asked for revised next steps with more detail.

### How:
Created `docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.md` and `.html`. The packet recommends Phase 5D1, a reviewer workflow scaffold that creates review request files, appends pending ledger entries, and prints reviewer prompts before attempting full reviewer pane spawn/capture automation. Updated `HANDOFF.md` and the Limux+hcom workflow guide to point to the richer plan.

### Impact:
The next session can choose between Phase 5D1 scaffold, full spawn/capture wrapper, real-agent readiness smoke, or consensus convention docs with clear tradeoffs and a copy-back payload.

### Related:
`docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.md` | `docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.html` | `HANDOFF.md` | `docs/limux-hcom-workflow.md`

## 2026-05-30 - Phase 5D1 Reviewer Workflow Scaffold
### What:
Implemented `limux review prepare` as the first reviewer workflow scaffold on
top of the Phase 5C review ledger.

### Why:
The operator selected Option A from the Phase 5C next-steps packet. The safest
next step was to make reviews durable and repeatable before automating real
reviewer pane launch, prompt delivery, output capture, or consensus finalization.

### How:
Added `review prepare` with required `--artifact`, `--reviewer`, `--lens`, and
`--summary` fields; optional `--cwd`, `--ledger-path`, `--reviews-dir`,
`--review-id`, and `--dry-run`; atomic `reviews/<review-id>.md` creation;
append-only pending entries in `LIMUX_REVIEW_LEDGER.md`; reviewer/lens
allowlists; and refusal for existing request files, leaf symlink review
directories, leaf symlink/non-regular ledgers, overlapping request/ledger paths,
and control characters in generated prompt fields. Documented that output
directories must be trusted because parent path components are not recursively
audited for symlinks. Updated README, roadmap, workflow Markdown/HTML, decision
packet, handoff, and this journal.

### Impact:
Limux can now prepare a file-backed review without contacting a running host or
launching an agent. The next practical lane is Phase 5D2: start a reviewer pane,
send the prepared prompt after readiness, capture or point to reviewer evidence,
and update the pending ledger entry without rewriting unrelated content.

### Verification:
Observed RED compile failure before implementation because `run_review_prepare`
did not exist. After implementation, `cargo test -p limux-cli review_prepare`,
`cargo test -p limux-cli review`, `cargo test -p limux-cli agent_team`,
`cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli
--all-targets -- -D warnings`, `git diff --check`,
`LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./scripts/check.sh`, and `./scripts/xvfb-smoke-test.sh` passed. Claude plugin
adversarial review found no high blockers; follow-up tightened symlink wording
and expanded refusal-branch tests before commit.

### Related:
`rust/limux-cli/src/main.rs` | `README.md` | `docs/cmux-parity-plan.md` |
`docs/limux-hcom-workflow.md` | `docs/limux-hcom-workflow.html` |
`HANDOFF.md`

## 2026-05-30 - End-Of-Night Limux Closeout
### What:
Closed the session after Phase 5D1 by confirming the implementation commit was
pushed and updating `HANDOFF.md` with the exact resume lane.

### Why:
The operator is stopping for the night and needs a zero-context successor to
resume without reconstructing the previous discussion or redoing completed work.

### How:
Recorded the pushed commit `e4ce6fd feat(cli): add review prepare scaffold`,
clean working-tree status at closeout, and the next scoped recommendation:
Phase 5D2 reviewer spawn/capture wrapper.

### Impact:
The next session should start from `HANDOFF.md`, treat Phase 5D1 as complete,
and proceed only with the Phase 5D2 wrapper unless a regression is discovered.

### Related:
`HANDOFF.md` | `e4ce6fd`

## 2026-06-05 - Phase 5D2 Reviewer Spawn Evidence Pointer
### What:
Implemented `limux review spawn` as the Phase 5D2 continuation of
`limux review prepare`.

### Why:
Phase 5D1 made review requests durable but deliberately stopped before real
reviewer pane launch. The next useful automation step was to start one reviewer
from an existing generated request, deliver the prepared prompt after pane
creation, and leave durable evidence/ledger pointers without storing raw
terminal transcripts.

### How:
Added `review spawn --review-id <id>` with optional `--cwd`, `--reviews-dir`,
`--ledger-path`, `--evidence-path`, `--workspace`, `--surface`, `--direction`,
`--no-launch`, and `--dry-run`. The command reads the generated request file,
refuses `manual` reviewers, creates a reviewer terminal pane through
`pane.create`, sends the request prompt through `surface.send_text` plus
explicit Enter, writes `reviews/<review-id>.evidence.md` with the reviewer
surface and capture command, and updates only the matching pending ledger block
to `in-progress`. Updated README, `docs/cmux-parity-plan.md`,
`docs/limux-hcom-workflow.md`, and `HANDOFF.md`.

### Impact:
Limux can now move a prepared review into a live reviewer pane while preserving
the file-first request/ledger model. Remaining Phase 5D work is a
collect/complete path that records reviewer verdicts and consensus back into
the existing ledger entry without unrelated rewrites.

### Verification:
Observed RED compile failure before implementation because `run_review_command`
was still prepare-only and synchronous. After implementation, `cargo test -p
limux-cli review_spawn -- --nocapture`, `cargo test -p limux-cli review --
--nocapture`, `cargo test -p limux-cli`, `cargo clippy -p limux-cli
--all-targets -- -D warnings`, `cargo fmt --check`, `git diff --check`,
`LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./scripts/check.sh`, and `LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh`
passed.

### Related:
`rust/limux-cli/src/main.rs` | `README.md` | `docs/cmux-parity-plan.md` |
`docs/limux-hcom-workflow.md` | `HANDOFF.md`

## 2026-06-06 - Restart-Safe Docs Closeout
### What:
Refreshed the Limux handoff for a PC restart after Phase 5D2 was already
verified and closed.

### Why:
The operator reported severe RAM pressure and asked active sessions to update
docs before restarting the machine.

### How:
Ran a local doc freshness pass, confirmed only `HANDOFF.md` is the canonical
handoff surface in this repo, confirmed `main` was aligned with `origin/main`
before edits, and added a restart closeout marker to `HANDOFF.md`.

### Impact:
A post-reboot successor can resume from `HANDOFF.md` without reconstructing the
Phase 5D2 closeout. No new Limux scope was started; the next lane remains Phase
5D3 review collect/complete plus consensus conventions.

### Related:
`HANDOFF.md` | `1f47aa1`

## 2026-06-10 - Local Limux Launcher Unblock
### What:
Added a repo-local `scripts/limux-dev` launcher and installed user-local
`limux` / `limux-cli` symlinks in `/home/riche/.local/bin`.

### Why:
The operator resumed the Limux lane to start using the app immediately. The repo
already built and smoked successfully, but no public `limux` command was on
`PATH`.

### How:
Verified the current tree before setup with the full workspace check plus debug
and release Xvfb smoke tests. Then added a strict Bash launcher that executes
the release CLI entrypoint, points it at the sibling host binary, and prepends
the checkout's `ghostty/zig-out/lib` path for `libghostty.so`. Avoided sudo,
`scripts/package.sh`, package installs, generated install scripts, and system
package mutation for this immediate unblock.

### Impact:
`limux`, `limux-cli`, and CLI workflows such as `limux agent-team --dry-run`
now resolve from the shell. The next feature lane remains Phase 5D3 after the
operator has had a chance to use Limux.

### Related:
`scripts/limux-dev` | `README.md` | `HANDOFF.md`

## 2026-06-10 - Hcom Launch Mode And Verification Isolation
### What:
Added `--launch-mode hcom` for `limux agent-team` and `limux review spawn`, and
fixed verification scripts so they do not inherit a live Limux pane's `LIMUX_*`
environment while running isolated checks.

### Why:
The operator normally starts agents through hcom (`hcom codex`, `hcom claude`)
and needs those sessions to stay inside Limux panes rather than opening
separate external terminals. The first verification run from inside Limux also
showed the test and smoke scripts could accidentally target the user's real
pane/workspace.

### How:
Used TDD around `agent-team` and `review spawn`, added explicit launch-mode
parsing with `direct` as the default and `hcom` mapping to
`hcom <agent> --run-here`, updated README/workflow/parity docs, and added a
smoke dry-run check for hcom launch-mode protocol generation. The scripts now
clear inherited live Limux pane/socket env before isolated test runs.

### Impact:
Users can start a Limux team with
`limux agent-team --agents codex,claude --launch-mode hcom --cwd "$PWD"`.
The local launcher and verification path are more reliable when operated from
inside Limux itself. Full/system install remains a separate gated supply-chain
lane.

### Related:
`678de2c` | `a0f4e34` | `rust/limux-cli/src/main.rs` | `scripts/xvfb-smoke-test.sh` | `HANDOFF.md`

## 2026-06-10 - Project Isolation Lab Goal Alignment
### What:
Recorded the operator-provided active goal that this lane is focused on a
reusable Project Isolation Lab, not Limux feature development.

### Why:
The prior Limux handoff still made Phase 5D3/internal Limux feature work look
like the default next lane. The operator clarified that the real goal is the
full-isolation Linux lab: persistent full VM(s), disposable full VM(s), then
Firecracker microVMs, with disposable WSL only as an ergonomics companion lane.

### How:
Updated `HANDOFF.md` with the official active goal and Limux boundary, added
`docs/project-isolation-lab-goal.md` as a Limux-local pointer to the
SCS-owned canonical lab docs, and left SCS-side canonical updates to gumo via
hcom routing rather than patching another repo's owner lane.

### Impact:
Future Limux sessions should keep the current Limux launcher/hcom setup usable
but avoid drifting into internal Limux product work unless the operator
explicitly redirects. The cross-project isolation lab is the active priority.

### Related:
`HANDOFF.md` | `docs/project-isolation-lab-goal.md`
