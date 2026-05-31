import {
  createAgentFeedbackFromReport,
  renderAgentFeedbackText,
  type AssuraCheckRunResult,
  type AgentFeedbackResult,
  type StructureCheckReport,
  type StructureViolation,
} from "./index.js";

export type SeverityName = "info" | "low" | "medium" | "high" | "critical";

export type HookBlockMode = "off" | "violations" | "errors" | "all";

export interface CodexHookOptions {
  minSeverity?: SeverityName;
  maxMessages?: number;
  blockMode?: HookBlockMode;
  blockCount?: number;
  sourceDescription?: string;
}

export interface CodexHookEvaluation {
  eventName: "UserPromptSubmit";
  additionalContext: string;
  exitCode: number;
  blockReason: string | null;
  filteredViolationCount: number;
  totalViolationCount: number;
}

export interface CodexHookOutput {
  hookSpecificOutput: {
    hookEventName: "UserPromptSubmit";
    additionalContext: string;
  };
}

const SEVERITY_ORDER: Record<SeverityName, number> = {
  info: 0,
  low: 1,
  medium: 2,
  high: 3,
  critical: 4,
};

export function renderCodexHookFeedback(
  report: StructureCheckReport,
  options: CodexHookOptions = {}
): CodexHookEvaluation {
  const minSeverity = options.minSeverity ?? "info";
  const maxMessages = options.maxMessages ?? 5;
  const blockMode = options.blockMode ?? "off";
  const blockCount = options.blockCount ?? 1;
  const matchingViolations = filterViolations(report.violations, minSeverity);
  const filteredReport = reportWithViolations(report, matchingViolations);
  const feedback = createAgentFeedbackFromReport(filteredReport, {
    advisory: blockMode === "off",
    maxMessages,
  });
  const blockReason = blockingReason({
    blockMode,
    blockCount,
    matchingViolationCount: matchingViolations.length,
    hasError: false,
  });

  return {
    eventName: "UserPromptSubmit",
    additionalContext: renderHookContext({
      feedback,
      minSeverity,
      maxMessages,
      sourceDescription:
        options.sourceDescription ?? "ran assura check --format json",
      matchingViolationCount: matchingViolations.length,
      totalViolationCount: report.violations.length,
      omittedMessageCount: Math.max(0, matchingViolations.length - maxMessages),
      blockMode,
      blockCount,
      blockReason,
      errorMessage: null,
    }),
    exitCode: blockReason ? 1 : 0,
    blockReason,
    filteredViolationCount: matchingViolations.length,
    totalViolationCount: report.violations.length,
  };
}

export function renderCodexHookExecutionError(
  error: Error,
  options: CodexHookOptions = {}
): CodexHookEvaluation {
  const blockMode = options.blockMode ?? "off";
  const blockReason = blockingReason({
    blockMode,
    blockCount: options.blockCount ?? 1,
    matchingViolationCount: 0,
    hasError: true,
  });

  return {
    eventName: "UserPromptSubmit",
    additionalContext: renderHookContext({
      feedback: null,
      minSeverity: options.minSeverity ?? "info",
      maxMessages: options.maxMessages ?? 5,
      sourceDescription:
        options.sourceDescription ?? "ran assura check --format json",
      matchingViolationCount: 0,
      totalViolationCount: 0,
      omittedMessageCount: 0,
      blockMode,
      blockCount: options.blockCount ?? 1,
      blockReason,
      errorMessage: error.message,
    }),
    exitCode: blockReason ? 2 : 0,
    blockReason,
    filteredViolationCount: 0,
    totalViolationCount: 0,
  };
}

export function renderCodexHookOutput(
  evaluation: CodexHookEvaluation
): CodexHookOutput {
  return {
    hookSpecificOutput: {
      hookEventName: evaluation.eventName,
      additionalContext: evaluation.additionalContext,
    },
  };
}

