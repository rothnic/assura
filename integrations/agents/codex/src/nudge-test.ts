import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import {
  AssuraCheckExecutionError,
  compareEvaluationRuns,
  createNudgeFromReport,
  observeSameTurnFeedback,
  parseStructureCheckReport,
  runAssuraCheck,
  type AssuraProcessRunner,
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

test("same-turn feedback observation records fixed and remaining violations", () => {
  const nudge = createNudgeFromReport({
    ...failingReport,
    violations: [
      failingReport.violations[0],
      {
        path: "/repo/packages/ui",
        rule: "exists_count",
        message: "Directory has 0 files matching AGENTS.md, expected 1",
        severity: "medium",
      },
    ],
  });
  const afterReport: StructureCheckReport = {
    ...failingReport,
    violations: [
      {
        path: "/repo/packages/ui",
        rule: "exists_count",
        message: "Directory has 0 files matching AGENTS.md, expected 1",
        severity: "medium",
      },
    ],
  };

  const observations = observeSameTurnFeedback(nudge, afterReport, 1, 1, {
    responseSource: "codex-test",
    turnBoundary: "same_turn",
    repeatNudgeCount: 0,
  });

  assert.deepEqual(observations, [
    {
      violationClass: "exists_count",
      nudgeCount: 1,
      fixedBeforeNewTurn: false,
      usefulness: "mixed",
      remainingViolations: 1,
      responseSource: "codex-test",
      turnBoundary: "same_turn",
      repeatNudgeCount: 0,
    },
    {
      violationClass: "file_naming",
      nudgeCount: 1,
      fixedBeforeNewTurn: true,
      usefulness: "mixed",
      remainingViolations: 0,
      responseSource: "codex-test",
      turnBoundary: "same_turn",
      repeatNudgeCount: 0,
    },
  ]);
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

test("runAssuraCheck preserves success exit code from JSON report", () => {
  const runner = runnerReturning(0, passingReport);

  const run = runAssuraCheck({ path: "src", runner });

  assert.equal(run.exitCode, 0);
  assert.equal(run.nudge.status, "pass");
  assert.equal(run.report.checked_path, passingReport.checked_path);
});

test("runAssuraCheck preserves validation failure exit code from JSON report", () => {
  const runner = runnerReturning(1, failingReport);

  const run = runAssuraCheck({ assuraBin: "assura-dev", path: ".", runner });

  assert.equal(run.exitCode, 1);
  assert.equal(run.nudge.status, "fail");
  assert.equal(run.nudge.affectedRules[0], "file_naming");
});

test("runAssuraCheck preserves non-JSON Assura failure exit code", () => {
  const runner: AssuraProcessRunner = () => ({
    status: 4,
    stdout: "",
    stderr: "Error: no .assura/config.yml found",
  });

  assert.throws(
    () => runAssuraCheck({ runner }),
    (error: unknown) =>
      error instanceof AssuraCheckExecutionError &&
      error.exitCode === 4 &&
      error.message.includes("no .assura/config.yml") &&
      error.stderr.includes("no .assura/config.yml")
  );
});

test("direct CLI mode preserves non-JSON Assura failure exit code", () => {
  const errors: string[] = [];
  const result = runCli(["--path", "."], {
    readFile: () => "",
    write: () => undefined,
    writeError: (message) => errors.push(message),
    runAssuraCheck: () => {
      throw new AssuraCheckExecutionError(
        "config missing. Stderr: no config",
        4,
        "",
        "no config"
      );
    },
  });

  assert.equal(result, 4);
  assert.deepEqual(errors, ["config missing. Stderr: no config"]);
});

function runnerReturning(
  status: number,
  report: StructureCheckReport
): AssuraProcessRunner {
  return (command, args) => {
    assert.equal(args[0], "check");
    assert.equal(args[1], "--format");
    assert.equal(args[2], "json");
    assert.ok(command.length > 0);
    return {
      status,
      stdout: JSON.stringify(report),
      stderr: "",
    };
  };
}
