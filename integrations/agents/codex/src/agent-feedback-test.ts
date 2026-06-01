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
      corrective_context:
        "Rename the file to match the effective naming rule, or update files.naming when the policy is stale.",
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
  assert.equal(
    feedback.messages[0]?.correctiveContext,
    "Rename the file to match the effective naming rule, or update files.naming when the policy is stale."
  );
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
  assert.equal(feedback.messages[0]?.rule, "unexpected_file");
  assert.equal(feedback.messages[0]?.severity, "critical");
  assert.equal(feedback.suppressedViolationCount, 2);
  assert.match(renderAgentFeedbackStatusLine(feedback), /1 blocking feedback/);
  assert.match(renderAgentFeedbackStatusLine(feedback), /high\+ severity/);
});

test("feedback priority uses deterministic tie breakers before truncation", () => {
  const feedback = createAgentFeedbackFromReport(
    {
      ...failingReport,
      violations: [
        {
          path: "/repo/zeta.ts",
          rule: "file_naming",
          message: "Zeta naming issue",
          severity: "high",
        },
        {
          path: "/repo/alpha.ts",
          rule: "file_naming",
          message: "Alpha naming issue",
          severity: "high",
        },
        {
          path: "/repo/critical.ts",
          rule: "unexpected_file",
          message: "Critical issue",
          severity: "critical",
        },
      ],
    },
    { maxMessages: 2 }
  );

  assert.deepEqual(
    feedback.messages.map((message) => [message.severity, message.path]),
    [
      ["critical", "/repo/critical.ts"],
      ["high", "/repo/alpha.ts"],
    ]
  );
  assert.equal(feedback.suppressedViolationCount, 1);
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
  assert.match(evaluation.additionalContext, /critical-name\.ts/);
  assert.doesNotMatch(evaluation.additionalContext, /low-name\.ts/);
  assert.doesNotMatch(evaluation.additionalContext, /high-name\.ts/);
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
