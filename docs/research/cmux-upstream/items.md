# Candidate Items

Snapshot date: 2026-07-02

| ID | Theme | Kind | Priority | Source | Fit | Value | Risk | Complexity | Limux Translation | Next Action |
|---|---|---|---|---|---|---:|---:|---:|---|---|
| cmux-20260702-001 | browser | feature/security | high | cmux issue #7178 | translate | 5 | 4 | 4 | Add browser command bridge parity with server-side domain allowlist and audit events for WebKitGTK. | Write PRD. |
| cmux-20260702-002 | notifications | ux | high | cmux PR #6480 / PR #7005 | translate | 5 | 3 | 3 | Add scalable agent lifecycle state to workspace/sidebar rows: running, needs input, idle, unknown. | Merge into agent sidebar PRD. |
| cmux-20260702-003 | workspace | feature | medium | cmux v0.64.11 | inspiration-only | 4 | 3 | 4 | Design workspace groups for many concurrent projects after sidebar basics stabilize. | Product PRD later. |
| cmux-20260702-004 | workspace | ux | medium | cmux v0.64.11 | translate | 4 | 3 | 3 | Add focus history and recently closed surfaces/workspaces. | Include in restore/session correctness PRD. |
| cmux-20260702-005 | performance | architecture | medium | cmux v0.64.11 | inspiration-only | 4 | 5 | 5 | Explore Linux-safe hibernation or throttling for idle agent panes without corrupting PTYs. | Research spike only. |
| cmux-20260702-006 | rendering | performance/fix | high | cmux issue #7186 | translate | 5 | 4 | 4 | Test and throttle hidden/occluded Ghostty surfaces that stream output. | Create performance benchmark task. |
| cmux-20260702-007 | workspace | performance/fix | high | cmux PR #6807 / PR #7182 | translate | 5 | 3 | 3 | Avoid sidebar/list model rebuilds for every title/output update with many agent panes. | Include in sidebar PRD. |
| cmux-20260702-008 | terminal | fix | high | cmux PR #7165 / issue #7155 | direct | 5 | 3 | 3 | Preserve cwd when splitting or opening tabs from panes hosting resumed agents. | Write focused fix task. |
| cmux-20260702-009 | terminal | fix | high | cmux PR #6892 | direct | 5 | 2 | 3 | Persist split order, ratios, and pane identity in autosave/restore. | Include in restore/session PRD. |
| cmux-20260702-010 | security | fix | medium | cmux PR #7173 / PR #7179 | direct | 4 | 2 | 2 | Ensure any Git observation uses `GIT_OPTIONAL_LOCKS=0` and never creates `.git/index.lock`. | Add to coding-agent/file sidebar rules. |
| cmux-20260702-011 | notifications | feature | medium | cmux PR #6983 | translate | 4 | 4 | 4 | Show PR/CI state in workspace sidebar only with explicit GitHub auth/rate-limit design. | Future PRD. |
| cmux-20260702-012 | settings | ux | medium | cmux PR #6906 | translate | 4 | 3 | 4 | Add GTK-native custom accelerator settings for commands. | Settings roadmap. |
| cmux-20260702-013 | workspace | ux | high | cmux PR #6994 / PR #6981 / upstream Limux PR #92 | translate | 5 | 2 | 3 | Add deterministic workspace/pane color flags, preserving unread state semantics. | Align with TaskMaster #20. |
| cmux-20260702-014 | terminal | architecture | medium | cmux v0.64.17 / PR #7020 / PR #7023 | inspiration-only | 4 | 5 | 5 | Consider Linux-native SSH/tmux remote session persistence after local runtime is stable. | Research later. |
| limux-upstream-20260702-001 | rendering | fix | high | upstream PR #83 / #100 / issue #82 | translate | 5 | 4 | 4 | Port physical-pixel sizing/fractional-scale fixes manually against current fork render code. | Focused render-sizing spike. |
| limux-upstream-20260702-002 | terminal | fix | high | upstream PR #90 / issue #89 | translate | 5 | 4 | 4 | Harvest IME/dead-key tests and Wayland IMContext behavior without blind merge. | Focused input PRD/task. |
| limux-upstream-20260702-003 | terminal | ux/fix | high | upstream PR #101 / issue #93 | direct | 4 | 3 | 3 | Make Ctrl+W close the active tab rather than the pane/workspace where appropriate. | Shortcut contract review. |
| limux-upstream-20260702-004 | terminal | fix | high | upstream PR #108 | translate | 4 | 4 | 3 | Inspect split SIGABRT and send-key Return changes; cherry-pick only if still relevant. | Patch review task. |
| limux-upstream-20260702-005 | runtime-isolation | test | high | upstream issue #106 | direct | 5 | 2 | 2 | Use "running Limux twice breaks first socket" as a regression case for stable/preview isolation. | Add to runtime smoke suite. |
| limux-upstream-20260702-006 | browser | security/fix | medium | upstream PR #86 / issue #85 | translate | 4 | 4 | 3 | Port safe Ctrl-click URL activation/preview behavior only after browser bridge boundary is designed. | Link to browser PRD. |
| limux-upstream-20260702-007 | settings | ux | medium | upstream PR #98 / issue #105 | translate | 4 | 3 | 4 | Add per-terminal font size/scaling in GTK-native settings model. | Settings roadmap. |
| limux-upstream-20260702-008 | workspace | ux | medium | upstream PR #103 / #62 | inspiration-only | 4 | 5 | 5 | Mine UI/settings concepts; do not merge large `window.rs` redesign. | Design-only review. |
| limux-upstream-20260702-009 | terminal | ux | low | upstream PR #94 / issue #58 | translate | 3 | 3 | 3 | Consider overlay scrollbar after render/input stabilization. | Defer. |
| limux-upstream-20260702-010 | packaging | packaging | low | upstream PR #88 / #81 / issues #75 #40 #80 #87 | inspiration-only | 3 | 5 | 4 | Track for release packaging only; do not import scripts without package-security review. | Defer to release lane. |

## First PRDs

1. Browser bridge parity plus domain allowlist.
2. Scalable agent sidebar state and notification correctness.
3. Restore/session correctness pack.
4. Render sizing/fractional-scale correctness.
5. IME/dead-key input correctness.
