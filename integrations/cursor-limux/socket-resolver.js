"use strict";

const fs = require("fs");
const path = require("path");
const { LimuxClient } = require("./limux-client");

const DEFAULT_PREVIEW_ID = "default";

function cleanPath(value) {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function sanitizeChannelId(raw) {
  const value = cleanPath(raw);
  if (!value || value === "." || value === "..") {
    return null;
  }
  return /^[A-Za-z0-9_-]+$/.test(value) ? value : null;
}

function parseRuntimeChannel(env = process.env) {
  const raw = cleanPath(env.LIMUX_CHANNEL);
  if (!raw) {
    return null;
  }

  if (raw === "stable") {
    return { kind: "stable", label: "stable" };
  }

  if (raw === "preview") {
    const id = sanitizeChannelId(env.LIMUX_PREVIEW_ID) || DEFAULT_PREVIEW_ID;
    return { kind: "preview", id, label: `preview/${id}` };
  }

  for (const prefix of ["preview:", "preview/"]) {
    if (raw.startsWith(prefix)) {
      const id = sanitizeChannelId(raw.slice(prefix.length));
      return id ? { kind: "preview", id, label: `preview/${id}` } : null;
    }
  }

  return null;
}

function runtimeChannelSocketPath(env = process.env) {
  const channel = parseRuntimeChannel(env);
  if (!channel) {
    return null;
  }

  const runtimeDir = cleanPath(env.XDG_RUNTIME_DIR);
  if (runtimeDir) {
    const parts =
      channel.kind === "stable"
        ? [runtimeDir, "limux", "stable", "limux.sock"]
        : [runtimeDir, "limux", "preview", channel.id, "limux.sock"];
    return { channel: channel.label, path: path.join(...parts) };
  }

  const fileName =
    channel.kind === "stable" ? "limux-stable.sock" : `limux-preview-${channel.id}.sock`;
  return { channel: channel.label, path: path.join("/tmp", fileName) };
}

function cursorRestrictedSocketPath(socketPath) {
  const parsed = path.parse(socketPath);
  if (parsed.base.endsWith(".cursor.sock")) {
    return socketPath;
  }
  const fileName = parsed.base.endsWith(".sock")
    ? `${parsed.base.slice(0, -".sock".length)}.cursor.sock`
    : `${parsed.base}.cursor`;
  return path.join(parsed.dir, fileName);
}

function pushCandidate(candidates, seen, candidate) {
  if (!candidate.path) {
    return;
  }
  const runtimePath = candidate.path;
  const restrictedPath = cursorRestrictedSocketPath(runtimePath);
  if (seen.has(restrictedPath)) {
    return;
  }
  seen.add(restrictedPath);
  candidates.push({
    ...candidate,
    path: restrictedPath,
    runtimePath,
    restricted: true,
  });
}

function resolveSocketCandidates(options = {}, env = process.env) {
  const candidates = [];
  const seen = new Set();
  const settingPath = cleanPath(options.socketPath);
  const limuxSocket = cleanPath(env.LIMUX_SOCKET);
  const limuxSocketPath = cleanPath(env.LIMUX_SOCKET_PATH);
  const runtimeDir = cleanPath(env.XDG_RUNTIME_DIR);
  const channelSocket = runtimeChannelSocketPath(env);

  pushCandidate(candidates, seen, { source: "setting", path: settingPath, explicit: true });
  pushCandidate(candidates, seen, { source: "LIMUX_SOCKET", path: limuxSocket, explicit: true });
  pushCandidate(candidates, seen, {
    source: "LIMUX_SOCKET_PATH",
    path: limuxSocketPath,
    explicit: true,
  });
  if (channelSocket) {
    pushCandidate(candidates, seen, {
      source: "LIMUX_CHANNEL",
      path: channelSocket.path,
      explicit: false,
      channel: channelSocket.channel,
    });
  }
  if (runtimeDir) {
    pushCandidate(candidates, seen, {
      source: "XDG_RUNTIME_DIR",
      path: path.join(runtimeDir, "limux", "limux.sock"),
      explicit: false,
    });
  }
  pushCandidate(candidates, seen, { source: "fallback", path: "/tmp/limux.sock", explicit: false });

  return candidates;
}

function socketFileLooksValid(socketPath) {
  try {
    return fs.statSync(socketPath).isSocket();
  } catch (error) {
    if (error && error.code === "ENOENT") {
      return true;
    }
    throw error;
  }
}

function probeSocket(socketPath, options = {}) {
  const timeoutMs = options.timeoutMs || 500;

  try {
    if (!socketFileLooksValid(socketPath)) {
      return Promise.resolve({
        path: socketPath,
        connected: false,
        identity: null,
        error: "path exists but is not a socket",
      });
    }
  } catch (error) {
    return Promise.resolve({
      path: socketPath,
      connected: false,
      identity: null,
      error: error.message,
    });
  }

  const client = new LimuxClient(socketPath, { timeoutMs });
  return client
    .sendRequest("system.identify", { caller: "cursor-limux" })
    .then((identity) => ({ path: socketPath, connected: true, identity, error: null }))
    .catch((error) => ({
      path: socketPath,
      connected: false,
      identity: null,
      error: error.code || error.message,
    }));
}

module.exports = {
  cursorRestrictedSocketPath,
  parseRuntimeChannel,
  probeSocket,
  resolveSocketCandidates,
  runtimeChannelSocketPath,
};
