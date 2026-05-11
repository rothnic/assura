/**
 * Integration Tests
 * 
 * End-to-end tests for the complete plugin workflow
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { createPlugin, AssuraOpenCodePlugin, type ToolCallContext, type ToolCallResult } from "../src/index";

describe("Integration Tests", () => {
  let plugin: AssuraOpenCodePlugin;

  beforeEach(async () => {
    plugin = createPlugin({
      validation: {
        strict: true,
        severityThreshold: "High",
        autoFix: false,
      },
      agent: {
        multiAgent: true,
        coordinationMode: "peer",
      },
    });
    await plugin.initialize();
  });

  afterEach(async () => {
    await plugin.shutdown();
  });

  const createContext = (overrides: Partial<ToolCallContext> = {}): ToolCallContext => ({
    callId: `call-${Date.now()}`,
    toolName: "WriteFile",
    args: {},
    agentId: "test-agent",
    sessionId: "test-session",
    timestamp: new Date(),
    ...overrides,
  });

  describe("Full Workflow", () => {
    it("should validate file creation workflow", async () => {
      const context = createContext({
        toolName: "WriteFile",
        args: {
          filePath: "src/new-file.ts",
          content: "export const x = 1;",
        },
      });

      // Pre-tool validation
      const preResult = await plugin.preToolUse(context);
      expect(preResult.proceed).toBe(true);

      // Simulate tool execution
      const toolResult: ToolCallResult = {
        success: true,
        affectedFiles: ["src/new-file.ts"],
        executionTime: 100,
      };

      // Post-tool validation
      const postResult = await plugin.postToolUse(context, toolResult);
      expect(postResult.proceed).toBe(true);
    });

    it("should block invalid file creation", async () => {
      const context = createContext({
        toolName: "WriteFile",
        args: {
          filePath: ".env",
          content: "SECRET=value",
        },
      });

      const result = await plugin.preToolUse(context);
      expect(result.proceed).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it("should handle multi-agent coordination", async () => {
      // Register multiple agents
      plugin.getAgentRegistry().registerAgent({
        id: "agent-1",
        name: "Agent 1",
        role: "leader",
        capabilities: ["validation"],
        status: "idle",
        lastActivity: new Date(),
      });

      plugin.getAgentRegistry().registerAgent({
        id: "agent-2",
        name: "Agent 2",
        role: "worker",
        capabilities: ["file-ops"],
        status: "idle",
        lastActivity: new Date(),
      });

      expect(plugin.getAgentRegistry().getAgentCount()).toBe(3); // Including assura-plugin

      // Coordinator agent requests coordination
      const coordinateMessage = {
        id: "coord-1",
        type: "COORDINATE" as const,
        sender: "agent-1",
        payload: {
          operation: "batch-validation",
          agents: ["agent-2"],
        },
        timestamp: new Date(),
      };

      const response = await plugin.getProtocolHandler().handleMessage(coordinateMessage);
      expect(response?.payload.success).toBe(true);
    });

    it("should maintain state across operations", async () => {
      // Set state from one operation
      plugin.getStateManager().setState("validation-count", 1, "agent-1");

      // Increment in another operation
      const current = plugin.getStateManager().getState("validation-count");
      plugin.getStateManager().setState(
        "validation-count",
        (current?.value as number) + 1,
        "agent-2"
      );

      const final = plugin.getStateManager().getState("validation-count");
      expect(final?.value).toBe(2);
    });

    it("should detect and report violations in post-validation", async () => {
      const context = createContext({
        toolName: "WriteFile",
        args: {
          filePath: "src/test.ts",
          content: "console.log('debug'); // TODO: remove",
        },
      });

      const preResult = await plugin.preToolUse(context);
      expect(preResult.proceed).toBe(true); // Allowed to proceed

      const toolResult: ToolCallResult = {
        success: true,
        affectedFiles: ["src/test.ts"],
        executionTime: 50,
      };

      const postResult = await plugin.postToolUse(context, toolResult);
      expect(postResult.errors.length).toBeGreaterThan(0);
      expect(postResult.errors.some((e) => e.rule === "no-console-log")).toBe(true);
      expect(postResult.errors.some((e) => e.rule === "no-todo-comments")).toBe(true);
    });
  });

  describe("Error Handling", () => {
    it("should handle errors gracefully", async () => {
      const context = createContext({
        toolName: "WriteFile",
        args: {
          filePath: "src/test.ts",
          content: "test",
        },
      });

      const error = new Error("Test error");
      await expect(plugin.onError(context, error)).resolves.toBeUndefined();

      // Verify error was logged to state
      const keys = plugin.getStateManager().listKeys();
      expect(keys.some((k) => k.startsWith("error:"))).toBe(true);
    });

    it("should recover from validation errors", async () => {
      const context = createContext({
        toolName: "UnknownTool",
        args: {},
      });

      const result = await plugin.preToolUse(context);
      expect(result.proceed).toBe(true); // Unknown tools are allowed
    });
  });

  describe("Complex Scenarios", () => {
    it("should handle batch file operations", async () => {
      const files = [
        "src/file-a.ts",
        "src/file-b.ts",
        "src/file-c.ts",
      ];

      for (const filePath of files) {
        const context = createContext({
          toolName: "WriteFile",
          args: {
            filePath,
            content: "export {};",
          },
        });

        const result = await plugin.preToolUse(context);
        expect(result.proceed).toBe(true);
      }
    });

    it("should validate directory structure before file creation", async () => {
      const context = createContext({
        toolName: "WriteFile",
        args: {
          filePath: "src/InvalidDir/file.ts",
          content: "test",
        },
      });

      const result = await plugin.preToolUse(context);
      // Directory name violates convention
      expect(result.errors.some((e) => e.rule === "directory-naming")).toBe(true);
    });

    it("should coordinate validation across agents", async () => {
      // Setup agents
      plugin.getAgentRegistry().registerAgent({
        id: "validator-agent",
        name: "Validator",
        role: "validator",
        capabilities: ["file-validation"],
        status: "idle",
        lastActivity: new Date(),
      });

      // Request validation from validator agent
      const message = {
        id: "val-req-1",
        type: "VALIDATE" as const,
        sender: "worker-agent",
        payload: {
          filePath: "src/component.tsx",
        },
        timestamp: new Date(),
      };

      const response = await plugin.getProtocolHandler().handleMessage(message);
      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(response?.payload).toHaveProperty("valid");
      expect(response?.payload).toHaveProperty("errors");
    });
  });
});
