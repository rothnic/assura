/**
 * Assura OpenCode Plugin
 *
 * Main plugin implementation that integrates Assura constraint validation
 * with OpenCode agent protocol.
 */

import type {
  OpenCodePlugin,
  PluginConfig,
  ToolCallContext,
  ToolCallResult,
  HookResult,
  ValidationError,
  AgentInfo,
  ProtocolMessage,
  StateEntry,
  SeverityLevel,
} from "./types";
import { pluginConfigSchema } from "./types";
import { AgentRegistryImpl } from "./agent-registry";
import { StateManagerImpl } from "./state-manager";
import { ProtocolHandlerImpl } from "./protocol";
import { ValidationEngine } from "./validation";

/**
 * Default plugin configuration
 */
const DEFAULT_CONFIG: PluginConfig = {
  name: "assura-opencode-plugin",
  version: "0.1.0",
  description: "Assura constraint validation for OpenCode agents",
  hooks: {
    preToolUse: true,
    postToolUse: true,
    onError: true,
  },
  validation: {
    strict: true,
    severityThreshold: "High",
    autoFix: false,
  },
  agent: {
    multiAgent: false,
    coordinationMode: "peer",
    stateNamespace: "assura",
  },
};

/**
 * Assura OpenCode Plugin implementation
 */
export class AssuraOpenCodePlugin implements OpenCodePlugin {
  public config: PluginConfig;
  private agentRegistry: AgentRegistryImpl;
  private stateManager: StateManagerImpl;
  private protocolHandler: ProtocolHandlerImpl;
  private validationEngine: ValidationEngine;
  private initialized = false;

  constructor(config: Partial<PluginConfig> = {}) {
    this.config = this.mergeConfig(config);
    this.agentRegistry = new AgentRegistryImpl();
    this.stateManager = new StateManagerImpl();
    this.protocolHandler = new ProtocolHandlerImpl(
      this.agentRegistry,
      this.stateManager
    );
    this.validationEngine = new ValidationEngine(this.config.validation);
  }

  /**
   * Merge user config with defaults
   */
  private mergeConfig(userConfig: Partial<PluginConfig>): PluginConfig {
    const merged = {
      ...DEFAULT_CONFIG,
      ...userConfig,
      hooks: {
        ...DEFAULT_CONFIG.hooks,
        ...userConfig.hooks,
      },
      validation: {
        ...DEFAULT_CONFIG.validation,
        ...userConfig.validation,
      },
      agent: {
        ...DEFAULT_CONFIG.agent,
        ...userConfig.agent,
      },
    };

    // Validate configuration
    const result = pluginConfigSchema.safeParse(merged);
    if (!result.success) {
      throw new Error(
        `Invalid plugin configuration: ${result.error.message}`
      );
    }

    return merged as PluginConfig;
  }

  /**
   * Initialize the plugin
   */
  public async initialize(): Promise<void> {
    if (this.initialized) {
      throw new Error("Plugin already initialized");
    }

    console.log(`[AssuraPlugin] Initializing ${this.config.name} v${this.config.version}`);

    // Initialize validation engine
    await this.validationEngine.initialize();

    // Register self as an agent
    this.agentRegistry.registerAgent({
      id: "assura-plugin",
      name: "Assura Constraint Validator",
      role: "validator",
      capabilities: ["file-validation", "constraint-checking", "naming-conventions"],
      status: "idle",
      lastActivity: new Date(),
    });

    this.initialized = true;
    console.log("[AssuraPlugin] Initialization complete");
  }

  /**
   * Shutdown the plugin
   */
  public async shutdown(): Promise<void> {
    if (!this.initialized) {
      return;
    }

    console.log("[AssuraPlugin] Shutting down...");

    // Unregister self
    this.agentRegistry.unregisterAgent("assura-plugin");

    // Cleanup
    await this.validationEngine.shutdown();

    this.initialized = false;
    console.log("[AssuraPlugin] Shutdown complete");
  }

  /**
   * Pre-tool use hook - validates operations before execution
   */
  public async preToolUse(context: ToolCallContext): Promise<HookResult> {
    if (!this.config.hooks?.preToolUse) {
      return { proceed: true, errors: [] };
    }

    console.log(`[AssuraPlugin] preToolUse: ${context.toolName}`);

    const errors: ValidationError[] = [];

    try {
      // Validate tool arguments based on tool type
      switch (context.toolName) {
        case "WriteFile":
        case "EditFile":
        case "Bash": {
          const fileErrors = await this.validateFileOperation(context);
          errors.push(...fileErrors);
          break;
        }
        case "Glob":
        case "Grep": {
          // These are read-only operations, less restrictive
          const readErrors = await this.validateReadOperation(context);
          errors.push(...readErrors);
          break;
        }
        default:
          // Unknown tool, apply general validation
          break;
      }

      // Check if we should block based on errors
      const shouldBlock = this.shouldBlockOperation(errors);

      return {
        proceed: !shouldBlock,
        errors,
        metadata: {
          validatedAt: new Date().toISOString(),
          toolName: context.toolName,
          agentId: context.agentId,
        },
      };
    } catch (error) {
      console.error("[AssuraPlugin] Error in preToolUse:", error);
      return {
        proceed: false,
        errors: [
          {
            message: `Validation error: ${error instanceof Error ? error.message : "Unknown error"}`,
            severity: "Critical",
            filePath: context.args.filePath as string || "unknown",
            rule: "validation-error",
            autoFixable: false,
          },
        ],
      };
    }
  }

