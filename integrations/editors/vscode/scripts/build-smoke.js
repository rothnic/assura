"use strict";

const { readFileSync } = require("node:fs");
const { spawnSync } = require("node:child_process");

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

const manifest = JSON.parse(readFileSync("package.json", "utf8"));
const commands = new Set(
  manifest.contributes.commands.map((command) => command.command),
);

for (const command of requiredCommands) {
  if (!commands.has(command)) {
    throw new Error(`missing contributed command: ${command}`);
  }
}

for (const file of ["src/assura-client.js", "src/extension.js"]) {
  const result = spawnSync(process.execPath, ["--check", file], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    process.stderr.write(result.stderr);
    process.exit(result.status);
  }
}

console.log("Assura VS Code build smoke passed.");
