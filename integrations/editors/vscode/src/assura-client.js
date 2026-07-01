"use strict";

const { execFile, spawn } = require("node:child_process");
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

function daemonCheckPathArgs(workspacePath, changedPath) {
  return [
    "daemon",
    "check-path",
    workspacePath,
    "--changed",
    changedPath,
    "--format",
    "json",
  ];
}

function checkArgs(workspacePath) {
  return ["check", "--format", "json", workspacePath];
}

function editorSessionArgs(workspacePath) {
  return ["editor", "session", workspacePath];
}

function editorDiagnosticsRequest(uri) {
  return {
    request_id: `diagnostics:${uri}`,
    method: "textDocument/diagnostics",
    params: {
      textDocument: { uri },
    },
  };
}

function editorContextRequest(uri, options = {}) {
  return {
    request_id: `context:${uri}`,
    method: "textDocument/context",
    params: {
      uri,
      text: options.text ?? uri,
      limit: options.limit ?? 10,
    },
  };
}

function editorCodeActionRequest(uri) {
  return {
    request_id: `code-action:${uri}`,
    method: "textDocument/codeAction",
    params: {
      uri,
    },
  };
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

function runEditorSessionRequest(assuraPath, workspacePath, request) {
  return new Promise((resolve, reject) => {
    const child = spawn(assuraPath, editorSessionArgs(workspacePath), {
      cwd: workspacePath,
      windowsHide: true,
      stdio: ["pipe", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", reject);
    child.on("close", (code) => {
      try {
        const firstLine = stdout.split(/\r?\n/).find((line) => line.trim());
        if (firstLine) {
          resolve(JSON.parse(firstLine));
          return;
        }
        const error = new Error(`assura editor session exited ${code}`);
        error.stderr = stderr;
        error.stdout = stdout;
        reject(error);
      } catch (error) {
        error.stderr = stderr;
        error.stdout = stdout;
        reject(error);
      }
    });

    child.stdin.end(`${JSON.stringify(request)}\n`);
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

function relativeWorkspacePath(workspacePath, filePath) {
  return path.relative(workspacePath, filePath).replace(/\\/g, "/") || ".";
}

function workspaceRelativeFilePath(workspacePath, filePath) {
  const relative = path.relative(workspacePath, filePath);
  if (!relative) {
    return ".";
  }
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    return null;
  }
  return relative.replace(/\\/g, "/");
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

function lspSeverityRank(severity) {
  switch (severity) {
    case 1:
      return 0;
    case 2:
      return 1;
    case 3:
      return 2;
    case 4:
      return 3;
    default:
      return 1;
  }
}

function structureViolations(payload) {
  if (Array.isArray(payload?.violations)) {
    return payload.violations;
  }
  if (Array.isArray(payload?.report?.violations)) {
    return payload.report.violations;
  }
  return [];
}

function diagnosticEntries(report, options = {}) {
  const maxDiagnostics = options.maxDiagnostics ?? 100;
  const editorDiagnostics = report?.result?.diagnostics;
  if (Array.isArray(editorDiagnostics)) {
    return editorDiagnostics.slice(0, maxDiagnostics).map((diagnostic) => {
      const relativePath =
        diagnostic.data?.path ?? report.result?.path ?? options.changedPath ?? ".";
      return {
        path: relativePath,
        absolutePath: path.resolve(options.workspacePath ?? ".", relativePath),
        severity: lspSeverityRank(diagnostic.severity),
        range: diagnostic.range,
        message: diagnostic.message ?? "Assura editor diagnostic",
        source: diagnostic.source ?? "assura",
        code: diagnostic.code ?? "assura",
        data: diagnostic.data ?? diagnostic,
      };
    });
  }

  return structureViolations(report).slice(0, maxDiagnostics).map((violation) => {
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
  daemonCheckPathArgs,
  daemonSummary,
  diagnosticEntries,
  editorCodeActionRequest,
  editorContextRequest,
  editorDiagnosticsRequest,
  editorSessionArgs,
  parseAssuraJsonResult,
  relativeWorkspacePath,
  runAssuraJson,
  runEditorSessionRequest,
  safeFixPreviewArgs,
  workspaceRelativeFilePath,
  workspacePathFromFolders,
};
