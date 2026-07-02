"use strict";

const { mkdirSync, readFileSync, writeFileSync } = require("node:fs");
const { join } = require("node:path");
const { spawnSync } = require("node:child_process");

const root = process.cwd();
const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = join(root, "dist");
const outputPath = join(outputDir, "assura-vscode-package-manifest.json");

const packagedFiles = [
  "README.md",
  "package.json",
  "src/assura-client.js",
  "src/extension.js",
];

const doctor = spawnSync(process.execPath, ["scripts/doctor.js"], {
  cwd: root,
  encoding: "utf8",
});
if (doctor.status !== 0) {
  process.stderr.write(doctor.stdout);
  process.stderr.write(doctor.stderr);
  process.exit(doctor.status);
}

mkdirSync(outputDir, { recursive: true });
writeFileSync(
  outputPath,
  `${JSON.stringify(
    {
      schema: "assura.vscode.local-package.v1",
      name: manifest.name,
      version: manifest.version,
      support: manifest.assura.support,
      marketplace: manifest.assura.marketplace,
      contracts: manifest.assura.contracts,
      activationEvents: manifest.activationEvents,
      commands: manifest.contributes.commands.map((command) => command.command),
      files: packagedFiles,
      install: {
        localDevelopmentHost:
          "code --extensionDevelopmentPath integrations/editors/vscode",
        marketplacePublication: "deferred",
      },
    },
    null,
    2,
  )}\n`,
);

console.log(`Assura VS Code package smoke wrote ${outputPath}.`);
