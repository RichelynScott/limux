# Hermes GLM Review

Slot: glm-cursor-api-rerun
Model: glm-5.2
Provider: ollama-cloud


session_id: 20260630_172920_3c8167
Verdict: PASS_WITH_CHANGES

Top Findings:

1. [P1] Activation model underspecified. Plan §Architecture lists a side view and commands but never names activation events or contributions. Cursor/VS Code needs `activationEvents` (or `contributes`-driven lazy activation) in package.json, and `onView:limux` to start the tree provider on first reveal, not eagerly. Without this the extension either fails to load or loads on startup unnecessarily. Fix: add an explicit `contributes.views` entry with `views-explorer` or a dedicated container and `activationEvents: ["onView:limux-workspaces"]`, and document that the provider lazily connects the socket on first `getChildren`/refresh rather than in `activate()`.

2. [P1] TreeDataProvider contract not pinned. Plan §Cursor UI says "Tree shape" but does not name `vscode.TreeDataProvider<T>` or the `TreeItem` shape per level. The provider must implement `getChildren(element?)` and `refresh` via `onDidChangeTreeData` emitter; per-item commands attach through `TreeItem.command` or context-menu `contributes.menus`. Multi-level (Workspace→Pane→Surface) requires stable element identity (`id` + context value) so `reveal()` and selection survive refresh. Fix: add a short "Extension API surface" subsection naming `TreeDataProvider`, `TreeItem.collapsibleState`, context-value scoping for per-level context menus, and the refresh emitter.

3. [P1] Socket I/O from extension host unaddressed. Plan §Socket Resolution covers path discovery but not the I/O layer. The extension runs in the Node-side extension host; raw `net.connect` to a Unix socket works, but the plan should state it uses `net.Socket` + a length-framed JSON reader, not `child_process` or fetch. Cursor's extension host is Node, so this is feasible — but the plan never confirms the transport primitive, leaving readers to infer it. Fix: one line: "extension uses Node `net.connect(AF_UNIX)` with the existing length-prefix framing."

4. [P2] `node --test` path is sound but unverified against Cursor's bundled Node. Plan §Tests says `node --test` and no npm/npx/vsce. `node --test` is stable since Node 18; Cursor/VS Code ship Node 18+ in the host. However, tests run in the developer's terminal, not the extension host, so the installed system Node governs — add a minimum-Node line (>=18.17) to avoid `--test` breakage on older dev machines.

5. [P2] `--extensionDevelopmentPath` manual smoke is correct but incomplete. Plan §Tests lists `cursor --extensionDevelopmentPath integrations/cursor-limux`. That launches an Extension Development Host, but a tree view needs the Limux socket reachable inside that host process env. Add: export `LIMUX_SOCKET` before launch, or verify the setting-based path, so the smoke is reproducible.

6. [P2] `Pseudoterminal` mentioned in Source Notes but unused in v1. Line 260 cites `Pseudoterminal` as an available API, yet v1 forbids terminal injection and attach. Remove the reference or clarify it is v2-only to avoid implying v1 uses it.

Missing Evidence:
- No `package.json` `contributes` skeleton; activationEvents, views, commands, menus all unstated.
- No confirmation that Cursor's extension host exposes `net.Socket` to AF_UNIX (it does, same as VS Code, but the plan does not assert it).
- No verification that `node --test` runs against the developer's system Node version in this environment.
- No evidence the `surface.read_text` host method actually exists in `control_bridge.rs`; plan assumes it. (Not in my review lens to confirm, but flagged.)

Recommended Plan Changes:
- Add an "Extension API Surface" subsection: `TreeDataProvider<LimuxNode>`, `TreeItem` with `contextValue` per level, `onDidChangeTreeData` refresh, lazy socket connect on first `getChildren`.
- Add `package.json` contributes skeleton: one view container, one TreeView, commands with `enablement` clauses, context menus scoped by `contextValue`.
- Specify `net.connect` + length-framed JSON as the transport primitive.
- State minimum Node 18.17 for the `node --test` path.
- Clarify `Pseudoterminal` is v2-only or remove it from Source Notes.
- Confirm `surface.read_text` exists in `control_bridge.rs` before implementation.

Exit status: 0