  /**
   * Post-tool use hook - validates results after execution
   */
  public async postToolUse(
    context: ToolCallContext,
    result: ToolCallResult
  ): Promise<HookResult> {
    if (!this.config.hooks?.postToolUse) {
      return { proceed: true, errors: [] };
    }

    console.log(`[AssuraPlugin] postToolUse: ${context.toolName}`);

    const errors: ValidationError[] = [];

    try {
      // Validate affected files
      if (result.affectedFiles && result.affectedFiles.length > 0) {
        for (const filePath of result.affectedFiles) {
          const fileErrors = await this.validationEngine.validateFile(filePath);
          errors.push(...fileErrors);
        }
      }

      // Validate created/modified content
      if (result.success && context.toolName === "WriteFile") {
        const content = context.args.content as string;
        const filePath = context.args.filePath as string;
        if (content && filePath) {
          const contentErrors = await this.validationEngine.validateContent(
            filePath,
            content
          );
          errors.push(...contentErrors);
        }
      }

      return {
        proceed: true,
        errors,
        metadata: {
          validatedAt: new Date().toISOString(),
          affectedFiles: result.affectedFiles || [],
          executionTime: result.executionTime,
        },
      };
    } catch (error) {
      console.error("[AssuraPlugin] Error in postToolUse:", error);
      return {
        proceed: true,
        errors: [
          {
            message: `Post-validation error: ${error instanceof Error ? error.message : "Unknown error"}`,
            severity: "Medium",
            filePath: "unknown",
            rule: "post-validation-error",
            autoFixable: false,
          },
        ],
      };
    }
  }

  /**
   * Error hook - handles errors from tool execution
   */
  public async onError(context: ToolCallContext, error: Error): Promise<void> {
    if (!this.config.hooks?.onError) {
      return;
    }

    console.error(`[AssuraPlugin] Error from ${context.toolName}:`, error.message);

    // Log error to state manager for tracking
    this.stateManager.setState(
      `error:${context.callId}`,
      {
        toolName: context.toolName,
        error: error.message,
        timestamp: new Date().toISOString(),
        agentId: context.agentId,
      },
      "assura-plugin"
    );
  }

  /**
   * Validate file operation (WriteFile, EditFile, Bash with file operations)
   */
  private async validateFileOperation(
    context: ToolCallContext
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];
    const filePath = context.args.filePath as string;

    if (!filePath) {
      return errors;
    }

    // Check for blocked paths
    const blockedPaths = [".env", ".env.local", ".env.production", ".git/config", ".ssh", ".aws", "secrets"];
    for (const blocked of blockedPaths) {
      if (filePath.includes(blocked)) {
        errors.push({
          message: `Access to "${blocked}" is restricted`,
          severity: "Critical",
          filePath,
          rule: "blocked-path",
          autoFixable: false,
        });
      }
    }

    // Check naming conventions
    const namingErrors = await this.validationEngine.validateFileName(filePath);
    errors.push(...namingErrors);

    // Check if file already exists (for write operations)
    if (context.toolName === "WriteFile") {
      const exists = await this.fileExists(filePath);
      if (exists) {
        // For edits, this is expected; for new files, check if it should be created
        const extension = filePath.split(".").pop() || "";
        const extErrors = await this.validationEngine.validateExtension(
          filePath,
          extension
        );
        errors.push(...extErrors);
      }
    }

    // Check directory structure
    const dirPath = filePath.substring(0, filePath.lastIndexOf("/"));
    if (dirPath) {
      const dirErrors = await this.validationEngine.validateDirectory(dirPath);
      errors.push(...dirErrors);
    }

    return errors;
  }

  /**
   * Validate read operation (Glob, Grep, Read)
   */
  private async validateReadOperation(
    context: ToolCallContext
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Read operations are generally safe, but we might want to restrict
    // access to certain sensitive paths
    const pattern = (context.args.pattern as string) || "";

    // Check for attempts to read sensitive files
    const sensitivePatterns = [
      ".env",
      ".env.local",
      ".env.production",
      "secrets",
      "credentials",
      ".ssh",
      ".aws",
    ];

    for (const sensitive of sensitivePatterns) {
      if (pattern.includes(sensitive)) {
        errors.push({
          message: `Attempting to access potentially sensitive path: ${sensitive}`,
          severity: "Medium",
          filePath: pattern,
          rule: "sensitive-path-access",
          autoFixable: false,
        });
      }
    }

    return errors;
  }

  /**
   * Check if operation should be blocked based on errors
   */
  private shouldBlockOperation(errors: ValidationError[]): boolean {
    if (!this.config.validation?.strict) {
      return false;
    }

    const threshold = this.config.validation.severityThreshold;
    const severityOrder: SeverityLevel[] = ["Critical", "High", "Medium", "Low"];
    const thresholdIndex = severityOrder.indexOf(threshold);

    for (const error of errors) {
      const errorIndex = severityOrder.indexOf(error.severity);
      if (errorIndex <= thresholdIndex) {
        return true;
      }
    }

    return false;
  }

  /**
   * Check if file exists
   */
  private async fileExists(filePath: string): Promise<boolean> {
    try {
      const { stat } = await import("node:fs/promises");
      await stat(filePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get agent registry
   */
  public getAgentRegistry() {
    return this.agentRegistry;
  }

  /**
   * Get state manager
   */
  public getStateManager() {
    return this.stateManager;
  }

  /**
   * Get protocol handler
   */
  public getProtocolHandler() {
    return this.protocolHandler;
  }
}

export default AssuraOpenCodePlugin;
