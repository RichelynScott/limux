"use strict";

const assert = require("assert");
const {
  RuntimeManager,
  hasRuntimeDiscriminator,
  isStateChangingMethod,
  runtimeQuickPickItems,
} = require("./runtime-manager");

function candidate(source, socketPath) {
  return { source, path: socketPath, explicit: source === "setting" };
}

function managerWith(probes, options = {}) {
  const candidates = probes.map((probe, index) => candidate(probe.source || `candidate-${index}`, probe.path));
  return new RuntimeManager({
    socketPath: options.socketPath || "",
    env: {},
    timeoutMs: 25,
    resolveSocketCandidates: () => candidates,
    probeSocket: async (socketPath) => {
      const probe = probes.find((item) => item.path === socketPath);
      if (!probe) {
        throw new Error(`unexpected probe for ${socketPath}`);
      }
      if (typeof probe.nextIdentity === "function") {
        return {
          path: socketPath,
          connected: true,
          identity: probe.nextIdentity(),
          error: null,
        };
      }
      return {
        path: socketPath,
        connected: probe.connected,
        identity: probe.identity || null,
        error: probe.error || null,
      };
    },
    clientFactory: options.clientFactory,
    notify: options.notify,
    showQuickPick: options.showQuickPick,
  });
}

assert.strictEqual(isStateChangingMethod("workspace.list"), false);
assert.strictEqual(isStateChangingMethod("surface.read_text"), false);
assert.strictEqual(isStateChangingMethod("workspace.select"), true);
assert.strictEqual(isStateChangingMethod("cursor.workspace_open_folder"), true);
assert.strictEqual(hasRuntimeDiscriminator({ name: "limux-control", version: "0.1.19" }), false);
assert.strictEqual(hasRuntimeDiscriminator({ name: "limux-control", pid: 123 }), true);

assert.deepStrictEqual(
  runtimeQuickPickItems([
    {
      path: "/tmp/limux.sock",
      candidate: { source: "LIMUX_CHANNEL", channel: "preview/test" },
      identity: { name: "limux-control", version: "0.1.19", runtime_id: "runtime-a", pid: 101 },
    },
  ]),
  [
    {
      label: "limux-control v0.1.19 runtime-a pid 101",
      description: "LIMUX_CHANNEL / preview/test",
      detail: "/tmp/limux.sock",
      runtime: {
        path: "/tmp/limux.sock",
        candidate: { source: "LIMUX_CHANNEL", channel: "preview/test" },
        identity: {
          name: "limux-control",
          version: "0.1.19",
          runtime_id: "runtime-a",
          pid: 101,
        },
      },
    },
  ],
);

(async () => {
  const single = managerWith([
    {
      path: "/tmp/limux-a.sock",
      connected: true,
      identity: { name: "limux-control", version: "0.1.19", runtime_id: "runtime-a" },
    },
  ]);
  const singleResult = await single.selectRuntime();
  assert.strictEqual(singleResult.state, "selected");
  assert.strictEqual(single.selectedRuntime().path, "/tmp/limux-a.sock");
  assert.strictEqual(single.selectedRuntime().identity.runtime_id, "runtime-a");

  let quickPickItems = null;
  const multiple = managerWith(
    [
      {
        path: "/tmp/limux-a.sock",
        connected: true,
        identity: { name: "limux-control", version: "0.1.19", runtime_id: "runtime-a" },
      },
      {
        path: "/tmp/limux-b.sock",
        connected: true,
        identity: { name: "limux-control", version: "0.1.19", runtime_id: "runtime-b" },
      },
    ],
    {
      showQuickPick: async (items) => {
        quickPickItems = items;
        return items[1];
      },
    },
  );
  const multipleResult = await multiple.selectRuntime();
  assert.strictEqual(multipleResult.state, "selected");
  assert.strictEqual(quickPickItems.length, 2);
  assert.strictEqual(multiple.selectedRuntime().identity.runtime_id, "runtime-b");

  let disconnectedNotice = null;
  const disconnected = managerWith(
    [
      { path: "/tmp/stale.sock", connected: false, error: "path exists but is not a socket" },
      { path: "/tmp/timeout.sock", connected: false, error: "timeout" },
    ],
    {
      notify: (message) => {
        disconnectedNotice = message;
      },
    },
  );
  const disconnectedResult = await disconnected.selectRuntime();
  assert.strictEqual(disconnectedResult.state, "disconnected");
  assert.strictEqual(disconnectedResult.selected, null);
  assert.match(disconnectedResult.message, /No Limux runtime sockets/);
  assert.match(disconnectedNotice, /No Limux runtime sockets/);

  let identity = { name: "limux-control", version: "0.1.19", runtime_id: "runtime-original" };
  let clientCalled = false;
  const changed = managerWith(
    [
      {
        path: "/tmp/limux-a.sock",
        connected: true,
        nextIdentity: () => identity,
      },
    ],
    {
      clientFactory: () => {
        clientCalled = true;
        return {
          sendRequest: async () => ({ ok: true }),
        };
      },
    },
  );
  await changed.selectRuntime();
  identity = { name: "limux-control", version: "0.1.19", runtime_id: "runtime-restarted" };
  await assert.rejects(
    () => changed.sendRequest("workspace.select", { workspace_id: "workspace-a" }),
    /runtime identity changed/,
  );
  assert.strictEqual(clientCalled, false);

  let oldHostClientCalled = false;
  const oldHost = managerWith(
    [
      {
        path: "/tmp/limux-old.sock",
        connected: true,
        identity: { name: "limux-control", protocol: "v1+v2", version: "0.1.19" },
      },
    ],
    {
      clientFactory: () => {
        oldHostClientCalled = true;
        return {
          sendRequest: async () => ({ ok: true }),
        };
      },
    },
  );
  await oldHost.selectRuntime();
  await assert.rejects(
    () => oldHost.sendRequest("workspace.select", { workspace_id: "workspace-a" }),
    /missing a runtime discriminator/,
  );
  assert.strictEqual(oldHostClientCalled, false);

  let sent = null;
  const stable = managerWith(
    [
      {
        path: "/tmp/limux-a.sock",
        connected: true,
        identity: { name: "limux-control", version: "0.1.19", runtime_id: "runtime-a" },
      },
    ],
    {
      clientFactory: (socketPath) => ({
        sendRequest: async (method, params) => {
          sent = { socketPath, method, params };
          return { selected: true };
        },
      }),
    },
  );
  await stable.selectRuntime();
  const sendResult = await stable.sendRequest("workspace.select", { workspace_id: "workspace-a" });
  assert.deepStrictEqual(sendResult, { selected: true });
  assert.deepStrictEqual(sent, {
    socketPath: "/tmp/limux-a.sock",
    method: "workspace.select",
    params: { workspace_id: "workspace-a" },
  });
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
