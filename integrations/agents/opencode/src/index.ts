/**
 * Assura OpenCode Plugin
 *
 * OpenCode plugin for Assura constraint validation system.
 * Provides file system validation and naming convention enforcement
 * for OpenCode agents.
 *
 * @example
 * ```typescript
 * import { createPlugin } from '@assura/opencode-plugin';
 *
 * const plugin = createPlugin({
 *   validation: {
 *     strict: true,
 *     severityThreshold: 'High',
 *     autoFix: false,
 *   },
 *   agent: {
 *     multiAgent: true,
 *     coordinationMode: 'peer',
 *   },
 * });
 *
 * await plugin.initialize();
 *
 * // Use hooks
 * const result = await plugin.preToolUse({
 *   callId: '123',
 *   toolName: 'WriteFile',
 *   args: { filePath: 'src/my-file.ts', content: '...' },
 *   agentId: 'agent-1',
 *   sessionId: 'session-1',
 *   timestamp: new Date(),
 * });
 *
 * if (!result.proceed) {
 *   console.error('Validation failed:', result.errors);
 * }
 * ```
 */

// Export types
export type {
  OpenCodePlugin,
  PluginConfig,
  PluginFactory,
  ToolCallContext,
  ToolCallResult,
  HookResult,
  ValidationError,
  SeverityLevel,
  AgentInfo,
  AgentStatus,
  AgentRegistry,
  ProtocolMessage,
  MessageType,
  ProtocolHandler,
  StateEntry,
  StateManager,
  CoordinationMode,
  ResolutionStrategy,
  ConflictResolution,
} from "./types";

// Export schemas
export {
  pluginConfigSchema,
  severityLevelSchema,
  coordinationModeSchema,
  agentStatusSchema,
  messageTypeSchema,
} from "./types";

// Export plugin
export { AssuraOpenCodePlugin } from "./plugin";

// Export createPlugin factory (re-exported from factory to avoid circular deps)
export { createPlugin } from "./factory";

// Export agent registry
export { AgentRegistryImpl } from "./agent-registry";

// Export state manager
export { StateManagerImpl } from "./state-manager";

// Export protocol handler
export { ProtocolHandlerImpl } from "./protocol";

// Export validation engine
export { ValidationEngine } from "./validation";

// Export hook handlers
export { HookHandlers } from "./hooks";

// Default export
export { createPlugin as default } from "./factory";
