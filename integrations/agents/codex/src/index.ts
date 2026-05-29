import { spawnSync } from "node:child_process";

export type WorkflowMode =
  | "instructions_only"
  | "agents_skills"
  | "assura_runtime_nudges";

export interface StructureViolation {
  path: string;
  rule: string;
  message: string;
  severity: string;
}

export interface StructureCheckReport {
  success: boolean;
  project_root: string;
  config_path: string;
  checked_path: string;
  files_checked: number;
  dirs_checked: number;
  violations: StructureViolation[];
}

export interface NudgeOptions {
  advisory?: boolean;
  guidanceReferences?: string[];
  minimumSeverity?: string;
  maxMessages?: number;
}

export interface NudgeMessage {
  path: string;
  rule: string;
  severity: string;
  problem: string;
  guidance: string[];
  references: string[];
}

export interface NudgeMetrics {
  structuralViolations: number;
  affectedRules: string[];
  affectedPaths: string[];
  nudgeCount: number;
}

export interface NudgeResult {
  status: "pass" | "fail";
  advisory: boolean;
  summary: string;
  violationCount: number;
  suppressedViolationCount: number;
  minimumSeverity: string | null;
  affectedRules: string[];
  messages: NudgeMessage[];
  metrics: NudgeMetrics;
}

export type FeedbackUsefulness = "useful" | "noisy" | "mixed";
export type FeedbackTurnBoundary = "same_turn" | "new_turn" | "unknown";

export interface SameTurnFeedbackOptions {
  responseSource?: string;
  turnBoundary?: FeedbackTurnBoundary;
  repeatNudgeCount?: number;
  usefulnessByViolationClass?: Record<string, FeedbackUsefulness>;
}

export interface SameTurnFeedbackObservation {
  violationClass: string;
  nudgeCount: number;
  fixedBeforeNewTurn: boolean;
  usefulness: FeedbackUsefulness;
  remainingViolations: number;
  responseSource: string;
  turnBoundary: FeedbackTurnBoundary;
  repeatNudgeCount: number;
}

export interface AssuraCheckRunOptions extends NudgeOptions {
  cwd?: string;
  path?: string;
  assuraBin?: string;
  runner?: AssuraProcessRunner;
}

export interface AssuraCheckRunResult {
  exitCode: number;
  stdout: string;
  stderr: string;
  report: StructureCheckReport;
  nudge: NudgeResult;
}

export interface AssuraProcessResult {
  status: number | null;
  stdout?: string;
  stderr?: string;
  error?: Error;
}

export type AssuraProcessRunner = (
  command: string,
  args: string[],
  options: { cwd?: string; encoding: "utf8" }
) => AssuraProcessResult;

export class AssuraCheckExecutionError extends Error {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;

  constructor(message: string, exitCode: number, stdout: string, stderr: string) {
    super(message);
    this.name = "AssuraCheckExecutionError";
    this.exitCode = exitCode;
    this.stdout = stdout;
    this.stderr = stderr;
  }
}

export interface EvaluationRun {
  mode: WorkflowMode;
  structuralViolationsIntroduced: number;
  correctionLoops: number;
  instructionAdherence: number;
  nudgeCount: number;
  usefulNudges: number;
  noisyNudges: number;
  missedViolations: number;
}

export interface EvaluationModeSummary extends EvaluationRun {
  nudgePrecision: number | null;
  correctionLoopDeltaVsInstructions: number | null;
  violationDeltaVsInstructions: number | null;
}

export interface EvaluationComparison {
  baselineMode: "instructions_only";
  summaries: EvaluationModeSummary[];
}

const DEFAULT_REFERENCES = [
  "AGENTS.md",
  ".agents/skills/",
  ".assura/config.yml",
];

export function parseStructureCheckReport(input: string): StructureCheckReport {
  let parsed: unknown;
  try {
    parsed = JSON.parse(input);
  } catch (error) {
    throw new Error(
      `Invalid Assura JSON report: ${error instanceof Error ? error.message : String(error)}`
    );
  }

  if (!isStructureCheckReport(parsed)) {
    throw new Error(
      "Invalid Assura JSON report: expected StructureCheckReport fields success, project_root, config_path, checked_path, files_checked, dirs_checked, and violations"
    );
  }

  return parsed;
}

