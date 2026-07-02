"use strict";

const assert = require("assert");
const fs = require("fs");
const net = require("net");
const os = require("os");
const path = require("path");
const {
  probeSocket,
  resolveSocketCandidates,
  runtimeChannelSocketPath,
} = require("./socket-resolver");

function withEnv(env, fn) {
  const previous = {};
  for (const key of Object.keys(env)) {
    previous[key] = process.env[key];
    if (env[key] === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = env[key];
    }
  }
  try {
    return fn();
  } finally {
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
  }
}

function tempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "limux-cursor-test-"));
}

async function withServer(handler, fn) {
  const dir = tempDir();
  const socketPath = path.join(dir, "limux.sock");
  const server = net.createServer(handler);

  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(socketPath, () => {
      server.off("error", reject);
      resolve();
    });
  });

  try {
    return await fn(socketPath);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    const archiveRoot = path.join(os.tmpdir(), "limux-cursor-test-archive");
    fs.mkdirSync(archiveRoot, { recursive: true });
    fs.renameSync(dir, path.join(archiveRoot, path.basename(dir)));
  }
}

assert.deepStrictEqual(
  withEnv(
    {
      LIMUX_SOCKET: "/tmp/from-limux-socket.sock",
      LIMUX_SOCKET_PATH: "/tmp/from-limux-socket-path.sock",
      LIMUX_CHANNEL: "stable",
      XDG_RUNTIME_DIR: "/run/user/1000",
    },
    () => resolveSocketCandidates({ socketPath: "/tmp/from-setting.sock" }),
  ),
  [
    { source: "setting", path: "/tmp/from-setting.sock", explicit: true },
    { source: "LIMUX_SOCKET", path: "/tmp/from-limux-socket.sock", explicit: true },
    { source: "LIMUX_SOCKET_PATH", path: "/tmp/from-limux-socket-path.sock", explicit: true },
    {
      source: "LIMUX_CHANNEL",
      path: "/run/user/1000/limux/stable/limux.sock",
      explicit: false,
      channel: "stable",
    },
    { source: "XDG_RUNTIME_DIR", path: "/run/user/1000/limux/limux.sock", explicit: false },
    { source: "fallback", path: "/tmp/limux.sock", explicit: false },
  ],
);

assert.deepStrictEqual(
  withEnv(
    {
      LIMUX_SOCKET: undefined,
      LIMUX_SOCKET_PATH: undefined,
      LIMUX_CHANNEL: "preview",
      LIMUX_PREVIEW_ID: "custom",
      XDG_RUNTIME_DIR: "/tmp/runtime",
    },
    () => runtimeChannelSocketPath(process.env),
  ),
  {
    channel: "preview/custom",
    path: "/tmp/runtime/limux/preview/custom/limux.sock",
  },
);

assert.deepStrictEqual(
  withEnv(
    {
      LIMUX_CHANNEL: "preview:beta",
      LIMUX_PREVIEW_ID: undefined,
      XDG_RUNTIME_DIR: undefined,
    },
    () => runtimeChannelSocketPath(process.env),
  ),
  {
    channel: "preview/beta",
    path: "/tmp/limux-preview-beta.sock",
  },
);

assert.strictEqual(runtimeChannelSocketPath({ LIMUX_CHANNEL: "bogus" }), null);
assert.strictEqual(runtimeChannelSocketPath({ LIMUX_CHANNEL: "preview:bad/slash" }), null);

(async () => {
  await withServer((socket) => {
    socket.setEncoding("utf8");
    let buffered = "";
    socket.on("data", (chunk) => {
      buffered += chunk;
      const newline = buffered.indexOf("\n");
      if (newline === -1) {
        return;
      }
      const request = JSON.parse(buffered.slice(0, newline));
      assert.strictEqual(request.method, "system.identify");
      socket.end(
        `${JSON.stringify({
          id: request.id,
          ok: true,
          result: { pid: 123, runtime_id: "test-runtime" },
        })}\n`,
      );
    });
  }, async (socketPath) => {
    const result = await probeSocket(socketPath, { timeoutMs: 500 });
    assert.deepStrictEqual(result, {
      path: socketPath,
      connected: true,
      identity: { pid: 123, runtime_id: "test-runtime" },
      error: null,
    });
  });

  const stale = path.join(tempDir(), "stale.sock");
  fs.writeFileSync(stale, "");
  const staleResult = await probeSocket(stale, { timeoutMs: 50 });
  assert.strictEqual(staleResult.connected, false);
  assert.match(staleResult.error, /not a socket|ECONNREFUSED|EINVAL/);

  const timeoutResult = await withServer(() => {}, (socketPath) =>
    probeSocket(socketPath, { timeoutMs: 50 }),
  );
  assert.strictEqual(timeoutResult.connected, false);
  assert.match(timeoutResult.error, /timeout/i);
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
