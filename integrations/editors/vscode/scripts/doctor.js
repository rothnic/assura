"use strict";

const { existsSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const root = process.cwd();
const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

const requiredFiles = [
  "README.md",
  "package.json",
  "src/assura-client.js",
  "src/extension.js",
  "tests/assura-client.test.js",
  "scripts/build-smoke.js",
  "scripts/doctor.js",
  "scripts/package-smoke.js",
];

const requiredCommands = [
  "assura.refreshDiagnostics",
  "assura.daemonStatus",
  "assura.daemonStart",
  "assura.daemonStop",
  "assura.daemonRestart",
  "assura.daemonDoctor",
  "assura.daemonLogs",
  "assura.safeFixPreview",
];

const requiredContracts = [
  "assura check --format json",
  "assura daemon ... --format json",
  "assura daemon check-path ... --format json",
  "assura editor session",
  "assura fix markdown --dry-run --format json",
];

function fail(message) {
  console.error(`Assura VS Code doctor failed: ${message}`);
  process.exitCode = 1;
}

for (const file of requiredFiles) {
  if (!existsSync(join(root, file))) {
    fail(`missing package file ${file}`);
  }
}

if (manifest.private !== true) {
  fail("package must remain private until marketplace publication is explicitly released");
}

if (manifest.assura?.support !== "supported-beta-local") {
  fail("manifest assura.support must be supported-beta-local");
}

if (manifest.assura?.marketplace !== false) {
  fail("manifest assura.marketplace must be false for this beta milestone");
}

for (const contract of requiredContracts) {
  if (!manifest.assura?.contracts?.includes(contract)) {
    fail(`missing shared contract metadata: ${contract}`);
  }
}

const commands = new Set(
  manifest.contributes?.commands?.map((command) => command.command) ?? [],
);
for (const command of requiredCommands) {
  if (!commands.has(command)) {
    fail(`missing contributed command: ${command}`);
  }
}

if (!manifest.scripts?.test || !manifest.scripts?.build || !manifest.scripts?.package) {
  fail("manifest must expose test, build, and package scripts");
}

if (process.exitCode) {
  process.exit(process.exitCode);
}

console.log(
  `Assura VS Code doctor passed for ${manifest.name}@${manifest.version}.`,
);
