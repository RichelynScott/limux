# H1 — Workspace-Entitlement Design Note (read-screen cross-workspace disclosure)

**Created by:** Claude Code (tutu / LIMUX_MGR · cd1a39d7)
**Date:** 2026-07-29 (EST)
**Purpose:** Design note (NOT a patch) for REPO_AUDIT H1 — the same-user cross-workspace surface disclosure. Traces the mechanism, inventories the blast radius of any server-side fix, and lays out options with tradeoffs. The code change is a separate operator-gated decision *after* the blast radius is visible; do not implement speculatively.

**Status:** DESIGN NOTE. No code change. No live probe was run (the probe IS the incident). REPO_AUDIT H1 (`docs/REPO_AUDIT_limux_2026-07-21.md:57`) is confirmed live on current code by static trace. Empirical precedent: reve incident 2026-07-19 (on the legacy `main-1005f58d` build).

---

## 1. Traced mechanism (source, current code)

**The auth layer is uid-level only — no workspace dimension.**
`limux-control/src/auth.rs` `is_authorized` (L66-72): three match arms —
`AllowAll => true`; `LimuxOnly => peer.uid == current_uid() && is_descendant(peer.pid)`;
`LocalUser => peer.uid == current_uid()`. Credentials come from `SO_PEERCRED`
(`peer_info`, getsockopt). `authorize_peer` is called **once at connection accept**
(`limux-control/src/server.rs:51`, `limux-host-linux/src/control_bridge.rs:1610`),
**not** per-request and **not** per-workspace. **limux's threat model is same-user
multi-agent** (different agents / lanes run as the same uid), so uid-auth does not
separate agents from each other.

**Surface resolution has NO ownership check, and the two control paths diverge**
(as CLAUDE.md notes: the live GTK bridge is "only partially equivalent" to the standalone dispatcher):

- **Standalone dispatcher** — `limux-core::resolve_surface_target` (`rust/limux-core/src/lib.rs:3704`):
  - `--surface` **alone** → `find_workspace_for_surface` (L3673) = **global scan across ALL workspaces** (`state.workspaces.iter().find_map(...)`), returns the surface from any lane. Reads any surface by id, no workspace needed.
  - `--surface` **+** `--workspace` → `workspace_contains_surface` — the surface must be in the named workspace, else `not_found`. But **any** workspace is nameable.
- **Live GTK bridge** — `limux-host-linux/src/window.rs` `ReadSurfaceText` handler (L6450):
  - workspace resolved from `target` via `workspace_index_for_target` (L1077): `Active` = the focused workspace; `Handle(id)` = any workspace by id — **no ownership check**.
  - surface then resolved **within that workspace** via `terminal_handle_for_root(&workspace.root, surface_hint)` — no global scan.
  - So `--surface` + `--workspace <foreign>` reads the named foreign workspace's surface; `--surface` **alone** (no `--workspace`) → `Active`/focused workspace = the reve 2026-07-19 fallback vector.

