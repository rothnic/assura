"use strict";

const { execFile } = require("node:child_process");
const path = require("node:path");

const DAEMON_ACTIONS = new Set([
  "status",
  "start",
  "stop",
  "restart",
  "doctor",
  "logs",
]);

function daemonArgs(action, workspacePath, options = {}) {
  if (!DAEMON_ACTIONS.has(action)) {
    throw new Error(`unsupported daemon action: ${action}`);
  }

  const args = ["daemon", action, workspacePath, "--format", "json"];
  if (action === "logs") {
    args.splice(3, 0, "--tail", String(options.tail ?? 100));
  }
  return args;
}

function checkArgs(workspacePath) {
  return ["check", "--format", "json", workspacePath];
}

function safeFixPreviewArgs(workspacePath) {
  return ["fix", "markdown", workspacePath, "--dry-run", "--format", "json"];
}

function parseAssuraJsonResult(error, stdout, stderr) {
  const output = stdout.trim();
  if (output) {
    try {
      return JSON.parse(output);
    } catch (parseError) {
      parseError.stderr = stderr;
      parseError.stdout = stdout;
      if (!error) {
        throw parseError;
      }
    }
  }

  if (error) {
    error.stderr = stderr;
    error.stdout = stdout;
    throw error;
  }

  return {};
}

function runAssuraJson(assuraPath, args, cwd) {
  return new Promise((resolve, reject) => {
    execFile(assuraPath, args, { cwd, windowsHide: true }, (error, stdout, stderr) => {
      try {
        resolve(parseAssuraJsonResult(error, stdout, stderr));
      } catch (resultError) {
        reject(resultError);
      }
    });
  });
}

function daemonSummary(payload) {
  const state = payload?.health?.state ?? payload?.daemon?.state ?? "unknown";
  const command =
    payload?.management?.doctor ??
    payload?.daemon?.doctor_command ??
    "assura daemon doctor --format json";
  return {
    state,
    label: `Assura: ${state}`,
    isHealthy: state === "running",
    recoveryCommand: command,
  };
}

function severityRank(severity) {
  switch (severity) {
    case "critical":
    case "high":
      return 0;
    case "medium":
      return 1;
    case "low":
      return 2;
    default:
      return 1;
  }
}

function diagnosticEntries(report, options = {}) {
  const maxDiagnostics = options.maxDiagnostics ?? 100;
  const violations = Array.isArray(report?.violations) ? report.violations : [];
  return violations.slice(0, maxDiagnostics).map((violation) => {
    const relativePath =
      violation.path ??
      violation.file ??
      violation.location?.path ??
      violation.details?.path ??
      ".";
    const rule = violation.rule ?? violation.rule_id ?? violation.code ?? "assura";
    const message =
      violation.message ??
      violation.problem ??
      `${rule} failed for ${relativePath}`;
    return {
      path: relativePath,
      absolutePath: path.resolve(options.workspacePath ?? ".", relativePath),
      severity: severityRank(violation.severity),
      message,
      source: "assura",
      code: rule,
      data: violation,
    };
  });
}

function workspacePathFromFolders(workspaceFolders) {
  const folder = workspaceFolders?.[0];
  return folder?.uri?.fsPath ?? null;
}

module.exports = {
  checkArgs,
  daemonArgs,
  daemonSummary,
  diagnosticEntries,
  parseAssuraJsonResult,
  runAssuraJson,
  safeFixPreviewArgs,
  workspacePathFromFolders,
};
