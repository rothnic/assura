/**
 * Hook Handlers
 * 
 * Specific implementations for preToolUse and postToolUse hooks
 * with detailed validation logic.
 */

import type {
  ToolCallContext,
  ToolCallResult,
  HookResult,
  ValidationError,
  SeverityLevel,
} from "./types";
import { ValidationEngine } from "./validation";

/**
 * Hook handlers configuration
 */
interface HookConfig {
  strict: boolean;
  severityThreshold: SeverityLevel;
  blockedTools: string[];
  blockedPaths: string[];
  allowedExtensions: string[];
}

/**
 * Hook handlers implementation
 */
export class HookHandlers {
  private validationEngine: ValidationEngine;
  private config: HookConfig;

  constructor(validationEngine: ValidationEngine, config?: Partial<HookConfig>) {
    this.validationEngine = validationEngine;
    this.config = {
      strict: true,
      severityThreshold: "High",
      blockedTools: [],
      blockedPaths: [".env", ".git", "node_modules/.bin"],
      allowedExtensions: ["ts", "tsx", "js", "jsx", "json", "md", "css", "html"],
      ...config,
    };
  }

  /**
   * Handle preToolUse hook
   */
  public async handlePreToolUse(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];

    // Check if tool is blocked
    if (this.config.blockedTools.includes(context.toolName)) {
      errors.push({
        message: `Tool "${context.toolName}" is blocked by policy`,
        severity: "Critical",
        filePath: "",
        rule: "blocked-tool",
        autoFixable: false,
      });

      return {
        proceed: false,
        errors,
      };
    }

