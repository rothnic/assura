/**
 * Codex integration package skeleton.
 *
 * Runtime hook installation and validation feedback behavior are intentionally
 * deferred. This package exists so downstream Codex integration code has a
 * canonical source location alongside other Assura agent integrations.
 */

export interface CodexIntegrationManifest {
  name: string;
  status: "skeleton";
  plannedCapabilities: string[];
}

export const manifest: CodexIntegrationManifest = {
  name: "@assura/codex-integration",
  status: "skeleton",
  plannedCapabilities: [
    "assura-check-feedback",
    "codex-hook-installation",
    "structured-validation-messages",
  ],
};
