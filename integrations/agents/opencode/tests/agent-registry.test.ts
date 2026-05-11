/**
 * Agent Registry Tests
 * 
 * Tests for agent registration and coordination
 */

import { describe, it, expect, beforeEach } from "bun:test";
import { AgentRegistryImpl, type AgentInfo } from "../src/index";

describe("Agent Registry", () => {
  let registry: AgentRegistryImpl;

  beforeEach(() => {
    registry = new AgentRegistryImpl();
  });

  const createAgent = (id: string, overrides: Partial<AgentInfo> = {}): AgentInfo => ({
    id,
    name: `Agent ${id}`,
    role: "worker",
    capabilities: ["file-read", "file-write"],
    status: "idle",
    lastActivity: new Date(),
    ...overrides,
  });

  it("should register an agent", () => {
    const agent = createAgent("agent-1");
    registry.registerAgent(agent);

    expect(registry.getAgentCount()).toBe(1);
    expect(registry.getAgent("agent-1")).toBeDefined();
  });

  it("should throw when registering duplicate agent", () => {
    const agent = createAgent("agent-1");
    registry.registerAgent(agent);

    expect(() => registry.registerAgent(agent)).toThrow(
      "already registered"
    );
  });

  it("should unregister an agent", () => {
    const agent = createAgent("agent-1");
    registry.registerAgent(agent);
    registry.unregisterAgent("agent-1");

    expect(registry.getAgentCount()).toBe(0);
    expect(registry.getAgent("agent-1")).toBeUndefined();
  });

  it("should throw when unregistering unknown agent", () => {
    expect(() => registry.unregisterAgent("unknown")).toThrow(
      "not registered"
    );
  });

  it("should list all agents", () => {
    registry.registerAgent(createAgent("agent-1"));
    registry.registerAgent(createAgent("agent-2"));
    registry.registerAgent(createAgent("agent-3"));

    const agents = registry.listAgents();
    expect(agents.length).toBe(3);
    expect(agents.map((a) => a.id).sort()).toEqual([
      "agent-1",
      "agent-2",
      "agent-3",
    ]);
  });

  it("should update agent status", () => {
    const agent = createAgent("agent-1");
    registry.registerAgent(agent);

    registry.updateAgentStatus("agent-1", "busy");

    const updated = registry.getAgent("agent-1");
    expect(updated?.status).toBe("busy");
  });

  it("should throw when updating unknown agent status", () => {
    expect(() => registry.updateAgentStatus("unknown", "busy")).toThrow(
      "not registered"
    );
  });

  it("should find agents by capability", () => {
    registry.registerAgent(createAgent("agent-1", { capabilities: ["read"] }));
    registry.registerAgent(createAgent("agent-2", { capabilities: ["write"] }));
    registry.registerAgent(createAgent("agent-3", { capabilities: ["read", "write"] }));

    const readers = registry.findAgentsByCapability("read");
    expect(readers.length).toBe(2);
    expect(readers.map((a) => a.id).sort()).toEqual(["agent-1", "agent-3"]);

    const writers = registry.findAgentsByCapability("write");
    expect(writers.length).toBe(2);
  });

  it("should find agents by role", () => {
    registry.registerAgent(createAgent("agent-1", { role: "leader" }));
    registry.registerAgent(createAgent("agent-2", { role: "worker" }));
    registry.registerAgent(createAgent("agent-3", { role: "worker" }));

    const leaders = registry.findAgentsByRole("leader");
    expect(leaders.length).toBe(1);
    expect(leaders[0].id).toBe("agent-1");

    const workers = registry.findAgentsByRole("worker");
    expect(workers.length).toBe(2);
  });

  it("should check if agent is registered", () => {
    registry.registerAgent(createAgent("agent-1"));

    expect(registry.isRegistered("agent-1")).toBe(true);
    expect(registry.isRegistered("agent-2")).toBe(false);
  });

  it("should notify listeners on register", () => {
    const events: Array<{ agent: AgentInfo; event: string }> = [];
    registry.subscribe((agent, event) => {
      events.push({ agent, event });
    });

    const agent = createAgent("agent-1");
    registry.registerAgent(agent);

    expect(events.length).toBe(1);
    expect(events[0].agent.id).toBe("agent-1");
    expect(events[0].event).toBe("register");
  });

  it("should notify listeners on unregister", () => {
    const events: Array<{ agent: AgentInfo; event: string }> = [];
    registry.subscribe((agent, event) => {
      events.push({ agent, event });
    });

    const agent = createAgent("agent-1");
    registry.registerAgent(agent);
    registry.unregisterAgent("agent-1");

    expect(events.length).toBe(2);
    expect(events[1].event).toBe("unregister");
  });

  it("should allow unsubscribing", () => {
    const events: Array<{ agent: AgentInfo; event: string }> = [];
    const unsubscribe = registry.subscribe((agent, event) => {
      events.push({ agent, event });
    });

    unsubscribe();

    const agent = createAgent("agent-1");
    registry.registerAgent(agent);

    expect(events.length).toBe(0);
  });
});