export function reportSourceDescription(
  reportPath: string | undefined,
  run: AssuraCheckRunResult | undefined
): string {
  if (reportPath) {
    return `reused Assura report from ${reportPath}`;
  }

  const checkedPath = run?.report.checked_path ?? ".";
  return `ran assura check --format json ${checkedPath}`;
}

export function parseSeverityName(input: string): SeverityName {
  if (isSeverityName(input)) {
    return input;
  }
  throw new Error(
    "severity must be one of info, low, medium, high, or critical"
  );
}

export function parseHookBlockMode(input: string): HookBlockMode {
  if (
    input === "off" ||
    input === "violations" ||
    input === "errors" ||
    input === "all"
  ) {
    return input;
  }
  throw new Error("block mode must be one of off, violations, errors, or all");
}

function renderHookContext({
  feedback,
  minSeverity,
  maxMessages,
  sourceDescription,
  matchingViolationCount,
  totalViolationCount,
  omittedMessageCount,
  blockMode,
  blockCount,
  blockReason,
  errorMessage,
}: {
  feedback: AgentFeedbackResult | null;
  minSeverity: SeverityName;
  maxMessages: number;
  sourceDescription: string;
  matchingViolationCount: number;
  totalViolationCount: number;
  omittedMessageCount: number;
  blockMode: HookBlockMode;
  blockCount: number;
  blockReason: string | null;
  errorMessage: string | null;
}): string {
  const lines = [
    "<assura-feedback>",
    "Hook event: UserPromptSubmit",
    `Check state: ${sourceDescription}`,
    `Filter: severity >= ${minSeverity}; max messages ${maxMessages}`,
    `Blocking: ${blockReason ?? `no (mode ${blockMode}, threshold ${blockCount})`}`,
  ];

  if (errorMessage) {
    lines.push(`Result: hook error (${errorMessage})`);
    lines.push(
      "Next: fix the hook command or run assura check --format json manually."
    );
    lines.push("</assura-feedback>");
    return lines.join("\n");
  }

  lines.push(
    `Result: ${matchingViolationCount} matching violation(s), ${totalViolationCount} total violation(s)`
  );
  if (omittedMessageCount > 0) {
    lines.push(`Omitted: ${omittedMessageCount} additional matching message(s)`);
  }
  if (matchingViolationCount === 0 && totalViolationCount > 0) {
    lines.push(
      "",
      "Assura found violations, but none matched this hook's severity filter."
    );
  } else if (feedback) {
    lines.push("", renderAgentFeedbackText(feedback));
  }
  lines.push("</assura-feedback>");
  return lines.join("\n");
}

function filterViolations(
  violations: StructureViolation[],
  minSeverity: SeverityName
): StructureViolation[] {
  const minRank = SEVERITY_ORDER[minSeverity];
  return violations.filter((violation) => {
    const severity = normalizeSeverity(violation.severity);
    return severity === null || SEVERITY_ORDER[severity] >= minRank;
  });
}

function reportWithViolations(
  report: StructureCheckReport,
  violations: StructureViolation[]
): StructureCheckReport {
  return {
    ...report,
    success: violations.length === 0,
    violations,
  };
}

function blockingReason({
  blockMode,
  blockCount,
  matchingViolationCount,
  hasError,
}: {
  blockMode: HookBlockMode;
  blockCount: number;
  matchingViolationCount: number;
  hasError: boolean;
}): string | null {
  if ((blockMode === "errors" || blockMode === "all") && hasError) {
    return "yes (hook error and error blocking enabled)";
  }
  if (
    (blockMode === "violations" || blockMode === "all") &&
    matchingViolationCount >= blockCount
  ) {
    return `yes (${matchingViolationCount} matching violation(s) >= threshold ${blockCount})`;
  }
  return null;
}

function normalizeSeverity(value: string): SeverityName | null {
  const normalized = value.toLowerCase();
  return isSeverityName(normalized) ? normalized : null;
}

function isSeverityName(value: string): value is SeverityName {
  return Object.prototype.hasOwnProperty.call(SEVERITY_ORDER, value);
}
