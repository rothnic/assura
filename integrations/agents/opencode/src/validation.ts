/**
 * Validation Engine
 * 
 * Core validation logic that interfaces with Assura constraint system.
 */

import type {
  ValidationError,
  SeverityLevel,
} from "./types";

/**
 * Validation configuration
 */
interface ValidationConfig {
  strict: boolean;
  severityThreshold: SeverityLevel;
  autoFix: boolean;
}

/**
 * Case convention types
 */
type CaseConvention =
  | "kebab-case"
  | "snake_case"
  | "camelCase"
  | "PascalCase"
  | "SCREAMING_SNAKE_CASE"
  | "COBOL-CASE"
  | "Train-Case"
  | "flatcase"
  | "FLATCASE";

/**
 * Validation engine implementation
 */
export class ValidationEngine {
  private config: ValidationConfig;
  private initialized = false;

  constructor(config?: Partial<ValidationConfig>) {
    this.config = {
      strict: true,
      severityThreshold: "High",
      autoFix: false,
      ...config,
    };
  }

  /**
   * Initialize the validation engine
   */
  public async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    console.log("[ValidationEngine] Initializing...");
    
    // In a real implementation, this would load Assura configuration
    // and initialize the Rust core via FFI or CLI calls
    
    this.initialized = true;
    console.log("[ValidationEngine] Initialized");
  }

  /**
   * Shutdown the validation engine
   */
  public async shutdown(): Promise<void> {
    if (!this.initialized) {
      return;
    }

    console.log("[ValidationEngine] Shutting down...");
    this.initialized = false;
  }

  /**
   * Validate a file by path
   */
  public async validateFile(filePath: string): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Validate file name
    const nameErrors = await this.validateFileName(filePath);
    errors.push(...nameErrors);

    // Validate extension
    const parts = filePath.split(".");
    if (parts.length > 1) {
      const extension = parts[parts.length - 1];
      const extErrors = await this.validateExtension(filePath, extension);
      errors.push(...extErrors);
    }

    return errors;
  }

  /**
   * Validate file content
   */
  public async validateContent(
    filePath: string,
    content: string
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Check for common issues in content
    const lines = content.split("\n");

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const lineNum = i + 1;

      // Check for trailing whitespace
      if (line.match(/\s+$/)) {
        errors.push({
          message: "Line has trailing whitespace",
          severity: "Low",
          filePath,
          rule: "no-trailing-whitespace",
          line: lineNum,
          autoFixable: true,
          suggestion: line.trimEnd(),
        });
      }

      // Check for tabs (should use spaces)
      if (line.includes("\t")) {
        errors.push({
          message: "Line contains tabs, should use spaces",
          severity: "Low",
          filePath,
          rule: "no-tabs",
          line: lineNum,
          autoFixable: true,
          suggestion: line.replace(/\t/g, "  "),
        });
      }

      // Check line length
      if (line.length > 120) {
        errors.push({
          message: `Line exceeds 120 characters (${line.length})`,
          severity: "Medium",
          filePath,
          rule: "max-line-length",
          line: lineNum,
          autoFixable: false,
        });
      }
    }

    // Validate based on file type
    if (filePath.endsWith(".md")) {
      const mdErrors = await this.validateMarkdown(filePath, content);
      errors.push(...mdErrors);
    } else if (filePath.endsWith(".ts") || filePath.endsWith(".js")) {
      const codeErrors = await this.validateCode(filePath, content);
      errors.push(...codeErrors);
    }

    return errors;
  }

  /**
   * Validate file name against naming conventions
   */
  public async validateFileName(filePath: string): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];
    const fileName = filePath.split("/").pop() || "";

    // Skip validation for hidden files and special directories
    if (fileName.startsWith(".") || filePath.includes("node_modules/")) {
      return errors;
    }

    // Get base name without extension
    const baseName = fileName.split(".")[0];

    // Check against kebab-case (default convention)
    if (!this.matchesConvention(baseName, "kebab-case")) {
      errors.push({
        message: `File name "${baseName}" does not match required naming convention (kebab-case)`,
        severity: "Medium",
        filePath,
        rule: "naming-convention",
        autoFixable: true,
        suggestion: this.convertToConvention(baseName, "kebab-case"),
      });
    }

    // Check for invalid characters
    if (fileName.match(/[^a-zA-Z0-9._\-]/)) {
      errors.push({
        message: `File name contains invalid characters`,
        severity: "High",
        filePath,
        rule: "invalid-characters",
        autoFixable: false,
      });
    }

    return errors;
  }

  /**
   * Validate file extension
   */
  public async validateExtension(
    filePath: string,
    extension: string
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // List of allowed extensions
    const allowedExtensions = [
      "ts", "tsx", "js", "jsx", "json", "md", "yml", "yaml",
      "css", "scss", "html", "svg", "txt", "rs", "toml",
    ];

    if (!allowedExtensions.includes(extension)) {
      errors.push({
        message: `Extension ".${extension}" is not in allowed list`,
        severity: "Medium",
        filePath,
        rule: "extension-allowed",
        autoFixable: false,
      });
    }

    return errors;
  }

  /**
   * Validate directory structure
   */
  public async validateDirectory(dirPath: string): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Skip validation for common directories
    const skipDirs = ["node_modules", ".git", "target", "dist", "build", ".cache"];
    const parts = dirPath.split("/");

    for (const part of parts) {
      if (skipDirs.includes(part)) {
        return errors;
      }
    }

    // Validate directory naming
    for (const part of parts) {
      if (!part) continue;
      
      if (!this.matchesConvention(part, "kebab-case")) {
        errors.push({
          message: `Directory "${part}" does not match naming convention (kebab-case)`,
          severity: "Low",
          filePath: dirPath,
          rule: "directory-naming",
          autoFixable: true,
          suggestion: this.convertToConvention(part, "kebab-case"),
        });
      }
    }

    return errors;
  }

  /**
   * Validate markdown content
   */
  private async validateMarkdown(
    filePath: string,
    content: string
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Check for required frontmatter
    if (!content.startsWith("---")) {
      errors.push({
        message: "Markdown file missing frontmatter",
        severity: "Low",
        filePath,
        rule: "markdown-frontmatter",
        autoFixable: false,
      });
    }

    // Check for broken links (simplified check)
    const linkPattern = /\[([^\]]+)\]\(([^)]+)\)/g;
    let match;
    while ((match = linkPattern.exec(content)) !== null) {
      const link = match[2];
      if (link.startsWith("http") && !link.startsWith("https://")) {
        errors.push({
          message: `Insecure link: ${link}`,
          severity: "Low",
          filePath,
          rule: "markdown-secure-links",
          autoFixable: true,
          suggestion: link.replace("http://", "https://"),
        });
      }
    }

    return errors;
  }

  /**
   * Validate code content
   */
  private async validateCode(
    filePath: string,
    content: string
  ): Promise<ValidationError[]> {
    const errors: ValidationError[] = [];

    // Check for console.log statements
    if (content.match(/console\.log\s*\(/)) {
      errors.push({
        message: "File contains console.log statements",
        severity: "Low",
        filePath,
        rule: "no-console-log",
        autoFixable: false,
      });
    }

    // Check for TODO comments
    if (content.match(/TODO|FIXME|XXX/)) {
      errors.push({
        message: "File contains TODO/FIXME comments",
        severity: "Low",
        filePath,
        rule: "no-todo-comments",
        autoFixable: false,
      });
    }

    return errors;
  }

  /**
   * Check if string matches naming convention
   */
  private matchesConvention(str: string, convention: CaseConvention): boolean {
    const patterns: Record<CaseConvention, RegExp> = {
      "kebab-case": /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/,
      "snake_case": /^[a-z][a-z0-9]*(_[a-z0-9]+)*$/,
      "camelCase": /^[a-z][a-zA-Z0-9]*$/,
      "PascalCase": /^[A-Z][a-zA-Z0-9]*$/,
      "SCREAMING_SNAKE_CASE": /^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$/,
      "COBOL-CASE": /^[A-Z][A-Z0-9]*(-[A-Z0-9]+)*$/,
      "Train-Case": /^[A-Z][a-z0-9]*(-[A-Z][a-z0-9]*)*$/,
      "flatcase": /^[a-z][a-z0-9]*$/,
      "FLATCASE": /^[A-Z][A-Z0-9]*$/,
    };

    return patterns[convention]?.test(str) ?? true;
  }

  /**
   * Convert string to naming convention
   */
  private convertToConvention(str: string, convention: CaseConvention): string {
    // Split into words
    const words = str
      .replace(/([A-Z])/g, "-$1")
      .replace(/_/g, "-")
      .toLowerCase()
      .split("-")
      .filter(Boolean);

    switch (convention) {
      case "kebab-case":
        return words.join("-");
      case "snake_case":
        return words.join("_");
      case "camelCase":
        return (
          words[0] +
          words
            .slice(1)
            .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
            .join("")
        );
      case "PascalCase":
        return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join("");
      case "SCREAMING_SNAKE_CASE":
        return words.join("_").toUpperCase();
      case "COBOL-CASE":
        return words.join("-").toUpperCase();
      case "Train-Case":
        return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join("-");
      case "flatcase":
        return words.join("");
      case "FLATCASE":
        return words.join("").toUpperCase();
      default:
        return str;
    }
  }
}

export default ValidationEngine;