**`dfb5d40` was a client-side mitigation only.** It made the CLI **default** `--workspace`
to `LIMUX_WORKSPACE_ID` when inside a limux pane (`build_read_screen_params_with_env`,
`rust/limux-cli/src/main.rs`), so an in-pane CLI call sends its own workspace. It added
**zero** server-side enforcement. The `read-screen --help` text and `main.rs` comments
("cross-lane read", "another lane's pane", "reve incident 2026-07-19", "cross-lane
misattribution") show the authors already know the fallback crosses lanes.

**Net:** any same-user process that passes the uid gate can read any workspace's surface
in that server — by surface-id alone (standalone path) or by explicit `--workspace`
targeting (live path). This is REPO_AUDIT H1's "SAME-USER CROSS-LANE INFORMATION DISCLOSURE".

---

## 2. Blast-radius inventory (the part that decides shippability)

Every current caller that touches a surface across a workspace boundary. Static grep of the in-tree CLI + host; external scripts are not enumerable from the repo (see §4).

| Caller | Cross-workspace? | Detail |
|---|---|---|
| **Surface-targeting CLI commands** (`read-screen`/`capture-pane`, `send`, `send-key`, `close-surface`, `new-pane`, `tab-action`, `pane-action`, `identify`, `browser`) | **No, in the common case** | All default `--workspace` to `LIMUX_WORKSPACE_ID` when unset (`main.rs` L1109/1229/1253/1278/1460/3037) → own workspace inside a pane. They become cross-workspace only when a caller *explicitly* passes a foreign `--workspace`/`--surface`, or runs with `LIMUX_WORKSPACE_ID` unset (outside a pane → server fallback). |
| **`agent-team`** | **No** | Splits ONE explicitly-targeted workspace into one pane per agent (peers are panes of the same workspace, `main.rs:326`/2754). "live runs require `--workspace`/`--surface` or `LIMUX_WORKSPACE_ID`/`LIMUX_SURFACE_ID`; the focused workspace is **never** used as an implicit target." Own-workspace scoped. |
| **Peer messaging** (`limux send --surface <peer-surface-id>`) | **No** — same-workspace cross-**surface** | Peers message each other across panes **within one workspace** (agent-team). The `workspace_contains_surface` branch already allows this. Note L2466 guidance actually *prefers* hcom over `limux send` for peer/orchestrator messages. |
| **Agent hooks** (`claude-hook`/`opencode-hook`/`gemini-hook`, the JS plugin) | **No** | Operate on the caller's own surface via `LIMUX_SURFACE_ID` (`main.rs:2648-2685`); skip when unset. |
| **`list-workspaces` / `workspace.list`** | **Enumeration, not a read** | Returns workspace metadata + ids (`control_bridge.rs:980`). Does not read surface text — but it is the **discovery primitive** that hands out the workspace/surface ids that make explicit foreign targeting trivial (no id-guessing needed). |
| **The OPERATOR (human) via interactive CLI** | **YES — and legitimate** | The human owns ALL their workspaces and may legitimately `read-screen --workspace <any>`. **Same uid as the agents.** This is the load-bearing tension: a blanket server-side cross-workspace reject would break the operator's own multi-workspace inspection, and the fix **cannot key on uid** to tell operator-from-agent. |
| Multi-workspace read/monitor/broadcast flow | **None found in-tree** | No CLI/host flow iterates workspaces to read or broadcast surface text. |

**Key finding:** the only *legitimate* cross-workspace surface reader in scope is the
**operator's own interactive use** — which shares the uid with every agent. So a
server-side entitlement model must distinguish "this connection is agent X (own its
workspace)" from "this connection is the operator (entitled to all)" **without** a uid
difference. That distinction is the whole design problem; it is why H1 is a
security-boundary redesign, not a one-function patch.

---

## 3. Options with tradeoffs

**(a) Drop the standalone `find_workspace_for_surface` global scan only.** Smallest.
Kills the worst path — reading *any* surface by id alone, no workspace, on the standalone
dispatcher. Blast radius: only affects `--surface`-without-`--workspace` on the standalone
path; the CLI in-pane default already supplies a workspace, and the live GTK path never had
the global scan. **Does NOT fully close H1** — explicit `--workspace <foreign>` + `--surface`
still discloses on both paths. Shippable independently as an immediate partial mitigation.

**(b) Bind connection → entitled workspace(s) at accept; reject others.** The real fix.
Requires a per-connection **workspace claim** at connect (an agent presents its own
`LIMUX_WORKSPACE_ID` as an entitlement; the server records connection→workspace and rejects
reads outside it). Blast radius: every agent connection must present its claim; **the operator's
interactive connection must be a distinguishable "unclaimed = all-entitled" path** (cannot be
uid-based); any legitimate cross-workspace orchestrator (none found in-tree, but external
scripts possible) needs an explicit grant. This is where the design work is.

**(c) Capability/token per workspace.** Biggest. Each workspace mints a capability token; a
connection presents the token for the workspace it accesses. Strongest isolation; largest new
surface (issuance, storage, rotation, revocation); most disruptive to existing callers. Likely
over-engineered for a same-user threat model.

**Framing (not a decision):** (a) is a safe, low-blast-radius immediate mitigation that can
ship on its own; (b) is the real fix but is gated on the operator-vs-agent entitlement design
above; (c) is disproportionate for same-user. The code change is operator-gated and must
follow this blast-radius picture — do not implement (b) speculatively.

---

## 4. What was NOT verified

- **No live probe** — deliberately. Probing an explicit cross-workspace read of a live
  foreign pane *is* the disclosure incident; it was declined and must be a scoped test with a
  consenting target if ever run.
- **No runtime confirmation of the live GTK path.** `ReadSurfaceText`/`workspace_index_for_target`
  were traced from source only; not runtime-observed on the current build.
- **The blast-radius grep is static and scoped** to the in-tree CLI (`limux-cli`) and host
  (`limux-host-linux`/`limux-control`). External operator scripts or third-party callers that
  span workspaces are not enumerable from this repo.
- **Not an exhaustive control-method audit.** I traced the surface-read path specifically
  (`surface.read_text`/`ReadSurfaceText` → `resolve_surface_target`/`workspace_index_for_target`
  → auth). I did not independently audit every control method for its own workspace handling.
- **reve's incident (the empirical proof) was on legacy `main-1005f58d`.** The current-code
  trace shows the mechanism persists; it was not runtime-reproduced on current code.