export function createNudgeFromReport(
  report: StructureCheckReport,
  options: NudgeOptions = {}
): NudgeResult {
  const advisory = options.advisory ?? true;
  const references = options.guidanceReferences ?? DEFAULT_REFERENCES;
  const minimumSeverity = options.minimumSeverity ?? null;
  const maxMessages = normalizeMaxMessages(options.maxMessages);
  const filteredViolations = report.violations.filter((violation) =>
    meetsMinimumSeverity(violation.severity, minimumSeverity)
  );
  const shownViolations =
    maxMessages === null ? filteredViolations : filteredViolations.slice(0, maxMessages);
  const affectedRules = unique(shownViolations.map((violation) => violation.rule));
  const affectedPaths = unique(shownViolations.map((violation) => violation.path));
  const suppressedViolationCount = report.violations.length - shownViolations.length;

  if (report.success) {
    return {
      status: "pass",
      advisory,
      summary: `Assura passed for ${report.checked_path}; no runtime nudge is needed.`,
      violationCount: 0,
      suppressedViolationCount: 0,
      minimumSeverity,
      affectedRules: [],
      messages: [],
      metrics: {
        structuralViolations: 0,
        affectedRules: [],
        affectedPaths: [],
        nudgeCount: 0,
      },
    };
  }

  const messages = shownViolations.map((violation) => ({
    path: violation.path,
    rule: violation.rule,
    severity: violation.severity,
    problem: violation.message,
    guidance: guidanceForViolation(violation),
    references,
  }));

  return {
    status: "fail",
    advisory,
    summary: summarizeNudge(report, messages.length, affectedRules.length, suppressedViolationCount, advisory, minimumSeverity),
    violationCount: report.violations.length,
    suppressedViolationCount,
    minimumSeverity,
    affectedRules,
    messages,
    metrics: {
      structuralViolations: report.violations.length,
      affectedRules,
      affectedPaths,
      nudgeCount: messages.length,
    },
  };
}

export function renderNudgeStatusLine(nudge: NudgeResult): string {
  if (nudge.status === "pass") {
    return "Assura: pass; no structural nudges.";
  }

  const mode = nudge.advisory ? "advisory" : "blocking";
  const threshold = nudge.minimumSeverity
    ? ` at ${nudge.minimumSeverity}+ severity`
    : "";
  const suppressed =
    nudge.suppressedViolationCount > 0
      ? `; ${nudge.suppressedViolationCount} lower-priority or overflow violation(s) suppressed`
      : "";

  return `Assura: ${nudge.violationCount} violation(s); ${nudge.messages.length} ${mode} nudge(s)${threshold}${suppressed}.`;
}

export function runAssuraCheck(
  options: AssuraCheckRunOptions = {}
): AssuraCheckRunResult {
  const assuraBin = options.assuraBin ?? "assura";
  const checkedPath = options.path ?? ".";
  const runner: AssuraProcessRunner = options.runner ?? spawnSync;
  const result = runner(assuraBin, ["check", "--format", "json", checkedPath], {
    cwd: options.cwd,
    encoding: "utf8",
  });

  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  if (result.error) {
    throw new Error(`Failed to run ${assuraBin}: ${result.error.message}`);
  }

  const exitCode = result.status ?? 1;
  let report: StructureCheckReport;
  try {
    report = parseStructureCheckReport(stdout);
  } catch (error) {
    const parseMessage = error instanceof Error ? error.message : String(error);
    const stderrContext = stderr.trim() ? ` Stderr: ${stderr.trim()}` : "";
    throw new AssuraCheckExecutionError(
      `Assura exited with code ${exitCode} but did not emit a StructureCheckReport JSON report: ${parseMessage}.${stderrContext}`,
      exitCode,
      stdout,
      stderr
    );
  }

  return {
    exitCode,
    stdout,
    stderr,
    report,
    nudge: createNudgeFromReport(report, options),
  };
}

export function compareEvaluationRuns(
  runs: EvaluationRun[]
): EvaluationComparison {
  const baseline = runs.find((run) => run.mode === "instructions_only");
  return {
    baselineMode: "instructions_only",
    summaries: runs.map((run) => summarizeEvaluationRun(run, baseline)),
  };
}

export function observeSameTurnFeedback(
  nudge: NudgeResult,
  afterReport: StructureCheckReport,
  usefulNudges: number,
  noisyNudges: number,
  options: SameTurnFeedbackOptions = {}
): SameTurnFeedbackObservation[] {
  const defaultUsefulness = classifyUsefulness(usefulNudges, noisyNudges);
  const turnBoundary = options.turnBoundary ?? "unknown";
  return nudge.affectedRules.map((violationClass) => {
    const remainingViolations = (afterReport?.violations ?? []).filter(
      (violation) => violation.rule === violationClass
    ).length;
    const nudgeCount = nudge.messages.filter(
      (message) => message.rule === violationClass
    ).length;

    return {
      violationClass,
      nudgeCount,
      fixedBeforeNewTurn:
        remainingViolations === 0 && turnBoundary === "same_turn",
      usefulness:
        options.usefulnessByViolationClass?.[violationClass] ?? defaultUsefulness,
      remainingViolations,
      responseSource: options.responseSource ?? "unspecified",
      turnBoundary,
      repeatNudgeCount: options.repeatNudgeCount ?? 0,
    };
  });
}

export function renderNudgeText(nudge: NudgeResult): string {
  const lines = [renderNudgeStatusLine(nudge), nudge.summary];
  if (nudge.messages.length === 0) {
    return lines.join("\n");
  }

  lines.push("");
  for (const message of nudge.messages) {
    lines.push(`- ${message.path} [${message.rule}/${message.severity}]`);
    lines.push(`  Problem: ${message.problem}`);
    for (const guidance of message.guidance) {
      lines.push(`  Next: ${guidance}`);
    }
    lines.push(`  References: ${message.references.join(", ")}`);
  }

  return lines.join("\n");
}

