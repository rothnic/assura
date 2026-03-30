/**
 * OpenCode Plugin Types
 * 
 * Type definitions for the OpenCode agent protocol and plugin interface.
 * These types define the contract between OpenCode agents and the Assura plugin.
 */

import { z } from "zod";

/**
 * Plugin configuration schema
 */
export interface PluginConfig {
  /** Plugin name */
  name: string;
  /** Plugin version */
  version: string;
  /** Plugin description */
  description?: string;
  /** Hook handlers configuration */
  hooks?: {
    /** Enable preToolUse hook */
    preToolUse?: boolean;
    /** Enable postToolUse hook */
    postToolUse?: boolean;
    /** Enable onError hook */
    onError?: boolean;
  };
  /** Validation configuration */
  validation?: {
    /** Strict mode - block all invalid operations */
    strict: boolean;
    /** Severity threshold for blocking (Critical, High, Medium, Low) */
    severityThreshold: SeverityLevel;
    /** Auto-fix violations when possible */
    autoFix: boolean;
  };
  /** Agent protocol configuration */
  agent?: {
    /** Multi-agent support */
    multiAgent: boolean;
    /** Agent coordination mode */
    coordinationMode: CoordinationMode;
    /** Shared state namespace */
    stateNamespace?: string;
  };
}

/**
 * Severity levels for validation errors
 */
export type SeverityLevel = "Critical" | "High" | "Medium" | "Low";

/**
 * Agent coordination modes
 */
export type CoordinationMode = "leader" | "peer" | "hierarchical";

/**
 * Tool call context passed to hooks
 */
export interface ToolCallContext {
  /** Unique tool call ID */
  callId: string;
  /** Tool name */
  toolName: string;
  /** Tool arguments */
  args: Record<string, unknown>;
  /** Agent ID making the call */
  agentId: string;
  /** Session ID */
  sessionId: string;
  /** Timestamp */
  timestamp: Date;
  /** Call stack trace */
  stackTrace?: string[];
}

/**
 * Tool call result
 */
export interface ToolCallResult {
  /** Success status */
  success: boolean;
  /** Result data */
  data?: unknown;
  /** Error message if failed */
  error?: string;
  /** Execution time in ms */
  executionTime: number;
  /** Modified/created files */
  affectedFiles?: string[];
}

/**
 * Validation error from constraint system
 */
export interface ValidationError {
  /** Error message */
  message: string;
  /** Severity level */
  severity: SeverityLevel;
  /** File path affected */
  filePath: string;
  /** Rule that was violated */
  rule: string;
  /** Line number if applicable */
  line?: number;
  /** Column number if applicable */
  column?: number;
  /** Suggested fix */
  suggestion?: string;
  /** Auto-fix available */
  autoFixable: boolean;
}

/**
 * Hook result - returned from preToolUse and postToolUse hooks
 */
