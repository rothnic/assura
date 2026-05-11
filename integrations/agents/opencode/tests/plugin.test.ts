/**
 * Plugin Factory Tests
 * 
 * Tests for plugin creation and initialization
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import {
  createPlugin,
  AssuraOpenCodePlugin,
  type PluginConfig,
} from "../src/index";

describe("Plugin Factory", () => {
  let plugin: AssuraOpenCodePlugin;

  afterEach(async () => {
    if (plugin) {
      await plugin.shutdown();
    }
  });

  it("should create plugin with default config", () => {
    plugin = createPlugin();
    expect(plugin).toBeDefined();
    expect(plugin.config.name).toBe("assura-opencode-plugin");
    expect(plugin.config.version).toBe("0.1.0");
  });

  it("should create plugin with custom config", () => {
    const customConfig: Partial<PluginConfig> = {
      name: "custom-plugin",
      version: "1.0.0",
      validation: {
        strict: false,
        severityThreshold: "Medium",
        autoFix: true,
      },
    };

    plugin = createPlugin(customConfig);
    expect(plugin.config.name).toBe("custom-plugin");
    expect(plugin.config.version).toBe("1.0.0");
    expect(plugin.config.validation?.strict).toBe(false);
    expect(plugin.config.validation?.severityThreshold).toBe("Medium");
    expect(plugin.config.validation?.autoFix).toBe(true);
  });

  it("should merge partial config with defaults", () => {
    plugin = createPlugin({
      validation: {
        strict: false,
        severityThreshold: "Critical",
        autoFix: false,
      },
    });

    // Custom values
    expect(plugin.config.validation?.strict).toBe(false);
    expect(plugin.config.validation?.severityThreshold).toBe("Critical");

    // Default values still present
    expect(plugin.config.name).toBe("assura-opencode-plugin");
    expect(plugin.config.hooks?.preToolUse).toBe(true);
  });

  it("should throw error for invalid config", () => {
    expect(() => {
      createPlugin({
        validation: {
          severityThreshold: "Invalid" as any,
          strict: true,
          autoFix: false,
        },
      });
    }).toThrow();
  });

  it("should initialize successfully", async () => {
    plugin = createPlugin();
    await plugin.initialize();
    expect(plugin.getAgentRegistry().getAgentCount()).toBe(1);
  });

  it("should not initialize twice", async () => {
    plugin = createPlugin();
    await plugin.initialize();
    expect(plugin.initialize()).rejects.toThrow("Plugin already initialized");
  });

  it("should shutdown successfully", async () => {
    plugin = createPlugin();
    await plugin.initialize();
    await plugin.shutdown();
    expect(plugin.getAgentRegistry().getAgentCount()).toBe(0);
  });

  it("should handle shutdown when not initialized", async () => {
    plugin = createPlugin();
    await expect(plugin.shutdown()).resolves.toBeUndefined();
  });
});
