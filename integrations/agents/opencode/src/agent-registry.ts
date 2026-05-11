/**
 * Agent Registry
 * 
 * Manages agent registration, discovery, and coordination.
 */

import type {
  AgentInfo,
  AgentStatus,
  AgentRegistry,
} from "./types";

/**
 * In-memory agent registry implementation
 */
export class AgentRegistryImpl implements AgentRegistry {
  private agents: Map<string, AgentInfo> = new Map();
  private listeners: Array<(agent: AgentInfo, event: "register" | "unregister" | "update") => void> = [];

  /**
   * Register a new agent
   */
  public registerAgent(agent: AgentInfo): void {
    if (this.agents.has(agent.id)) {
      throw new Error(`Agent ${agent.id} is already registered`);
    }

    this.agents.set(agent.id, {
      ...agent,
      lastActivity: new Date(),
    });

    console.log(`[AgentRegistry] Registered agent: ${agent.id} (${agent.name})`);
    this.notifyListeners(agent, "register");
  }

  /**
   * Unregister an agent
   */
  public unregisterAgent(agentId: string): void {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new Error(`Agent ${agentId} is not registered`);
    }

    this.agents.delete(agentId);
    console.log(`[AgentRegistry] Unregistered agent: ${agentId}`);
    this.notifyListeners(agent, "unregister");
  }

  /**
   * Get agent by ID
   */
  public getAgent(agentId: string): AgentInfo | undefined {
    return this.agents.get(agentId);
  }

  /**
   * List all registered agents
   */
  public listAgents(): AgentInfo[] {
    return Array.from(this.agents.values());
  }

  /**
   * Update agent status
   */
  public updateAgentStatus(agentId: string, status: AgentStatus): void {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new Error(`Agent ${agentId} is not registered`);
    }

    agent.status = status;
    agent.lastActivity = new Date();
    this.agents.set(agentId, agent);
    this.notifyListeners(agent, "update");
  }

  /**
   * Find agents by capability
   */
  public findAgentsByCapability(capability: string): AgentInfo[] {
    return this.listAgents().filter((agent) =>
      agent.capabilities.includes(capability)
    );
  }

  /**
   * Find agents by role
   */
  public findAgentsByRole(role: string): AgentInfo[] {
    return this.listAgents().filter((agent) => agent.role === role);
  }

  /**
   * Check if agent is registered
   */
  public isRegistered(agentId: string): boolean {
    return this.agents.has(agentId);
  }

  /**
   * Get agent count
   */
  public getAgentCount(): number {
    return this.agents.size;
  }

  /**
   * Subscribe to registry events
   */
  public subscribe(
    listener: (agent: AgentInfo, event: "register" | "unregister" | "update") => void
  ): () => void {
    this.listeners.push(listener);
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of an event
   */
  private notifyListeners(
    agent: AgentInfo,
    event: "register" | "unregister" | "update"
  ): void {
    for (const listener of this.listeners) {
      try {
        listener(agent, event);
      } catch (error) {
        console.error("[AgentRegistry] Error in listener:", error);
      }
    }
  }

  /**
   * Clear all agents (useful for testing)
   */
  public clear(): void {
    this.agents.clear();
    this.listeners = [];
  }
}

export default AgentRegistryImpl;
