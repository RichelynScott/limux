"use strict";

const assert = require("assert");
const {
  allowedMethods,
  buildRequest,
  cursorPaneCreateEmpty,
  cursorWorkspaceOpenFolder,
  surfaceReadText,
  windowPresent,
  workspaceList,
  workspaceSelect,
} = require("./request-builder");

const expectedMethods = [
  "workspace.list",
  "workspace.select",
  "window.present",
  "cursor.pane_create_empty",
  "surface.read_text",
  "cursor.workspace_open_folder",
];

assert.deepStrictEqual(allowedMethods(), expectedMethods);

assert.deepStrictEqual(workspaceList({}, "req-1"), {
  id: "req-1",
  method: "workspace.list",
  params: {},
});
assert.deepStrictEqual(workspaceSelect({ workspace_id: "workspace:1" }), {
  method: "workspace.select",
  params: { workspace_id: "workspace:1" },
});
assert.strictEqual(windowPresent({}).method, "window.present");
assert.strictEqual(cursorPaneCreateEmpty({}).method, "cursor.pane_create_empty");
assert.strictEqual(surfaceReadText({ surface_id: "surface:1:tab" }).method, "surface.read_text");
assert.strictEqual(
  cursorWorkspaceOpenFolder({ folder: "/tmp/limux" }).method,
  "cursor.workspace_open_folder",
);

for (const method of [
  "surface.send_text",
  "surface.send_key",
  "pane.create",
  "pane.create.command",
  "workspace.create",
  "debug.terminal.read_text",
]) {
  assert.throws(
    () => buildRequest(method, {}),
    /restricted Limux method is not allowlisted/,
    `${method} should be rejected`,
  );
}

for (const params of [null, [], "text", 42]) {
  assert.throws(() => buildRequest("workspace.list", params), /params must be a plain object/);
}
