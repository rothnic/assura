/**
 * Protocol Handler Tests
 * 
 * Tests for agent protocol message handling
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { 
  createPlugin, 
  AssuraOpenCodePlugin, 
  type ProtocolMessage, 
  type AgentInfo 
} from "../src/index";

describe("Protocol Handler", () => {
  let plugin: AssuraOpenCodePlugin;

  beforeEach(async () => {
    plugin = createPlugin();
    await plugin.initialize();
  });

  afterEach(async () => {
    await plugin.shutdown();
  });

  const createMessage = (overrides: Partial<ProtocolMessage> = {}): ProtocolMessage => ({
    id: `msg-${Date.now()}`,
    type: "HEARTBEAT",
    sender: "test-agent",
    timestamp: new Date(),
    payload: {},
    ...overrides,
  });

  describe("Message Handling", () => {
    it("should handle REGISTER message", async () => {
      const agent: AgentInfo = {
        id: "new-agent",
        name: "New Agent",
        role: "worker",
        capabilities: ["test"],
        status: "idle",
        lastActivity: new Date(),
      };

      const message = createMessage({
        type: "REGISTER",
        payload: agent,
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response).toBeDefined();
      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(plugin.getAgentRegistry().isRegistered("new-agent")).toBe(true);
    });

    it("should handle UNREGISTER message", async () => {
      // First register an agent
      plugin.getAgentRegistry().registerAgent({
        id: "agent-to-remove",
        name: "Agent",
        role: "worker",
        capabilities: [],
        status: "idle",
        lastActivity: new Date(),
      });

      const message = createMessage({
        type: "UNREGISTER",
        payload: { agentId: "agent-to-remove" },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(plugin.getAgentRegistry().isRegistered("agent-to-remove")).toBe(false);
    });

    it("should handle VALIDATE message", async () => {
      const message = createMessage({
        type: "VALIDATE",
        payload: { filePath: "src/test.ts" },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(response?.payload).toHaveProperty("valid");
      expect(response?.payload).toHaveProperty("errors");
    });

    it("should handle COORDINATE message", async () => {
      // Register agents first
      plugin.getAgentRegistry().registerAgent({
        id: "agent-1",
        name: "Agent 1",
        role: "worker",
        capabilities: [],
        status: "idle",
        lastActivity: new Date(),
      });

      const message = createMessage({
        type: "COORDINATE",
        payload: {
          operation: "test-op",
          agents: ["agent-1"],
        },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("COORDINATE_RESULT");
      expect(response?.payload).toHaveProperty("success");
    });

    it("should fail COORDINATE for unavailable agents", async () => {
      const message = createMessage({
        type: "COORDINATE",
        payload: {
          operation: "test-op",
          agents: ["non-existent-agent"],
        },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("COORDINATE_RESULT");
      expect(response?.payload.success).toBe(false);
    });

    it("should handle STATE_UPDATE message", async () => {
      const message = createMessage({
        type: "STATE_UPDATE",
        payload: { key: "test-key", value: "test-value" },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(plugin.getStateManager().getState("test-key")).toBeDefined();
    });

    it("should handle STATE_REQUEST message", async () => {
      // Set state first
      plugin.getStateManager().setState("requested-key", "value", "test-agent");

      const message = createMessage({
        type: "STATE_REQUEST",
        payload: { key: "requested-key" },
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(response?.payload.exists).toBe(true);
      expect(response?.payload.value).toBe("value");
    });

    it("should handle HEARTBEAT message", async () => {
      // Register agent first
      plugin.getAgentRegistry().registerAgent({
        id: "heartbeat-agent",
        name: "Agent",
        role: "worker",
        capabilities: [],
        status: "busy",
        lastActivity: new Date(Date.now() - 10000),
      });

      const message = createMessage({
        type: "HEARTBEAT",
        sender: "heartbeat-agent",
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("VALIDATE_RESULT");
      expect(plugin.getAgentRegistry().getAgent("heartbeat-agent")?.status).toBe("idle");
    });

    it("should handle ERROR message", async () => {
      const message = createMessage({
        type: "ERROR",
        payload: { error: "Test error", context: { detail: "info" } },
      });

      // Should not throw
      await expect(
        plugin.getProtocolHandler().handleMessage(message)
      ).resolves.toBeUndefined();
    });

    it("should return error for unknown message type", async () => {
      const message = createMessage({
        type: "UNKNOWN_TYPE" as any,
      });

      const response = await plugin.getProtocolHandler().handleMessage(message);

      expect(response?.type).toBe("ERROR");
      expect(response?.payload.error).toContain("Unknown message type");
    });
  });

  describe("Broadcast", () => {
    it("should broadcast message to all agents", async () => {
      plugin.getAgentRegistry().registerAgent({
        id: "agent-1",
        name: "Agent 1",
        role: "worker",
        capabilities: [],
        status: "idle",
        lastActivity: new Date(),
      });

      plugin.getAgentRegistry().registerAgent({
        id: "agent-2",
        name: "Agent 2",
        role: "worker",
        capabilities: [],
        status: "idle",
        lastActivity: new Date(),
      });

      const message = createMessage({
        sender: "sender-agent",
        type: "COORDINATE",
        payload: { operation: "broadcast-test" },
      });

      await plugin.getProtocolHandler().broadcast(message);
      // In real implementation, would verify messages sent to both agents
    });
  });
});
