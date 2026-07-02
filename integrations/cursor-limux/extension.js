"use strict";

const vscode = require("vscode");

class LimuxTreeDataProvider {
  getTreeItem(item) {
    return item;
  }

  getChildren() {
    return [];
  }
}

function activate(context) {
  const provider = new LimuxTreeDataProvider();
  context.subscriptions.push(vscode.window.registerTreeDataProvider("limux", provider));
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
