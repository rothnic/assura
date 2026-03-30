/**
 * Plugin Factory
 *
 * Factory function for creating Assura OpenCode plugin instances.
 * Separated from plugin.ts to avoid circular dependency issues.
 */

import { AssuraOpenCodePlugin } from "./plugin";
import type { PluginConfig } from "./types";

/**
 * Create a new Assura OpenCode plugin instance
 */
export function createPlugin(
  config?: Partial<PluginConfig>
): AssuraOpenCodePlugin {
  return new AssuraOpenCodePlugin(config);
}

export default createPlugin;
