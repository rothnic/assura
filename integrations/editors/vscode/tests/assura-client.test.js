"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const {
  checkArgs,
  daemonArgs,
  daemonSummary,
  diagnosticEntries,
  parseAssuraJsonResult,
  safeFixPreviewArgs,
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

test("workspace path comes from the first VS Code workspace folder", () => {
  assert.equal(
    workspacePathFromFolders([{ uri: { fsPath: "/repo" } }]),
    "/repo",
  );
  assert.equal(workspacePathFromFolders([]), null);
});
