"use strict";

const net = require("net");

function assertPlainObject(value, label) {
  if (
    value === null ||
    Array.isArray(value) ||
    typeof value !== "object" ||
    Object.getPrototypeOf(value) !== Object.prototype
  ) {
    throw new TypeError(`${label} must be a plain object`);
  }
}

function encodeRequest(request) {
  return `${JSON.stringify(request)}\n`;
}

function parseResponse(line) {
  let response;
  try {
    response = JSON.parse(line);
  } catch (error) {
    throw new Error(`invalid Limux response JSON: ${error.message}`);
  }

  if (response && response.ok === true) {
    return response.result === undefined ? null : response.result;
  }
  if (response && Object.prototype.hasOwnProperty.call(response, "result")) {
    return response.result;
  }
  if (response && response.error) {
    throw new Error(
      typeof response.error === "string" ? response.error : JSON.stringify(response.error),
    );
  }
  throw new Error("invalid Limux response envelope");
}

function sendRequest(socketPath, request, options = {}) {
  if (!socketPath || typeof socketPath !== "string") {
    return Promise.reject(new TypeError("socketPath must be a non-empty string"));
  }
  try {
    assertPlainObject(request, "request");
  } catch (error) {
    return Promise.reject(error);
  }

  const timeoutMs = options.timeoutMs || 500;

  return new Promise((resolve, reject) => {
    const socket = net.connect(socketPath);
    let buffer = "";
    let settled = false;
    const timer = setTimeout(() => finishReject(new Error("timeout")), timeoutMs);

    function cleanup() {
      clearTimeout(timer);
      socket.destroy();
    }

    function finishResolve(value) {
      if (settled) {
        return;
      }
      settled = true;
      cleanup();
      resolve(value);
    }

    function finishReject(error) {
      if (settled) {
        return;
      }
      settled = true;
      cleanup();
      reject(error);
    }

    socket.setEncoding("utf8");
    socket.once("connect", () => {
      socket.write(encodeRequest(request));
    });
    socket.on("data", (chunk) => {
      buffer += chunk;
      const newline = buffer.indexOf("\n");
      if (newline === -1) {
        return;
      }
      try {
        finishResolve(parseResponse(buffer.slice(0, newline)));
      } catch (error) {
        finishReject(error);
      }
    });
    socket.once("error", (error) => finishReject(error));
    socket.once("end", () => {
      if (!settled) {
        finishReject(new Error("connection closed before response"));
      }
    });
  });
}

class LimuxClient {
  constructor(socketPath, options = {}) {
    this.socketPath = socketPath;
    this.timeoutMs = options.timeoutMs || 500;
    this.sequence = 0;
  }

  sendRequest(method, params = {}, options = {}) {
    if (typeof method !== "string" || method.trim().length === 0) {
      return Promise.reject(new TypeError("method must be a non-empty string"));
    }
    try {
      assertPlainObject(params, "params");
    } catch (error) {
      return Promise.reject(error);
    }
    this.sequence += 1;
    return sendRequest(
      this.socketPath,
      {
        id: `cursor-${this.sequence}`,
        method,
        params: { ...params },
      },
      { timeoutMs: options.timeoutMs || this.timeoutMs },
    );
  }
}

module.exports = {
  LimuxClient,
  encodeRequest,
  parseResponse,
  sendRequest,
};