export interface HookResult {
  /** Whether the operation should proceed */
  proceed: boolean;
  /** Validation errors if any */
  errors: ValidationError[];
  /** Modified arguments (for preToolUse) */
  modifiedArgs?: Record<string, unknown>;
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Agent information
 */
export interface AgentInfo {
  /** Agent ID */
  id: string;
  /** Agent name */
  name: string;
  /** Agent role */
  role: string;
  /** Capabilities */
  capabilities: string[];
  /** Current status */
  status: AgentStatus;
  /** Last activity timestamp */
  lastActivity: Date;
}

/**
 * Agent status
 */
export type AgentStatus = "idle" | "busy" | "error" | "offline";

/**
 * Protocol message for agent communication
 */
export interface ProtocolMessage {
  /** Message ID */
  id: string;
  /** Message type */
  type: MessageType;
  /** Sender agent ID */
  sender: string;
  /** Recipient agent ID (empty for broadcast) */
  recipient?: string;
  /** Message payload */
  payload: unknown;
  /** Timestamp */
  timestamp: Date;
  /** Message correlation ID for request/response */
  correlationId?: string;
}

/**
 * Message types for agent protocol
 */
export type MessageType =
  | "REGISTER"
  | "UNREGISTER"
  | "VALIDATE"
  | "VALIDATE_RESULT"
  | "COORDINATE"
  | "COORDINATE_RESULT"
  | "STATE_UPDATE"
  | "STATE_REQUEST"
  | "ERROR"
  | "HEARTBEAT";

/**
 * Shared state entry
 */
export interface StateEntry {
  /** State key */
  key: string;
  /** State value */
  value: unknown;
  /** Owner agent ID */
  owner: string;
  /** Last updated timestamp */
  updatedAt: Date;
  /** Version for optimistic locking */
  version: number;
}

/**
 * Conflict resolution result
 */
export interface ConflictResolution {
  /** Resolved state */
  resolved: StateEntry;
  /** Agents involved in conflict */
  agents: string[];
  /** Resolution strategy used */
  strategy: ResolutionStrategy;
  /** Timestamp of resolution */
  resolvedAt: Date;
}

/**
 * Conflict resolution strategies
 */
export type ResolutionStrategy =
  | "last-write-wins"
  | "first-write-wins"
  | "merge"
  | "manual";

/**
 * Plugin interface - main contract for OpenCode plugins
 */
export interface OpenCodePlugin {
  /** Plugin configuration */
  config: PluginConfig;
  /** Initialize the plugin */
  initialize(): Promise<void>;
  /** Shutdown the plugin */
  shutdown(): Promise<void>;
  /** Pre-tool use hook */
  preToolUse?(context: ToolCallContext): Promise<HookResult>;
  /** Post-tool use hook */
  postToolUse?(
    context: ToolCallContext,
    result: ToolCallResult
  ): Promise<HookResult>;
  /** Error hook */
  onError?(context: ToolCallContext, error: Error): Promise<void>;
}

/**
 * Plugin factory function type
 */
export type PluginFactory = (config: Partial<PluginConfig>) => OpenCodePlugin;

/**
 * Agent registry interface
 */
export interface AgentRegistry {
  /** Register an agent */
  registerAgent(agent: AgentInfo): void;
  /** Unregister an agent */
  unregisterAgent(agentId: string): void;
  /** Get agent by ID */
  getAgent(agentId: string): AgentInfo | undefined;
  /** List all registered agents */
  listAgents(): AgentInfo[];
  /** Update agent status */
  updateAgentStatus(agentId: string, status: AgentStatus): void;
}

/**
 * State manager interface
 */
export interface StateManager {
  /** Get state entry */
  getState(key: string): StateEntry | undefined;
  /** Set state entry */
  setState(key: string, value: unknown, owner: string): void;
  /** Delete state entry */
  deleteState(key: string, owner: string): boolean;
  /** List all state keys */
  listKeys(): string[];
  /** Clear all state */
  clearState(): void;
}

/**
 * Protocol handler interface
 */
export interface ProtocolHandler {
  /** Handle incoming message */
  handleMessage(message: ProtocolMessage): Promise<ProtocolMessage | void>;
  /** Send message to agent */
  sendMessage(message: ProtocolMessage): Promise<void>;
  /** Broadcast message to all agents */
  broadcast(message: Omit<ProtocolMessage, "recipient">): Promise<void>;
}

/**
 * Zod schemas for runtime validation
 */
export const severityLevelSchema = z.enum([
  "Critical",
  "High",
  "Medium",
  "Low",
]);

export const coordinationModeSchema = z.enum([
  "leader",
  "peer",
  "hierarchical",
]);

export const agentStatusSchema = z.enum([
  "idle",
  "busy",
  "error",
  "offline",
]);

export const messageTypeSchema = z.enum([
  "REGISTER",
  "UNREGISTER",
  "VALIDATE",
  "VALIDATE_RESULT",
  "COORDINATE",
  "COORDINATE_RESULT",
  "STATE_UPDATE",
  "STATE_REQUEST",
  "ERROR",
  "HEARTBEAT",
]);

export const pluginConfigSchema = z.object({
  name: z.string(),
  version: z.string(),
  description: z.string().optional(),
  hooks: z
    .object({
      preToolUse: z.boolean().optional(),
      postToolUse: z.boolean().optional(),
      onError: z.boolean().optional(),
    })
    .optional(),
  validation: z
    .object({
      strict: z.boolean(),
      severityThreshold: severityLevelSchema,
      autoFix: z.boolean(),
    })
    .optional(),
  agent: z
    .object({
      multiAgent: z.boolean(),
      coordinationMode: coordinationModeSchema,
      stateNamespace: z.string().optional(),
    })
    .optional(),
});
