#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import {
  AssuraCheckExecutionError,
  createNudgeFromReport,
  parseStructureCheckReport,
  renderNudgeJson,
  renderNudgeText,
  runAssuraCheck as executeAssuraCheck,
} from "./index.js";

interface CliArgs {
  reportPath?: string;
  format: "json" | "text";
  assuraBin?: string;
  checkedPath: string;
  help: boolean;
}

export interface CliIo {
  readFile(path: string): string;
  write(message: string): void;
  writeError(message: string): void;
  runAssuraCheck?: typeof executeAssuraCheck;
}

export function runCli(
  argv: string[],
  io: CliIo = {
    readFile: (path) => readFileSync(path, "utf8"),
    write: (message) => console.log(message),
    writeError: (message) => console.error(message),
  }
): number {
  let args: CliArgs;
  try {
    args = parseArgs(argv);
  } catch (error) {
    io.writeError(error instanceof Error ? error.message : String(error));
    return 2;
  }

  if (args.help) {
    io.write(helpText());
    return 0;
  }

  try {
    if (args.reportPath) {
      const report = parseStructureCheckReport(io.readFile(args.reportPath));
      const nudge = createNudgeFromReport(report);
      printNudge(nudge, args.format, io);
      return report.success ? 0 : 1;
    }

    const runAssuraCheck = io.runAssuraCheck ?? executeAssuraCheck;
    const run = runAssuraCheck({
      assuraBin: args.assuraBin,
      path: args.checkedPath,
    });
    printNudge(run.nudge, args.format, io);
    return run.exitCode;
  } catch (error) {
    io.writeError(error instanceof Error ? error.message : String(error));
    return error instanceof AssuraCheckExecutionError ? error.exitCode : 2;
  }
}

if (pathToFileURL(process.argv[1] ?? "").href === import.meta.url) {
  process.exit(runCli(process.argv.slice(2)));
}

function parseArgs(argv: string[]): CliArgs {
  const parsed: CliArgs = {
    format: "text",
    checkedPath: ".",
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
      case "--format": {
        const format = requiredValue(argv, index, "--format");
        if (format !== "json" && format !== "text") {
          throw new Error("--format must be either json or text");
        }
        parsed.format = format;
        index += 1;
        break;
      }
      case "--assura-bin":
        parsed.assuraBin = requiredValue(argv, index, "--assura-bin");
        index += 1;
        break;
      case "--path":
        parsed.checkedPath = requiredValue(argv, index, "--path");
        index += 1;
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return parsed;
}

function requiredValue(argv: string[], index: number, flag: string): string {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function printNudge(
  nudge: ReturnType<typeof createNudgeFromReport>,
  format: "json" | "text",
  io: CliIo
): void {
  if (format === "json") {
    io.write(renderNudgeJson(nudge));
  } else {
    io.write(renderNudgeText(nudge));
  }
}

function helpText(): string {
  return `assura-codex-nudge

Create an advisory Codex/agent nudge from Assura structure-check output.

Usage:
  assura-codex-nudge --report assura-report.json [--format text|json]
  assura-codex-nudge [--path .] [--assura-bin assura] [--format text|json]

Exit codes:
  0  Assura report passed
  1  Assura report contained validation failures
  2  Nudge CLI error or invalid report
`;
}
