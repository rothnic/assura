#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import {
  AssuraCheckExecutionError,
  parseStructureCheckReport,
  runAssuraCheck as executeAssuraCheck,
} from "./index.js";
import {
  parseHookBlockMode,
  parseSeverityName,
  renderCodexHookExecutionError,
  renderCodexHookFeedback,
  renderCodexHookOutput,
  reportSourceDescription,
  type CodexHookOptions,
  type HookBlockMode,
  type SeverityName,
} from "./hook.js";

interface HookCliArgs {
  reportPath?: string;
  assuraBin?: string;
  checkedPath: string;
  minSeverity: SeverityName;
  maxMessages: number;
  blockMode: HookBlockMode;
  blockCount: number;
  help: boolean;
}

export interface HookCliIo {
  readFile(path: string): string;
  write(message: string): void;
  writeError(message: string): void;
  runAssuraCheck?: typeof executeAssuraCheck;
}

export function runHookCli(
  argv: string[],
  io: HookCliIo = {
    readFile: (path) => readFileSync(path, "utf8"),
    write: (message) => console.log(message),
    writeError: (message) => console.error(message),
  }
): number {
  const parseErrorOptions = hookOptionsFromPartialArgv(argv);
  let args: HookCliArgs;
  try {
    args = parseArgs(argv);
  } catch (error) {
    const err = error instanceof Error ? error : new Error(String(error));
    const evaluation = renderCodexHookExecutionError(err, {
      ...parseErrorOptions,
      sourceDescription: "could not parse assura-codex-hook arguments",
    });
    io.write(JSON.stringify(renderCodexHookOutput(evaluation)));
    io.writeError(err.message);
    return evaluation.exitCode;
  }

  if (args.help) {
    io.write(helpText());
    return 0;
  }

  const hookOptions: CodexHookOptions = {
    minSeverity: args.minSeverity,
    maxMessages: args.maxMessages,
    blockMode: args.blockMode,
    blockCount: args.blockCount,
  };
  const sourceDescription = attemptedSourceDescription(args);

  try {
    if (args.reportPath) {
      const report = parseStructureCheckReport(io.readFile(args.reportPath));
      const evaluation = renderCodexHookFeedback(report, {
        ...hookOptions,
        sourceDescription,
      });
      io.write(JSON.stringify(renderCodexHookOutput(evaluation)));
      return evaluation.exitCode;
    }

    const runAssuraCheck = io.runAssuraCheck ?? executeAssuraCheck;
    const run = runAssuraCheck({
      assuraBin: args.assuraBin,
      path: args.checkedPath,
    });
    const evaluation = renderCodexHookFeedback(run.report, {
      ...hookOptions,
      sourceDescription: reportSourceDescription(undefined, run),
    });
    io.write(JSON.stringify(renderCodexHookOutput(evaluation)));
    return evaluation.exitCode;
  } catch (error) {
    const err = error instanceof Error ? error : new Error(String(error));
    const evaluation = renderCodexHookExecutionError(err, {
      ...hookOptions,
      sourceDescription,
    });
    io.write(JSON.stringify(renderCodexHookOutput(evaluation)));
    if (error instanceof AssuraCheckExecutionError) {
      io.writeError(error.message);
    }
    return evaluation.exitCode;
  }
}

if (pathToFileURL(process.argv[1] ?? "").href === import.meta.url) {
  process.exit(runHookCli(process.argv.slice(2)));
}

function parseArgs(argv: string[]): HookCliArgs {
  const parsed: HookCliArgs = {
    checkedPath: ".",
    minSeverity: "info",
    maxMessages: 5,
    blockMode: "off",
    blockCount: 1,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    switch (arg) {
      case "--help":
      case "-h":
        parsed.help = true;
        break;
      case "--report":
        parsed.reportPath = requiredValue(argv, index, "--report");
        index += 1;
        break;
      case "--assura-bin":
        parsed.assuraBin = requiredValue(argv, index, "--assura-bin");
        index += 1;
        break;
      case "--path":
        parsed.checkedPath = requiredValue(argv, index, "--path");
        index += 1;
        break;
      case "--min-severity":
        parsed.minSeverity = parseSeverityName(
          requiredValue(argv, index, "--min-severity")
        );
        index += 1;
        break;
      case "--max-messages":
        parsed.maxMessages = parsePositiveInteger(
          requiredValue(argv, index, "--max-messages"),
          "--max-messages"
        );
        index += 1;
        break;
      case "--block-mode":
        parsed.blockMode = parseHookBlockMode(
          requiredValue(argv, index, "--block-mode")
        );
        index += 1;
        break;
      case "--block-count":
        parsed.blockCount = parsePositiveInteger(
          requiredValue(argv, index, "--block-count"),
          "--block-count"
        );
        index += 1;
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return parsed;
}

function hookOptionsFromPartialArgv(argv: string[]): CodexHookOptions {
  const blockModeIndex = argv.indexOf("--block-mode");
  if (blockModeIndex < 0) {
    return {};
  }

  const blockMode = argv[blockModeIndex + 1];
  if (blockMode === "errors" || blockMode === "all") {
    return { blockMode };
  }

  return {};
}

function requiredValue(argv: string[], index: number, flag: string): string {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function attemptedSourceDescription(args: HookCliArgs): string {
  if (args.reportPath) {
    return reportSourceDescription(args.reportPath, undefined);
  }
  return `ran assura check --format json ${args.checkedPath}`;
}

function parsePositiveInteger(input: string, flag: string): number {
  const value = Number.parseInt(input, 10);
  if (!Number.isInteger(value) || value < 1 || String(value) !== input) {
    throw new Error(`${flag} must be a positive integer`);
  }
  return value;
}

function helpText(): string {
  return `assura-codex-hook

Emit native Codex UserPromptSubmit hook JSON with Assura feedback.

Usage:
  assura-codex-hook --report assura-report.json [options]
  assura-codex-hook [--path .] [--assura-bin assura] [options]

Options:
  --report <path>        Reuse an existing assura check --format json report.
  --path <path>          Run assura check --format json for this path. Default: .
  --assura-bin <bin>     Assura executable to run when --report is omitted. Default: assura.
  --min-severity <name>  info|low|medium|high|critical. Default: info.
  --max-messages <n>     Max path-specific messages injected into Codex. Default: 5.
  --block-mode <mode>    off|violations|errors|all. Default: off.
  --block-count <n>      Matching violation threshold for blocking. Default: 1.

Behavior:
  With --report, the hook reuses that JSON report. Without --report, it runs
  assura check --format json <path>. Feedback is written as Codex hook JSON
  with hookSpecificOutput.additionalContext.

Defaults are advisory: validation failures, hook execution errors, and malformed
arguments are reported to Codex context and exit 0 unless blocking is configured.
`;
}
