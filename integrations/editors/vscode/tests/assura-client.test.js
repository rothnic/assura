"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const {
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
  safeFixPreviewArgs,
  workspaceRelativeFilePath,
  workspacePathFromFolders,
} = require("../src/assura-client");

test("daemon commands use shared JSON contracts", () => {
  assert.deepEqual(daemonArgs("status", "/repo"), [
    "daemon",
    "status",
    "/repo",
    "--format",
    "json",
  ]);
  assert.deepEqual(daemonArgs("restart", "/repo"), [
    "daemon",
    "restart",
    "/repo",
    "--format",
    "json",
  ]);
  assert.deepEqual(daemonArgs("logs", "/repo", { tail: 25 }), [
    "daemon",
    "logs",
    "/repo",
    "--tail",
    "25",
    "--format",
    "json",
  ]);
});

test("changed document diagnostics use daemon check-path and editor session contracts", () => {
  assert.deepEqual(daemonCheckPathArgs("/repo", "docs/guide.md"), [
    "daemon",
    "check-path",
    "/repo",
    "--changed",
    "docs/guide.md",
    "--format",
    "json",
  ]);
  assert.deepEqual(editorSessionArgs("/repo"), ["editor", "session", "/repo"]);
  assert.deepEqual(editorDiagnosticsRequest("docs/guide.md"), {
    request_id: "diagnostics:docs/guide.md",
    method: "textDocument/diagnostics",
    params: {
      textDocument: { uri: "docs/guide.md" },
    },
  });
});

test("editor context and code-action requests are preview oriented", () => {
  assert.deepEqual(editorContextRequest("docs/guide.md", { text: "guide" }), {
    request_id: "context:docs/guide.md",
    method: "textDocument/context",
    params: {
      uri: "docs/guide.md",
      text: "guide",
      limit: 10,
    },
  });
  assert.deepEqual(editorCodeActionRequest("docs/guide.md"), {
    request_id: "code-action:docs/guide.md",
    method: "textDocument/codeAction",
    params: {
      uri: "docs/guide.md",
    },
  });
});

test("diagnostic commands use one-shot fallback and safe-fix preview only", () => {
  assert.deepEqual(checkArgs("/repo"), ["check", "--format", "json", "/repo"]);
  assert.deepEqual(safeFixPreviewArgs("/repo"), [
    "fix",
    "markdown",
    "/repo",
    "--dry-run",
    "--format",
    "json",
  ]);
  assert(!safeFixPreviewArgs("/repo").includes("--apply"));
});

test("non-zero Assura exits still return valid JSON payloads", () => {
  const error = new Error("exit code 1");
  const payload = parseAssuraJsonResult(
    error,
    JSON.stringify({
      success: false,
      violations: [{ path: "README.md", rule: "markdown_link_target" }],
    }),
    "",
  );

  assert.equal(payload.success, false);
  assert.equal(payload.violations[0].rule, "markdown_link_target");
});

test("non-zero Assura exits without JSON remain failures", () => {
  const error = new Error("exit code 1");
  assert.throws(() => parseAssuraJsonResult(error, "", "failed"), /exit code 1/);
});

test("daemon summary exposes health and recovery command", () => {
  assert.deepEqual(
    daemonSummary({
      health: { state: "unavailable" },
      management: { doctor: "assura daemon doctor --format json /repo" },
    }),
    {
      state: "unavailable",
      label: "Assura: unavailable",
      isHealthy: false,
      recoveryCommand: "assura daemon doctor --format json /repo",
    },
  );
});

test("editor session diagnostics map to VS Code diagnostic entries", () => {
  const diagnostics = diagnosticEntries(
    {
      result: {
        path: "docs/guide.md",
        diagnostics: [
          {
            range: {
              start: { line: 3, character: 2 },
              end: { line: 3, character: 12 },
            },
            severity: 2,
            source: "assura",
            code: "content_runtime:missing_reference",
            message: "Missing reference",
            data: {
              id: "diagnostic-1",
              path: "docs/guide.md",
            },
          },
        ],
      },
    },
    { workspacePath: "/repo" },
  );

  assert.equal(diagnostics.length, 1);
  assert.equal(diagnostics[0].path, "docs/guide.md");
  assert.equal(diagnostics[0].severity, 1);
  assert.equal(diagnostics[0].code, "content_runtime:missing_reference");
  assert.deepEqual(diagnostics[0].range.start, { line: 3, character: 2 });
});

test("structure report violations become bounded diagnostic entries", () => {
  const diagnostics = diagnosticEntries(
    {
      violations: [
        {
          path: "src/BadName.rs",
          rule: "file_naming",
          severity: "high",
          message: "File must be kebab-case",
        },
        {
          path: "README.md",
          rule: "markdown_link_target",
          severity: "low",
          message: "Use a relative link",
        },
      ],
    },
    { workspacePath: "/repo", maxDiagnostics: 1 },
  );

  assert.equal(diagnostics.length, 1);
  assert.equal(diagnostics[0].path, "src/BadName.rs");
  assert.equal(diagnostics[0].code, "file_naming");
  assert.equal(diagnostics[0].severity, 0);
  assert.equal(diagnostics[0].source, "assura");
});

test("workspace-relative paths are portable for daemon changed-path checks", () => {
  assert.equal(relativeWorkspacePath("/repo", "/repo/docs/guide.md"), "docs/guide.md");
  assert.equal(
    workspaceRelativeFilePath("/repo", "/repo/docs/guide.md"),
    "docs/guide.md",
  );
  assert.equal(workspaceRelativeFilePath("/repo", "/repo"), ".");
  assert.equal(workspaceRelativeFilePath("/repo", "/other/docs/guide.md"), null);
  assert.equal(workspaceRelativeFilePath("/repo", "/repo-other/guide.md"), null);
});

test("workspace path comes from the first VS Code workspace folder", () => {
  assert.equal(
    workspacePathFromFolders([{ uri: { fsPath: "/repo" } }]),
    "/repo",
  );
  assert.equal(workspacePathFromFolders([]), null);
});
