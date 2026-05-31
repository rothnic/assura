import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import {
  AssuraCheckExecutionError,
  compareEvaluationRuns,
  createAgentFeedbackFromReport,
  observeSameTurnFeedback,
  parseStructureCheckReport,
  renderCodexHookFeedback,
  renderCodexHookOutput,
  renderAgentFeedbackStatusLine,
  runAssuraCheck,
  type AssuraProcessRunner,
  type StructureCheckReport,
} from "./index.js";
import { runCli } from "./cli.js";
import { runHookCli } from "./hook-cli.js";

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

const mixedSeverityReport: StructureCheckReport = {
  ...failingReport,
  violations: [
    {
      path: "/repo/low-name.ts",
      rule: "file_naming",
      message: "Low severity naming issue",
      severity: "low",
    },
    {
      path: "/repo/high-name.ts",
      rule: "file_naming",
      message: "High severity naming issue",
      severity: "high",
    },
    {
      path: "/repo/critical-name.ts",
      rule: "file_naming",
      message: "Critical severity naming issue",
      severity: "critical",
    },
  ],
};

test("passing Assura JSON produces no blocking feedback", () => {
  const feedback = createAgentFeedbackFromReport(passingReport);

  assert.equal(feedback.status, "pass");
  assert.equal(feedback.violationCount, 0);
  assert.equal(feedback.messages.length, 0);
  assert.equal(feedback.metrics.feedbackCount, 0);
});

test("failing Assura JSON produces actionable feedback content", () => {
  const feedback = createAgentFeedbackFromReport(failingReport);

  assert.equal(feedback.status, "fail");
  assert.equal(feedback.violationCount, 1);
  assert.equal(feedback.suppressedViolationCount, 0);
  assert.deepEqual(feedback.affectedRules, ["file_naming"]);
  assert.match(feedback.summary, /advisory/);
  assert.match(feedback.messages[0]?.guidance.join(" "), /Rename the file/);
  assert.ok(feedback.messages[0]?.references.includes(".assura/config.yml"));
});

test("feedback options filter by severity and cap message count", () => {
  const feedback = createAgentFeedbackFromReport(
    {
      ...failingReport,
      violations: [
        {
          path: "/repo/readme.md",
          rule: "file_naming",
          message: "File name should be README.md",
          severity: "low",
        },
        {
          path: "/repo/packages/ui",
          rule: "exists_count",
          message: "Directory has 0 files matching AGENTS.md, expected 1",
          severity: "high",
        },
        {
          path: "/repo/scratch.md",
          rule: "unexpected_file",
          message: "Unexpected file",
          severity: "critical",
        },
      ],
    },
    { advisory: false, maxMessages: 1, minimumSeverity: "high" }
  );

  assert.equal(feedback.violationCount, 3);
  assert.equal(feedback.messages.length, 1);
  assert.equal(feedback.messages[0]?.rule, "exists_count");
  assert.equal(feedback.suppressedViolationCount, 2);
  assert.match(renderAgentFeedbackStatusLine(feedback), /1 blocking feedback/);
  assert.match(renderAgentFeedbackStatusLine(feedback), /high\+ severity/);
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
      feedbackCount: 0,
      usefulFeedback: 0,
      noisyFeedback: 0,
      missedViolations: 4,
    },
    {
      mode: "assura_runtime_feedback",
      structuralViolationsIntroduced: 1,
      correctionLoops: 2,
      instructionAdherence: 0.9,
      feedbackCount: 3,
      usefulFeedback: 2,
      noisyFeedback: 1,
      missedViolations: 1,
    },
  ]);

  const runtime = comparison.summaries.find(
    (summary) => summary.mode === "assura_runtime_feedback"
  );
  assert.equal(runtime?.feedbackPrecision, 2 / 3);
  assert.equal(runtime?.correctionLoopDeltaVsInstructions, -3);
  assert.equal(runtime?.violationDeltaVsInstructions, -3);
});

