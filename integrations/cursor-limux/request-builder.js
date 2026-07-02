"use strict";

const allowlist = require("./methods.json");

const ALLOWED_METHODS = Object.freeze([...allowlist.methods]);
const ALLOWED_METHOD_SET = new Set(ALLOWED_METHODS);

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

function normalizeParams(params) {
  assertPlainObject(params, "params");
  return { ...params };
}

function buildRequest(method, params = {}, id) {
  if (!ALLOWED_METHOD_SET.has(method)) {
    throw new Error(`restricted Limux method is not allowlisted: ${method}`);
  }

  const request = {
    method,
    params: normalizeParams(params),
  };

  if (id !== undefined) {
    request.id = id;
  }

  return request;
}

function allowedMethods() {
  return [...ALLOWED_METHODS];
}

function workspaceList(params, id) {
  return buildRequest("workspace.list", params, id);
}

function workspaceSelect(params, id) {
  return buildRequest("workspace.select", params, id);
}

function windowPresent(params, id) {
  return buildRequest("window.present", params, id);
}

function cursorPaneCreateEmpty(params, id) {
  return buildRequest("cursor.pane_create_empty", params, id);
}

function surfaceReadText(params, id) {
  return buildRequest("surface.read_text", params, id);
}

function cursorWorkspaceOpenFolder(params, id) {
  return buildRequest("cursor.workspace_open_folder", params, id);
}

module.exports = {
  allowedMethods,
  buildRequest,
  workspaceList,
  workspaceSelect,
  windowPresent,
  cursorPaneCreateEmpty,
  surfaceReadText,
  cursorWorkspaceOpenFolder,
};
