/**
 * OpenCode Plugin Example
 * 
 * Demonstrates how to use the Assura OpenCode plugin
 */

import { createPlugin } from "./src/index.js";

async function main() {
  console.log("=== Assura OpenCode Plugin Example ===\n");

  // Create plugin instance
  const plugin = createPlugin({
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

  // Initialize plugin
  await plugin.initialize();
  console.log("Plugin initialized successfully\n");

  // Example 1: Valid file creation
  console.log("Example 1: Valid file creation");
  const validResult = await plugin.preToolUse({
    callId: "call-1",
    toolName: "WriteFile",
    args: {
      filePath: "src/my-component.tsx",
      content: "export function MyComponent() { return <div>Hello</div>; }",
    },
    agentId: "agent-1",
    sessionId: "session-1",
    timestamp: new Date(),
  });

  console.log(`  Proceed: ${validResult.proceed}`);
  console.log(`  Errors: ${validResult.errors.length}\n`);

  // Example 2: Blocked file (.env)
  console.log("Example 2: Blocked file (.env)");
  const blockedResult = await plugin.preToolUse({
    callId: "call-2",
    toolName: "WriteFile",
    args: {
      filePath: ".env",
      content: "API_KEY=secret123",
    },
    agentId: "agent-1",
    sessionId: "session-1",
    timestamp: new Date(),
  });

  console.log(`  Proceed: ${blockedResult.proceed}`);
  console.log(`  Errors: ${blockedResult.errors.length}`);
  if (blockedResult.errors.length > 0) {
    console.log(`  First error: ${blockedResult.errors[0].message}`);
  }
  console.log();

  // Example 3: Invalid naming convention
  console.log("Example 3: Invalid naming convention");
  const namingResult = await plugin.preToolUse({
    callId: "call-3",
    toolName: "WriteFile",
    args: {
      filePath: "src/MyInvalidFile.ts",
      content: "export const x = 1;",
    },
    agentId: "agent-1",
    sessionId: "session-1",
    timestamp: new Date(),
  });

  console.log(`  Proceed: ${namingResult.proceed}`);
  console.log(`  Errors: ${namingResult.errors.length}`);
  namingResult.errors.forEach((err) => {
    console.log(`  - ${err.rule}: ${err.message}`);
  });
  console.log();

  // Example 4: Dangerous Bash command
  console.log("Example 4: Dangerous Bash command");
  const bashResult = await plugin.preToolUse({
    callId: "call-4",
    toolName: "Bash",
    args: {
      command: "rm -rf /important/data",
    },
    agentId: "agent-1",
    sessionId: "session-1",
    timestamp: new Date(),
  });

  console.log(`  Proceed: ${bashResult.proceed}`);
  console.log(`  Errors: ${bashResult.errors.length}`);
  if (bashResult.errors.length > 0) {
    console.log(`  First error: ${bashResult.errors[0].message}`);
  }
  console.log();

  // Example 5: Multi-agent coordination
  console.log("Example 5: Multi-agent coordination");
  
  // Register additional agents
  plugin.getAgentRegistry().registerAgent({
    id: "validator-agent",
    name: "Validator Agent",
    role: "validator",
    capabilities: ["file-validation", "syntax-check"],
    status: "idle",
    lastActivity: new Date(),
  });

  plugin.getAgentRegistry().registerAgent({
    id: "writer-agent",
    name: "Writer Agent",
    role: "worker",
    capabilities: ["file-write", "file-read"],
    status: "idle",
    lastActivity: new Date(),
  });

  console.log(`  Total agents: ${plugin.getAgentRegistry().getAgentCount()}`);
  
  const validators = plugin.getAgentRegistry().findAgentsByCapability("file-validation");
  console.log(`  Validators: ${validators.map((a) => a.name).join(", ")}`);
  console.log();

  // Example 6: State management
  console.log("Example 6: State management");
  plugin.getStateManager().setState("validation-count", 42, "agent-1");
  plugin.getStateManager().setState("last-error", null, "agent-1");
  
  const state = plugin.getStateManager().getState("validation-count");
  console.log(`  Validation count: ${state?.value}`);
  console.log(`  State owner: ${state?.owner}`);
  console.log(`  Total state keys: ${plugin.getStateManager().getStateSize()}`);
  console.log();

  // Shutdown plugin
  await plugin.shutdown();
  console.log("Plugin shutdown successfully");
}

main().catch(console.error);