    // Handle specific tool types
    switch (context.toolName) {
      case "WriteFile":
        return this.handleWriteFile(context);
      case "EditFile":
        return this.handleEditFile(context);
      case "Bash":
        return this.handleBash(context);
      case "ReadFile":
        return this.handleReadFile(context);
      case "Glob":
        return this.handleGlob(context);
      default:
        // Unknown tool - allow but log
        console.log(`[HookHandlers] Unknown tool: ${context.toolName}`);
        return { proceed: true, errors: [] };
    }
  }

  /**
   * Handle postToolUse hook
   */
  public async handlePostToolUse(
    context: ToolCallContext,
    result: ToolCallResult
  ): Promise<HookResult> {
    const errors: ValidationError[] = [];

    // If the tool failed, don't validate results
    if (!result.success) {
      return { proceed: true, errors };
    }

    // Validate affected files
    if (result.affectedFiles) {
      for (const filePath of result.affectedFiles) {
        const fileErrors = await this.validationEngine.validateFile(filePath);
        errors.push(...fileErrors);
      }
    }

    // Tool-specific post-validation
    switch (context.toolName) {
      case "WriteFile":
      case "EditFile": {
        const filePath = context.args.filePath as string;
        const content = context.args.content as string;
        if (filePath && content) {
          const contentErrors = await this.validationEngine.validateContent(
            filePath,
            content
          );
          errors.push(...contentErrors);
        }
        break;
      }
    }

    return {
      proceed: true,
      errors,
      metadata: {
        validatedFiles: result.affectedFiles || [],
        errorCount: errors.length,
      },
    };
  }

  /**
   * Handle WriteFile tool
   */
  private async handleWriteFile(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];
    const filePath = context.args.filePath as string;
    const content = context.args.content as string;

    if (!filePath) {
      errors.push({
        message: "WriteFile missing filePath argument",
        severity: "Critical",
        filePath: "",
        rule: "missing-argument",
        autoFixable: false,
      });
      return { proceed: false, errors };
    }

    // Check blocked paths
    const pathError = this.checkBlockedPath(filePath);
    if (pathError) {
      errors.push(pathError);
    }

    // Validate file name
    const nameErrors = await this.validationEngine.validateFileName(filePath);
    errors.push(...nameErrors);

    // Validate directory structure
    const dirPath = filePath.substring(0, filePath.lastIndexOf("/"));
    if (dirPath) {
      const dirErrors = await this.validationEngine.validateDirectory(dirPath);
      errors.push(...dirErrors);
    }

    // Validate content if provided
    if (content) {
      const contentErrors = await this.validationEngine.validateContent(
        filePath,
        content
      );
      errors.push(...contentErrors);
    }

    // Check if should block
    const shouldBlock = this.shouldBlock(errors);

    return {
      proceed: !shouldBlock,
      errors,
      metadata: {
        operation: "write",
        filePath,
        preValidated: true,
      },
    };
  }

  /**
   * Handle EditFile tool
   */
  private async handleEditFile(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];
    const filePath = context.args.filePath as string;

    if (!filePath) {
      errors.push({
        message: "EditFile missing filePath argument",
        severity: "Critical",
        filePath: "",
        rule: "missing-argument",
        autoFixable: false,
      });
      return { proceed: false, errors };
    }

    // Check blocked paths
    const pathError = this.checkBlockedPath(filePath);
    if (pathError) {
      errors.push(pathError);
    }

    // For edits, we can be less strict about naming since file already exists
    // But we should still validate the new content
    const oldString = context.args.oldString as string;
    const newString = context.args.newString as string;

    if (newString) {
      // Create temporary content to validate
      const tempContent = oldString
        ? `PLACEHOLDER\n${newString}\nPLACEHOLDER`
        : newString;
      
      const contentErrors = await this.validationEngine.validateContent(
        filePath,
        tempContent
      );
      errors.push(...contentErrors);
    }

    const shouldBlock = this.shouldBlock(errors);

    return {
      proceed: !shouldBlock,
      errors,
      metadata: {
        operation: "edit",
        filePath,
      },
    };
  }

  /**
   * Handle Bash tool
   */
  private async handleBash(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];
    const command = context.args.command as string;

    if (!command) {
      errors.push({
        message: "Bash missing command argument",
        severity: "Critical",
        filePath: "",
        rule: "missing-argument",
        autoFixable: false,
      });
      return { proceed: false, errors };
    }

    // Check for dangerous commands
    const dangerousPatterns = [
      { pattern: /rm\s+-rf\s+\//, desc: "Recursive root deletion" },
      { pattern: />\s*\/dev\/null.*\bfb\b/, desc: "Fork bomb" },
      { pattern: /mkfs\.\w+\s+/, desc: "Filesystem formatting" },
      { pattern: /dd\s+if=.*of=\/(dev|disk)/, desc: "Direct disk write" },
      { pattern: /chmod\s+-R\s+777\s+\//, desc: "Insecure permissions" },
    ];

    for (const { pattern, desc } of dangerousPatterns) {
      if (pattern.test(command)) {
        errors.push({
          message: `Dangerous command detected: ${desc}`,
          severity: "Critical",
          filePath: "",
          rule: "dangerous-command",
          autoFixable: false,
        });
      }
    }

    // Check for file operations that might violate constraints
    const fileOpMatch = command.match(/(?:cat|echo|tee)\s+["']?([^"'\s]+)/);
    if (fileOpMatch) {
      const targetFile = fileOpMatch[1];
      const pathError = this.checkBlockedPath(targetFile);
      if (pathError) {
        errors.push(pathError);
      }
    }

    const shouldBlock = this.shouldBlock(errors);

    return {
      proceed: !shouldBlock,
      errors,
      metadata: {
        operation: "bash",
        command: command.substring(0, 50) + (command.length > 50 ? "..." : ""),
      },
    };
  }

  /**
   * Handle ReadFile tool
   */
  private async handleReadFile(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];
    const filePath = context.args.filePath as string;

    if (!filePath) {
      errors.push({
        message: "ReadFile missing filePath argument",
        severity: "Critical",
        filePath: "",
        rule: "missing-argument",
        autoFixable: false,
      });
      return { proceed: false, errors };
    }

    // Check blocked paths for sensitive files
    const pathError = this.checkBlockedPath(filePath, true);
    if (pathError) {
      errors.push(pathError);
    }

    return {
      proceed: errors.length === 0,
      errors,
    };
  }

  /**
   * Handle Glob tool
   */
  private async handleGlob(context: ToolCallContext): Promise<HookResult> {
    const errors: ValidationError[] = [];
    const pattern = context.args.pattern as string;

    if (!pattern) {
      errors.push({
        message: "Glob missing pattern argument",
        severity: "Critical",
        filePath: "",
        rule: "missing-argument",
        autoFixable: false,
      });
      return { proceed: false, errors };
    }

    // Check for overly broad patterns that might hit sensitive files
    if (pattern.includes("**") && !pattern.includes("/")) {
      errors.push({
        message: "Glob pattern might match sensitive files. Use more specific patterns.",
        severity: "Medium",
        filePath: pattern,
        rule: "broad-glob-pattern",
        autoFixable: false,
      });
    }

    return {
      proceed: true,
      errors,
    };
  }

  /**
   * Check if path is blocked
   */
  private checkBlockedPath(
    filePath: string,
    sensitiveOnly = false
  ): ValidationError | null {
    const sensitivePaths = [".env", ".env.local", ".git/config", ".ssh", ".aws"];
    const blockedPaths = sensitiveOnly
      ? sensitivePaths
      : [...this.config.blockedPaths, ...sensitivePaths];

    for (const blocked of blockedPaths) {
      if (filePath.includes(blocked)) {
        return {
          message: `Access to "${blocked}" is restricted`,
          severity: sensitiveOnly ? "High" : "Critical",
          filePath,
          rule: "blocked-path",
          autoFixable: false,
        };
      }
    }

    return null;
  }

  /**
   * Determine if operation should be blocked based on errors
   */
  private shouldBlock(errors: ValidationError[]): boolean {
    if (!this.config.strict) {
      return false;
    }

    const severityOrder: SeverityLevel[] = ["Critical", "High", "Medium", "Low"];
    const thresholdIndex = severityOrder.indexOf(this.config.severityThreshold);

    for (const error of errors) {
      const errorIndex = severityOrder.indexOf(error.severity);
      if (errorIndex <= thresholdIndex) {
        return true;
      }
    }

    return false;
  }

  /**
   * Update configuration
   */
  public updateConfig(config: Partial<HookConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  public getConfig(): HookConfig {
    return { ...this.config };
  }
}

export default HookHandlers;
