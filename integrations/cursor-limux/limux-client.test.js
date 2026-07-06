"use strict";

const assert = require("assert");
const fs = require("fs");
const net = require("net");
const os = require("os");
const path = require("path");
const { LimuxClient, encodeRequest, parseResponse } = require("./limux-client");

function tempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "limux-client-test-"));
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
    const archiveRoot = path.join(os.tmpdir(), "limux-client-test-archive");
    fs.mkdirSync(archiveRoot, { recursive: true });
    fs.renameSync(dir, path.join(archiveRoot, path.basename(dir)));
  }
}

assert.strictEqual(encodeRequest({ method: "system.ping", params: {} }), '{"method":"system.ping","params":{}}\n');
assert.deepStrictEqual(parseResponse('{"ok":true,"result":{"pong":true}}'), { pong: true });
assert.deepStrictEqual(parseResponse('{"result":{"pong":true}}'), { pong: true });
assert.throws(() => parseResponse("not json"), /invalid Limux response JSON/);
assert.throws(() => parseResponse('{"error":"boom"}'), /boom/);
assert.throws(() => parseResponse("{}"), /invalid Limux response envelope/);

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
      assert.strictEqual(request.method, "workspace.list");
      assert.deepStrictEqual(request.params, { limit: 1 });
      socket.end(
        `${JSON.stringify({
          id: request.id,
          ok: true,
          result: { workspaces: [{ id: "w1" }] },
        })}\n`,
      );
    });
  }, async (socketPath) => {
    const client = new LimuxClient(socketPath, { timeoutMs: 500 });
    const result = await client.sendRequest("workspace.list", { limit: 1 });
    assert.deepStrictEqual(result, { workspaces: [{ id: "w1" }] });
  });

  await withServer((socket) => {
    socket.on("data", () => socket.end("not json\n"));
  }, async (socketPath) => {
    const client = new LimuxClient(socketPath, { timeoutMs: 500 });
    await assert.rejects(() => client.sendRequest("workspace.list"), /invalid Limux response JSON/);
  });

  await withServer(() => {}, async (socketPath) => {
    const client = new LimuxClient(socketPath, { timeoutMs: 25 });
    await assert.rejects(() => client.sendRequest("workspace.list"), /timeout/);
  });

  await withServer((socket) => {
    socket.destroy();
  }, async (socketPath) => {
    const client = new LimuxClient(socketPath, { timeoutMs: 500 });
    await assert.rejects(
      () => client.sendRequest("workspace.list"),
      /connection closed before response|ECONNRESET/,
    );
  });

  const client = new LimuxClient("/tmp/limux.sock");
  await assert.rejects(() => client.sendRequest("", {}), /method must be a non-empty string/);
  await assert.rejects(() => client.sendRequest("workspace.list", []), /params must be a plain object/);
  await assert.rejects(
    () => client.sendRequest("workspace.list", { bad: BigInt(1) }),
    /invalid Limux request JSON/,
  );

  const circular = {};
  circular.self = circular;
  await assert.rejects(
    () => client.sendRequest("workspace.list", circular),
    /invalid Limux request JSON/,
  );
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
