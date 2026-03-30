/**
 * Hook Handler Tests
 * 
 * Tests for preToolUse and postToolUse hooks
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { createPlugin, AssuraOpenCodePlugin, type ToolCallContext } from "../src/index";
import { HookHandlers } from "../src/hooks";
import { ValidationEngine } from "../src/validation";

describe("Hook Handlers", () => {
  let plugin: AssuraOpenCodePlugin;
  let handlers: HookHandlers;

  beforeEach(async () => {
    plugin = createPlugin();
    await plugin.initialize();
    handlers = new HookHandlers(plugin["validationEngine"]);
  });

  afterEach(async () => {
    await plugin.shutdown();
  });

  const createContext = (overrides: Partial<ToolCallContext> = {}): ToolCallContext => ({
    callId: "test-call-1",
    toolName: "WriteFile",
    args: {
      filePath: "src/test-file.ts",
      content: "console.log('test');",
    },
    agentId: "test-agent",
    sessionId: "test-session",
    timestamp: new Date(),
    ...overrides,
  });

  describe("preToolUse hook", () => {
    it("should allow valid WriteFile operation", async () => {
      const context = createContext();
      const result = await handlers.handlePreToolUse(context);

      expect(result.proceed).toBe(true);
    });

    it("should block WriteFile to blocked path", async () => {
      const context = createContext({
        args: {
          filePath: ".env",
          content: "SECRET=key",
        },
      });

      const result = await handlers.handlePreToolUse(context);

      expect(result.proceed).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].rule).toBe("blocked-path");
    });

    it("should validate file naming convention", async () => {
      const context = createContext({
        args: {
          filePath: "src/InvalidFileName.ts",
          content: "test",
        },
      });

      const result = await handlers.handlePreToolUse(context);

      // Should have naming convention errors but not block in non-strict mode
      expect(result.errors.some((e) => e.rule === "naming-convention")).toBe(
        true
      );
    });

    it("should block dangerous Bash commands", async () => {
      const context = createContext({
        toolName: "Bash",
        args: {
          command: "rm -rf /",
        },
      });

      const result = await handlers.handlePreToolUse(context);

      expect(result.proceed).toBe(false);
      expect(result.errors.some((e) => e.rule === "dangerous-command")).toBe(
        true
      );
    });

    it("should block missing arguments", async () => {
      const context = createContext({
        args: {},
      });

      const result = await handlers.handlePreToolUse(context);

      expect(result.proceed).toBe(false);
      expect(result.errors[0].rule).toBe("missing-argument");
    });

    it("should handle EditFile operations", async () => {
      const context = createContext({
        toolName: "EditFile",
        args: {
          filePath: "src/existing-file.ts",
          oldString: "old",
          newString: "new",
        },
      });

      const result = await handlers.handlePreToolUse(context);

      expect(result.proceed).toBe(true);
    });

    it("should validate content for code files", async () => {
      const context = createContext({
        args: {
          filePath: "src/test.ts",
          content: "const x = 1;\n// TODO: fix this\nconsole.log(x);",
        },
      });

      const result = await handlers.handlePreToolUse(context);

      expect(
        result.errors.some((e) => e.rule === "no-todo-comments")
      ).toBe(true);
      expect(result.errors.some((e) => e.rule === "no-console-log")).toBe(true);
    });
  });

  describe("postToolUse hook", () => {
    it("should validate affected files", async () => {
      const context = createContext();
      const result = {
        success: true,
        affectedFiles: ["src/test-file.ts"],
        executionTime: 100,
      };

      const hookResult = await handlers.handlePostToolUse(context, result);

      expect(hookResult.proceed).toBe(true);
      expect(hookResult.metadata?.validatedFiles).toContain("src/test-file.ts");
    });

    it("should skip validation on failed tool execution", async () => {
      const context = createContext();
      const result = {
        success: false,
        error: "File not found",
        executionTime: 50,
      };

      const hookResult = await handlers.handlePostToolUse(context, result);

      expect(hookResult.proceed).toBe(true);
      expect(hookResult.errors.length).toBe(0);
    });
  });
});