export function renderNudgeJson(nudge: NudgeResult): string {
  return JSON.stringify(nudge, null, 2);
}

function summarizeEvaluationRun(
  run: EvaluationRun,
  baseline: EvaluationRun | undefined
): EvaluationModeSummary {
  const usefulAndNoisy = run.usefulNudges + run.noisyNudges;
  return {
    ...run,
    nudgePrecision:
      usefulAndNoisy === 0 ? null : run.usefulNudges / usefulAndNoisy,
    correctionLoopDeltaVsInstructions: baseline
      ? run.correctionLoops - baseline.correctionLoops
      : null,
    violationDeltaVsInstructions: baseline
      ? run.structuralViolationsIntroduced -
        baseline.structuralViolationsIntroduced
      : null,
  };
}

function classifyUsefulness(
  usefulNudges: number,
  noisyNudges: number
): FeedbackUsefulness {
  if (usefulNudges > 0 && noisyNudges > 0) {
    return "mixed";
  }
  if (noisyNudges > 0) {
    return "noisy";
  }
  return "useful";
}

function summarizeNudge(
  report: StructureCheckReport,
  shownMessageCount: number,
  shownRuleCount: number,
  suppressedViolationCount: number,
  advisory: boolean,
  minimumSeverity: string | null
): string {
  const threshold = minimumSeverity ? ` at ${minimumSeverity}+ severity` : "";
  const mode = advisory ? "advisory" : "blocking";
  const suppressed =
    suppressedViolationCount > 0
      ? ` ${suppressedViolationCount} violation(s) were suppressed by severity or message-count settings.`
      : "";

  return `Assura found ${report.violations.length} structural violation(s); ${shownMessageCount} ${mode} nudge(s)${threshold} will be shown across ${shownRuleCount} rule(s).${suppressed} This only blocks when the surrounding workflow enforces the Assura exit code or treats this nudge as blocking.`;
}

function normalizeMaxMessages(maxMessages: number | undefined): number | null {
  if (maxMessages === undefined) {
    return null;
  }
  if (!Number.isInteger(maxMessages) || maxMessages < 0) {
    throw new Error("maxMessages must be a non-negative integer");
  }
  return maxMessages;
}

function meetsMinimumSeverity(
  severity: string,
  minimumSeverity: string | null
): boolean {
  if (!minimumSeverity) {
    return true;
  }

  const severityRank = severityRankFor(severity);
  const minimumRank = severityRankFor(minimumSeverity);
  if (severityRank === null || minimumRank === null) {
    return severity === minimumSeverity;
  }
  return severityRank >= minimumRank;
}

function severityRankFor(severity: string): number | null {
  switch (severity.toLowerCase()) {
    case "low":
      return 1;
    case "medium":
      return 2;
    case "high":
      return 3;
    case "critical":
      return 4;
    default:
      return null;
  }
}

function guidanceForViolation(violation: StructureViolation): string[] {
  switch (violation.rule) {
    case "file_naming":
      return [
        "Rename the file to match the configured file naming convention for its directory.",
        "Check `.assura/config.yml` for the effective file naming rule before editing nearby files.",
      ];
    case "directory_naming":
      return [
        "Rename the directory to match the configured directory naming convention.",
        "Update imports, references, and documentation that mention the old path.",
      ];
    case "unexpected_file":
    case "unexpected_directory":
      return [
        "Move, remove, or explicitly allow this path in `.assura/config.yml` if it belongs in the project shape.",
        "Prefer following the existing project shape over adding one-off exceptions.",
      ];
    case "exists_count":
      return [
        "Create, remove, or rename direct children so the configured existence count is satisfied.",
        "Remember that existence count rules apply to direct children only.",
      ];
    default:
      return [
        "Inspect the rule and path, then make the smallest change that satisfies the project shape.",
        "Load the relevant repo-local skill or project documentation before changing policy.",
      ];
  }
}

function isStructureCheckReport(value: unknown): value is StructureCheckReport {
  if (!isRecord(value)) {
    return false;
  }

  return (
    typeof value.success === "boolean" &&
    typeof value.project_root === "string" &&
    typeof value.config_path === "string" &&
    typeof value.checked_path === "string" &&
    typeof value.files_checked === "number" &&
    typeof value.dirs_checked === "number" &&
    Array.isArray(value.violations) &&
    value.violations.every(isStructureViolation)
  );
}

function isStructureViolation(value: unknown): value is StructureViolation {
  if (!isRecord(value)) {
    return false;
  }

  return (
    typeof value.path === "string" &&
    typeof value.rule === "string" &&
    typeof value.message === "string" &&
    typeof value.severity === "string"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function unique(values: string[]): string[] {
  return [...new Set(values)].sort();
}