test("same-turn feedback observation records fixed and remaining violations", () => {
  const feedback = createAgentFeedbackFromReport({
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

  const observations = observeSameTurnFeedback(feedback, afterReport, 1, 1, {
    responseSource: "codex-test",
    turnBoundary: "same_turn",
    repeatFeedbackCount: 0,
  });

  assert.deepEqual(observations, [
    {
      violationClass: "exists_count",
      feedbackCount: 1,
      fixedBeforeNewTurn: false,
      usefulness: "mixed",
      remainingViolations: 1,
      responseSource: "codex-test",
      turnBoundary: "same_turn",
      repeatFeedbackCount: 0,
    },
    {
      violationClass: "file_naming",
      feedbackCount: 1,
      fixedBeforeNewTurn: true,
      usefulness: "mixed",
      remainingViolations: 0,
      responseSource: "codex-test",
      turnBoundary: "same_turn",
      repeatFeedbackCount: 0,
    },
  ]);
});

test("CLI reads a report file and outputs JSON feedback data", () => {
  const dir = mkdtempSync(join(tmpdir(), "assura-agent-feedback-"));
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
  const feedback = JSON.parse(output.join("\n")) as {
    status: string;
    violationCount: number;
  };
  assert.equal(feedback.status, "fail");
  assert.equal(feedback.violationCount, 1);
});

test("CLI can print a configured status-line feedback", () => {
  const output: string[] = [];

  const exitCode = runCli(
    [
      "--report",
      "assura-report.json",
      "--format",
      "status",
      "--blocking",
      "--minimum-severity",
      "high",
      "--max-messages",
      "1",
    ],
    {
      readFile: () =>
        JSON.stringify({
          ...failingReport,
          violations: [
            failingReport.violations[0],
            {
              path: "/repo/scratch.md",
              rule: "unexpected_file",
              message: "Unexpected file",
              severity: "high",
            },
          ],
        }),
      write: (message) => output.push(message),
      writeError: (message) => output.push(message),
    }
  );

  assert.equal(exitCode, 1);
  assert.match(output.join("\n"), /1 blocking feedback/);
  assert.match(output.join("\n"), /high\+ severity/);
});

test("runAssuraCheck preserves success exit code from JSON report", () => {
  const runner = runnerReturning(0, passingReport);

  const run = runAssuraCheck({ path: "src", runner });

  assert.equal(run.exitCode, 0);
  assert.equal(run.feedback.status, "pass");
  assert.equal(run.report.checked_path, passingReport.checked_path);
});

test("runAssuraCheck preserves validation failure exit code from JSON report", () => {
  const runner = runnerReturning(1, failingReport);

  const run = runAssuraCheck({ assuraBin: "assura-dev", path: ".", runner });

  assert.equal(run.exitCode, 1);
  assert.equal(run.feedback.status, "fail");
  assert.equal(run.feedback.affectedRules[0], "file_naming");
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

test("Codex hook feedback injects advisory additional context by default", () => {
  const evaluation = renderCodexHookFeedback(failingReport, {
    sourceDescription: "reused Assura report from assura-report.json",
  });
  const output = renderCodexHookOutput(evaluation);

  assert.equal(evaluation.exitCode, 0);
  assert.equal(evaluation.blockReason, null);
  assert.equal(output.hookSpecificOutput.hookEventName, "UserPromptSubmit");
  assert.match(
    output.hookSpecificOutput.additionalContext,
    /<assura-feedback>/
  );
  assert.match(
    output.hookSpecificOutput.additionalContext,
    /Check state: reused Assura report/
  );
  assert.match(output.hookSpecificOutput.additionalContext, /Blocking: no/);
});

test("Codex hook feedback filters severity and limits injected messages", () => {
  const evaluation = renderCodexHookFeedback(mixedSeverityReport, {
    minSeverity: "high",
    maxMessages: 1,
    blockMode: "off",
  });

  assert.equal(evaluation.filteredViolationCount, 2);
  assert.equal(evaluation.totalViolationCount, 3);
  assert.match(evaluation.additionalContext, /Assura found 2 structural/);
  assert.match(evaluation.additionalContext, /high-name\.ts/);
  assert.doesNotMatch(evaluation.additionalContext, /low-name\.ts/);
  assert.doesNotMatch(evaluation.additionalContext, /critical-name\.ts/);
  assert.match(evaluation.additionalContext, /Omitted: 1/);
});

test("Codex hook feedback can block on configured violation count", () => {
  const evaluation = renderCodexHookFeedback(mixedSeverityReport, {
    minSeverity: "high",
    blockMode: "violations",
    blockCount: 2,
  });

  assert.equal(evaluation.exitCode, 1);
  assert.match(evaluation.blockReason ?? "", /2 matching violation/);
  assert.match(evaluation.additionalContext, /Blocking: yes/);
});

test("Codex hook CLI reuses report files and preserves advisory default", () => {
  const dir = mkdtempSync(join(tmpdir(), "assura-codex-hook-"));
  const reportPath = join(dir, "assura-report.json");
  writeFileSync(reportPath, JSON.stringify(failingReport), "utf8");
  const output: string[] = [];

  const exitCode = runHookCli(["--report", reportPath], {
    readFile: (path) => {
      assert.equal(path, reportPath);
      return JSON.stringify(failingReport);
    },
    write: (message) => output.push(message),
    writeError: (message) => output.push(message),
  });

  assert.equal(exitCode, 0);
  const hookOutput = JSON.parse(output.join("\n")) as {
    hookSpecificOutput: {
      hookEventName: string;
      additionalContext: string;
    };
  };
  assert.equal(hookOutput.hookSpecificOutput.hookEventName, "UserPromptSubmit");
  assert.match(hookOutput.hookSpecificOutput.additionalContext, /Blocking: no/);
  assert.match(
    hookOutput.hookSpecificOutput.additionalContext,
    /reused Assura report/
  );
});

test("Codex hook CLI blocks only when explicitly configured", () => {
  const output: string[] = [];

  const exitCode = runHookCli(
    ["--report", "assura-report.json", "--block-mode", "violations"],
    {
      readFile: () => JSON.stringify(failingReport),
      write: (message) => output.push(message),
      writeError: (message) => output.push(message),
    }
  );

  assert.equal(exitCode, 1);
  const hookOutput = JSON.parse(output.join("\n")) as {
    hookSpecificOutput: { additionalContext: string };
  };
  assert.match(hookOutput.hookSpecificOutput.additionalContext, /Blocking: yes/);
});

test("Codex hook CLI keeps hook execution errors advisory unless configured", () => {
  const advisoryOutput: string[] = [];
  const advisoryExit = runHookCli(["--path", "."], {
    readFile: () => "",
    write: (message) => advisoryOutput.push(message),
    writeError: (message) => advisoryOutput.push(message),
    runAssuraCheck: () => {
      throw new Error("assura unavailable");
    },
  });

  assert.equal(advisoryExit, 0);
  assert.match(advisoryOutput.join("\n"), /hook error \(assura unavailable\)/);

  const blockingOutput: string[] = [];
  const blockingExit = runHookCli(["--path", ".", "--block-mode", "errors"], {
    readFile: () => "",
    write: (message) => blockingOutput.push(message),
    writeError: (message) => blockingOutput.push(message),
    runAssuraCheck: () => {
      throw new Error("assura unavailable");
    },
  });

  assert.equal(blockingExit, 2);
  assert.match(blockingOutput.join("\n"), /Blocking: yes/);
});

test("Codex hook CLI attributes invalid report errors to report reuse", () => {
  const output: string[] = [];

  const exitCode = runHookCli(["--report", "missing-report.json"], {
    readFile: () => "{not json",
    write: (message) => output.push(message),
    writeError: (message) => output.push(message),
  });

  assert.equal(exitCode, 0);
  const hookOutput = JSON.parse(output[0] ?? "") as {
    hookSpecificOutput: { additionalContext: string };
  };
  assert.match(
    hookOutput.hookSpecificOutput.additionalContext,
    /Check state: reused Assura report from missing-report\.json/
  );
  assert.doesNotMatch(
    hookOutput.hookSpecificOutput.additionalContext,
    /ran assura check --format json/
  );
  assert.match(hookOutput.hookSpecificOutput.additionalContext, /hook error/);
});

test("Codex hook CLI reports malformed arguments as advisory context", () => {
  const output: string[] = [];
  const errors: string[] = [];

  const exitCode = runHookCli(["--unknown"], {
    readFile: () => "",
    write: (message) => output.push(message),
    writeError: (message) => errors.push(message),
  });

  assert.equal(exitCode, 0);
  assert.deepEqual(errors, ["Unknown argument: --unknown"]);
  const hookOutput = JSON.parse(output[0] ?? "") as {
    hookSpecificOutput: { additionalContext: string };
  };
  assert.match(
    hookOutput.hookSpecificOutput.additionalContext,
    /Check state: could not parse assura-codex-hook arguments/
  );
  assert.match(hookOutput.hookSpecificOutput.additionalContext, /Blocking: no/);
});

test("Codex hook CLI can block malformed arguments when error blocking is configured", () => {
  const output: string[] = [];
  const errors: string[] = [];

  const exitCode = runHookCli(["--block-mode", "errors", "--unknown"], {
    readFile: () => "",
    write: (message) => output.push(message),
    writeError: (message) => errors.push(message),
  });

  assert.equal(exitCode, 2);
  assert.deepEqual(errors, ["Unknown argument: --unknown"]);
  const hookOutput = JSON.parse(output[0] ?? "") as {
    hookSpecificOutput: { additionalContext: string };
  };
  assert.match(hookOutput.hookSpecificOutput.additionalContext, /Blocking: yes/);
});

test("Codex hook CLI help describes install-debugging behavior", () => {
  const output: string[] = [];

  const exitCode = runHookCli(["--help"], {
    readFile: () => "",
    write: (message) => output.push(message),
    writeError: (message) => output.push(message),
  });

  assert.equal(exitCode, 0);
  assert.match(output.join("\n"), /--report <path>/);
  assert.match(output.join("\n"), /assura check --format json <path>/);
  assert.match(output.join("\n"), /hookSpecificOutput\.additionalContext/);
  assert.match(output.join("\n"), /--block-count <n>/);
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
