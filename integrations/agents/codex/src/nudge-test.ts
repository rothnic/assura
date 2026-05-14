import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import {
  compareEvaluationRuns,
  createNudgeFromReport,
  parseStructureCheckReport,
  type StructureCheckReport,
} from "./index.js";
import { runCli } from "./cli.js";

const passingReport: StructureCheckReport = {
  success: true,
  project_root: "/repo",
  config_path: "/repo/.assura/config.yml",
  checked_path: "/repo",
  files_checked: 2,
  dirs_checked: 1,
  violations: [],
};

const failingReport: StructureCheckReport = {
  success: false,
  project_root: "/repo",
  config_path: "/repo/.assura/config.yml",
  checked_path: "/repo",
  files_checked: 2,
  dirs_checked: 1,
  violations: [
    {
      path: "/repo/BadName.ts",
      rule: "file_naming",
      message: "File name 'BadName' does not match kebab-case",
      severity: "medium",
    },
  ],
};

test("passing Assura JSON produces no blocking nudge", () => {
  const nudge = createNudgeFromReport(passingReport);

  assert.equal(nudge.status, "pass");
  assert.equal(nudge.violationCount, 0);
  assert.equal(nudge.messages.length, 0);
  assert.equal(nudge.metrics.nudgeCount, 0);
});

test("failing Assura JSON produces actionable nudge content", () => {
  const nudge = createNudgeFromReport(failingReport);

  assert.equal(nudge.status, "fail");
  assert.equal(nudge.violationCount, 1);
  assert.deepEqual(nudge.affectedRules, ["file_naming"]);
  assert.match(nudge.summary, /advisory/);
  assert.match(nudge.messages[0]?.guidance.join(" "), /Rename the file/);
  assert.ok(nudge.messages[0]?.references.includes(".assura/config.yml"));
});

test("invalid JSON is rejected with a clear error", () => {
  assert.throws(
    () => parseStructureCheckReport("{"),
    /Invalid Assura JSON report/
  );

  assert.throws(
    () => parseStructureCheckReport(JSON.stringify({ success: true })),
    /expected StructureCheckReport fields/
  );
});

test("measurement comparison computes precision and loop deltas", () => {
  const comparison = compareEvaluationRuns([
    {
      mode: "instructions_only",
      structuralViolationsIntroduced: 4,
      correctionLoops: 5,
      instructionAdherence: 0.6,
      nudgeCount: 0,
      usefulNudges: 0,
      noisyNudges: 0,
      missedViolations: 4,
    },
    {
      mode: "assura_runtime_nudges",
      structuralViolationsIntroduced: 1,
      correctionLoops: 2,
      instructionAdherence: 0.9,
      nudgeCount: 3,
      usefulNudges: 2,
      noisyNudges: 1,
      missedViolations: 1,
    },
  ]);

  const runtime = comparison.summaries.find(
    (summary) => summary.mode === "assura_runtime_nudges"
  );
  assert.equal(runtime?.nudgePrecision, 2 / 3);
  assert.equal(runtime?.correctionLoopDeltaVsInstructions, -3);
  assert.equal(runtime?.violationDeltaVsInstructions, -3);
});

test("CLI reads a report file and outputs JSON nudge data", () => {
  const dir = mkdtempSync(join(tmpdir(), "assura-codex-nudge-"));
  const reportPath = join(dir, "assura-report.json");
  writeFileSync(reportPath, JSON.stringify(failingReport), "utf8");
  const output: string[] = [];

  const exitCode = runCli(["--report", reportPath, "--format", "json"], {
    readFile: (path) => {
      assert.equal(path, reportPath);
      return JSON.stringify(failingReport);
    },
    write: (message) => output.push(message),
    writeError: (message) => output.push(message),
  });

  assert.equal(exitCode, 1);
  const nudge = JSON.parse(output.join("\n")) as {
    status: string;
    violationCount: number;
  };
  assert.equal(nudge.status, "fail");
  assert.equal(nudge.violationCount, 1);
});
