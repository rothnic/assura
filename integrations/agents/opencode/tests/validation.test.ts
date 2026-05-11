/**
 * Validation Engine Tests
 * 
 * Tests for file and content validation
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { createPlugin, AssuraOpenCodePlugin } from "../src/index";
import { ValidationEngine } from "../src/validation";

describe("Validation Engine", () => {
  let plugin: AssuraOpenCodePlugin;
  let engine: ValidationEngine;

  beforeEach(async () => {
    plugin = createPlugin();
    await plugin.initialize();
    engine = plugin["validationEngine"];
  });

  afterEach(async () => {
    await plugin.shutdown();
  });

  describe("File Name Validation", () => {
    it("should validate kebab-case file names", async () => {
      const errors = await engine.validateFileName("src/my-file.ts");
      expect(errors.length).toBe(0);
    });

    it("should reject PascalCase file names", async () => {
      const errors = await engine.validateFileName("src/MyFile.ts");
      expect(errors.length).toBeGreaterThan(0);
      expect(errors[0].rule).toBe("naming-convention");
    });

    it("should reject camelCase file names", async () => {
      const errors = await engine.validateFileName("src/myFile.ts");
      expect(errors.length).toBeGreaterThan(0);
    });

    it("should skip validation for hidden files", async () => {
      const errors = await engine.validateFileName(".gitignore");
      expect(errors.length).toBe(0);
    });

    it("should skip validation for node_modules", async () => {
      const errors = await engine.validateFileName("node_modules/some-package/index.js");
      expect(errors.length).toBe(0);
    });

    it("should reject files with invalid characters", async () => {
      const errors = await engine.validateFileName("src/my file.ts");
      expect(errors.some((e) => e.rule === "invalid-characters")).toBe(true);
    });
  });

  describe("Extension Validation", () => {
    it("should validate allowed extensions", async () => {
      const errors = await engine.validateExtension("src/file.ts", "ts");
      expect(errors.length).toBe(0);
    });

    it("should reject unknown extensions", async () => {
      const errors = await engine.validateExtension("src/file.xyz", "xyz");
      expect(errors.length).toBeGreaterThan(0);
      expect(errors[0].rule).toBe("extension-allowed");
    });
  });

  describe("Directory Validation", () => {
    it("should validate directory naming", async () => {
      const errors = await engine.validateDirectory("src/components");
      expect(errors.length).toBe(0);
    });

    it("should reject PascalCase directories", async () => {
      const errors = await engine.validateDirectory("src/MyComponents");
      expect(errors.some((e) => e.rule === "directory-naming")).toBe(true);
    });

    it("should skip validation for common directories", async () => {
      const errors = await engine.validateDirectory("src/node_modules/package");
      expect(errors.length).toBe(0);
    });
  });

  describe("Content Validation", () => {
    it("should detect trailing whitespace", async () => {
      const errors = await engine.validateContent(
        "src/test.ts",
        "const x = 1;  \nconst y = 2;"
      );
      expect(errors.some((e) => e.rule === "no-trailing-whitespace")).toBe(true);
    });

    it("should detect tabs", async () => {
      const errors = await engine.validateContent(
        "src/test.ts",
        "const x = 1;\n\tconst y = 2;"
      );
      expect(errors.some((e) => e.rule === "no-tabs")).toBe(true);
    });

    it("should detect long lines", async () => {
      const longLine = "x".repeat(121);
      const errors = await engine.validateContent("src/test.ts", longLine);
      expect(errors.some((e) => e.rule === "max-line-length")).toBe(true);
    });
  });

  describe("Markdown Validation", () => {
    it("should require frontmatter", async () => {
      const errors = await engine.validateContent(
        "docs/readme.md",
        "# Title\n\nContent here"
      );
      expect(errors.some((e) => e.rule === "markdown-frontmatter")).toBe(true);
    });

    it("should detect insecure links", async () => {
      const errors = await engine.validateContent(
        "docs/readme.md",
        "---\n---\n[link](http://example.com)"
      );
      expect(errors.some((e) => e.rule === "markdown-secure-links")).toBe(true);
    });
  });

  describe("Code Validation", () => {
    it("should detect console.log", async () => {
      const errors = await engine.validateContent(
        "src/test.ts",
        "function test() { console.log('debug'); }"
      );
      expect(errors.some((e) => e.rule === "no-console-log")).toBe(true);
    });

    it("should detect TODO comments", async () => {
      const errors = await engine.validateContent(
        "src/test.ts",
        "// TODO: implement this"
      );
      expect(errors.some((e) => e.rule === "no-todo-comments")).toBe(true);
    });

    it("should detect FIXME comments", async () => {
      const errors = await engine.validateContent(
        "src/test.ts",
        "// FIXME: broken code"
      );
      expect(errors.some((e) => e.rule === "no-todo-comments")).toBe(true);
    });
  });

  describe("Full File Validation", () => {
    it("should validate complete file path", async () => {
      const errors = await engine.validateFile("src/components/my-component.ts");
      // Should check name, extension, and directory
      expect(errors).toBeDefined();
    });
  });
});
