"use strict";

const vscode = require("vscode");
const {
  checkArgs,
  daemonArgs,
  daemonSummary,
  diagnosticEntries,
  runAssuraJson,
  safeFixPreviewArgs,
  workspacePathFromFolders,
} = require("./assura-client");

function assuraPath() {
  return vscode.workspace.getConfiguration("assura").get("path", "assura");
}

function maxDiagnostics() {
  return vscode.workspace.getConfiguration("assura").get("maxDiagnostics", 100);
}

function workspacePath() {
  return workspacePathFromFolders(vscode.workspace.workspaceFolders);
}

function diagnosticRange() {
  return new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 1));
}

function toVsCodeDiagnostics(entries) {
  const grouped = new Map();
  for (const entry of entries) {
    const diagnostic = new vscode.Diagnostic(
      diagnosticRange(),
      entry.message,
      entry.severity,
    );
    diagnostic.source = entry.source;
    diagnostic.code = entry.code;
    diagnostic.data = entry.data;

    const current = grouped.get(entry.absolutePath) ?? [];
    current.push(diagnostic);
    grouped.set(entry.absolutePath, current);
  }
  return grouped;
}

async function showJsonDocument(title, payload) {
  const document = await vscode.workspace.openTextDocument({
    content: JSON.stringify(payload, null, 2),
    language: "json",
  });
  await vscode.window.showTextDocument(document, { preview: true });
  return title;
}

async function refreshDiagnostics(collection) {
  const root = workspacePath();
  if (!root) {
    vscode.window.showWarningMessage("Assura needs an open workspace folder.");
    return;
  }

  const report = await runAssuraJson(assuraPath(), checkArgs(root), root);
  collection.clear();
  for (const [filePath, diagnostics] of toVsCodeDiagnostics(
    diagnosticEntries(report, {
      workspacePath: root,
      maxDiagnostics: maxDiagnostics(),
    }),
  )) {
    collection.set(vscode.Uri.file(filePath), diagnostics);
  }
}

async function runDaemonAction(action, statusBar) {
  const root = workspacePath();
  if (!root) {
    vscode.window.showWarningMessage("Assura needs an open workspace folder.");
    return null;
  }

  const payload = await runAssuraJson(assuraPath(), daemonArgs(action, root), root);
  const summary = daemonSummary(payload);
  statusBar.text = summary.label;
  statusBar.tooltip = summary.recoveryCommand;
  statusBar.show();

  if (!summary.isHealthy && action === "status") {
    vscode.window.showWarningMessage(
      `Assura daemon is ${summary.state}. Run Assura: Daemon Doctor for recovery details.`,
    );
  }
  if (action === "doctor" || action === "logs") {
    await showJsonDocument(`Assura daemon ${action}`, payload);
  }
  return payload;
}

async function previewSafeFixes() {
  const root = workspacePath();
  if (!root) {
    vscode.window.showWarningMessage("Assura needs an open workspace folder.");
    return;
  }

  const payload = await runAssuraJson(assuraPath(), safeFixPreviewArgs(root), root);
  const document = await vscode.workspace.openTextDocument({
    content: JSON.stringify(payload, null, 2),
    language: "json",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection("assura");
  const statusBar = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  statusBar.command = "assura.daemonStatus";
  statusBar.text = "Assura";
  statusBar.show();

  context.subscriptions.push(diagnostics, statusBar);
  context.subscriptions.push(
    vscode.commands.registerCommand("assura.refreshDiagnostics", () =>
      refreshDiagnostics(diagnostics),
    ),
    vscode.commands.registerCommand("assura.daemonStatus", () =>
      runDaemonAction("status", statusBar),
    ),
    vscode.commands.registerCommand("assura.daemonStart", () =>
      runDaemonAction("start", statusBar),
    ),
    vscode.commands.registerCommand("assura.daemonStop", () =>
      runDaemonAction("stop", statusBar),
    ),
    vscode.commands.registerCommand("assura.daemonRestart", () =>
      runDaemonAction("restart", statusBar),
    ),
    vscode.commands.registerCommand("assura.daemonDoctor", () =>
      runDaemonAction("doctor", statusBar),
    ),
    vscode.commands.registerCommand("assura.daemonLogs", () =>
      runDaemonAction("logs", statusBar),
    ),
    vscode.commands.registerCommand("assura.safeFixPreview", previewSafeFixes),
  );

  runDaemonAction("status", statusBar).catch((error) => {
    statusBar.text = "Assura: unavailable";
    statusBar.tooltip = error.message;
  });
  refreshDiagnostics(diagnostics).catch((error) => {
    vscode.window.showWarningMessage(`Assura diagnostics failed: ${error.message}`);
  });
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
