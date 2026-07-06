"use strict";

const { LimuxClient } = require("./limux-client");
const { probeSocket, resolveSocketCandidates } = require("./socket-resolver");

const READ_ONLY_METHODS = new Set([
  "system.capabilities",
  "system.identify",
  "system.ping",
  "surface.read_text",
  "workspace.list",
]);

const IDENTITY_KEYS = [
  "runtime_id",
  "instance_id",
  "pid",
  "socket_path",
  "channel",
  "build_id",
  "name",
  "protocol",
  "version",
];

function isPlainObject(value) {
  return (
    value !== null &&
    !Array.isArray(value) &&
    typeof value === "object" &&
    Object.getPrototypeOf(value) === Object.prototype
  );
}

function stableIdentity(identity) {
  if (!isPlainObject(identity)) {
    return {};
  }

  const stable = {};
  for (const key of IDENTITY_KEYS) {
    if (Object.prototype.hasOwnProperty.call(identity, key)) {
      stable[key] = identity[key];
    }
  }
  return stable;
}

function identityFingerprint(identity, socketPath) {
  return JSON.stringify({
    socketPath,
    identity: stableIdentity(identity),
  });
}

function runtimeDisplayName(runtime) {
  const identity = isPlainObject(runtime.identity) ? runtime.identity : {};
  const name = identity.name || "Limux";
  const version = identity.version ? `v${identity.version}` : null;
  const runtimeId = identity.runtime_id || identity.instance_id || null;
  const pid = identity.pid ? `pid ${identity.pid}` : null;
  return [name, version, runtimeId, pid].filter(Boolean).join(" ");
}

function runtimeDescription(runtime) {
  const bits = [runtime.candidate.source];
  if (runtime.candidate.channel) {
    bits.push(runtime.candidate.channel);
  }
  return bits.filter(Boolean).join(" / ");
}

function runtimeQuickPickItems(runtimes) {
  return runtimes.map((runtime) => ({
    label: runtimeDisplayName(runtime),
    description: runtimeDescription(runtime),
    detail: runtime.path,
    runtime,
  }));
}

function isStateChangingMethod(method) {
  return !READ_ONLY_METHODS.has(method);
}

function pinRuntime(runtime) {
  return {
    path: runtime.path,
    candidate: { ...runtime.candidate },
    identity: isPlainObject(runtime.identity) ? { ...runtime.identity } : runtime.identity,
    identityFingerprint: identityFingerprint(runtime.identity, runtime.path),
  };
}

class RuntimeManager {
  constructor(options = {}) {
    this.socketPath = options.socketPath || "";
    this.env = options.env || process.env;
    this.timeoutMs = options.timeoutMs || 500;
    this.resolveSocketCandidates = options.resolveSocketCandidates || resolveSocketCandidates;
    this.probeSocket = options.probeSocket || probeSocket;
    this.clientFactory =
      options.clientFactory || ((socketPath, clientOptions) => new LimuxClient(socketPath, clientOptions));
    this.showQuickPick = options.showQuickPick || null;
    this.notify = options.notify || (() => {});
    this.selected = null;
  }

  clearSelection() {
    this.selected = null;
  }

  selectedRuntime() {
    return this.selected ? { ...this.selected, candidate: { ...this.selected.candidate } } : null;
  }

  async discoverRuntimes(options = {}) {
    const socketPath =
      Object.prototype.hasOwnProperty.call(options, "socketPath") ? options.socketPath : this.socketPath;
    const env = options.env || this.env;
    const timeoutMs = options.timeoutMs || this.timeoutMs;
    const candidates = this.resolveSocketCandidates({ socketPath }, env);

    const probes = await Promise.all(
      candidates.map(async (candidate) => {
        try {
          const result = await this.probeSocket(candidate.path, { timeoutMs });
          return {
            ...result,
            candidate,
            path: result.path || candidate.path,
          };
        } catch (error) {
          return {
            path: candidate.path,
            connected: false,
            identity: null,
            error: error.code || error.message,
            candidate,
          };
        }
      }),
    );

    const runtimes = probes.filter((probe) => probe.connected);
    return {
      state: runtimes.length > 0 ? "connected" : "disconnected",
      candidates,
      probes,
      runtimes,
      message: runtimes.length > 0 ? null : "No Limux runtime sockets are reachable.",
    };
  }

  async pickRuntime(runtimes) {
    if (runtimes.length === 1) {
      return runtimes[0];
    }
    if (!this.showQuickPick) {
      throw new Error("multiple Limux runtimes found, but no QuickPick selector is available");
    }

    const picked = await this.showQuickPick(runtimeQuickPickItems(runtimes), {
      placeHolder: "Select Limux runtime",
      matchOnDescription: true,
      matchOnDetail: true,
    });
    return picked ? picked.runtime : null;
  }

  async selectRuntime(options = {}) {
    const discovery = await this.discoverRuntimes(options);
    if (discovery.runtimes.length === 0) {
      this.clearSelection();
      this.notify(discovery.message);
      return {
        ...discovery,
        selected: null,
      };
    }

    const runtime = await this.pickRuntime(discovery.runtimes);
    if (!runtime) {
      this.clearSelection();
      return {
        ...discovery,
        state: "selection-cancelled",
        selected: null,
        message: "Limux runtime selection was cancelled.",
      };
    }

    this.selected = pinRuntime(runtime);
    return {
      ...discovery,
      state: "selected",
      selected: this.selectedRuntime(),
      message: null,
    };
  }

  async ensureRuntime(options = {}) {
    if (!this.selected || options.forceSelect) {
      const result = await this.selectRuntime(options);
      if (!result.selected) {
        throw new Error(result.message || "No Limux runtime selected.");
      }
    }
    return this.selectedRuntime();
  }

  async verifySelectedRuntime(options = {}) {
    const selected = await this.ensureRuntime(options);
    const timeoutMs = options.timeoutMs || this.timeoutMs;
    const current = await this.probeSocket(selected.path, { timeoutMs });
    if (!current.connected) {
      this.clearSelection();
      throw new Error(`selected Limux runtime is unavailable: ${current.error || "not connected"}`);
    }

    const currentFingerprint = identityFingerprint(current.identity, selected.path);
    if (currentFingerprint !== selected.identityFingerprint) {
      this.clearSelection();
      throw new Error("selected Limux runtime identity changed; select a runtime again before continuing");
    }

    return selected;
  }

  async clientFor(method, options = {}) {
    const selected = isStateChangingMethod(method)
      ? await this.verifySelectedRuntime(options)
      : await this.ensureRuntime(options);
    return this.clientFactory(selected.path, { timeoutMs: options.timeoutMs || this.timeoutMs });
  }

  async sendRequest(method, params = {}, options = {}) {
    const client = await this.clientFor(method, options);
    return client.sendRequest(method, params, options);
  }
}

module.exports = {
  RuntimeManager,
  identityFingerprint,
  isStateChangingMethod,
  runtimeQuickPickItems,
  stableIdentity,
};
