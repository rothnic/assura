/**
 * Protocol Handler
 * 
 * Handles agent protocol messages for multi-agent coordination.
 */

import type {
  ProtocolMessage,
  MessageType,
  ProtocolHandler,
  AgentInfo,
  ValidationError,
  HookResult,
} from "./types";
import type { AgentRegistryImpl } from "./agent-registry";
import type { StateManagerImpl } from "./state-manager";

/**
 * Protocol handler implementation
 */
export class ProtocolHandlerImpl implements ProtocolHandler {
  private agentRegistry: AgentRegistryImpl;
  private stateManager: StateManagerImpl;
  private messageHandlers: Map<MessageType, (message: ProtocolMessage) => Promise<ProtocolMessage | void>> = new Map();
  private messageQueue: ProtocolMessage[] = [];
  private processing = false;

  constructor(agentRegistry: AgentRegistryImpl, stateManager: StateManagerImpl) {
    this.agentRegistry = agentRegistry;
    this.stateManager = stateManager;
    this.registerDefaultHandlers();
  }

  /**
   * Register default message handlers
   */
  private registerDefaultHandlers(): void {
    this.messageHandlers.set("REGISTER", this.handleRegister.bind(this));
    this.messageHandlers.set("UNREGISTER", this.handleUnregister.bind(this));
    this.messageHandlers.set("VALIDATE", this.handleValidate.bind(this));
    this.messageHandlers.set("VALIDATE_RESULT", this.handleValidateResult.bind(this));
    this.messageHandlers.set("COORDINATE", this.handleCoordinate.bind(this));
    this.messageHandlers.set("COORDINATE_RESULT", this.handleCoordinateResult.bind(this));
    this.messageHandlers.set("STATE_UPDATE", this.handleStateUpdate.bind(this));
    this.messageHandlers.set("STATE_REQUEST", this.handleStateRequest.bind(this));
    this.messageHandlers.set("ERROR", this.handleError.bind(this));
    this.messageHandlers.set("HEARTBEAT", this.handleHeartbeat.bind(this));
  }

  /**
   * Handle incoming message
   */
  public async handleMessage(message: ProtocolMessage): Promise<ProtocolMessage | void> {
    const handler = this.messageHandlers.get(message.type);
    
    if (!handler) {
      console.warn(`[ProtocolHandler] No handler for message type: ${message.type}`);
      return this.createErrorResponse(
        message,
        `Unknown message type: ${message.type}`
      );
    }

    try {
      return await handler(message);
    } catch (error) {
      console.error(`[ProtocolHandler] Error handling message ${message.type}:`, error);
      return this.createErrorResponse(
        message,
        error instanceof Error ? error.message : "Unknown error"
      );
    }
  }

  /**
   * Send message to specific agent
   */
  public async sendMessage(message: ProtocolMessage): Promise<void> {
    if (!message.recipient) {
      throw new Error("Recipient is required for direct messages");
    }

    const agent = this.agentRegistry.getAgent(message.recipient);
    if (!agent) {
      throw new Error(`Agent ${message.recipient} not found`);
    }

    // In a real implementation, this would send over a transport
    // For now, we just queue it
    this.messageQueue.push(message);
    this.processQueue();
  }

  /**
   * Broadcast message to all agents
   */
  public async broadcast(message: Omit<ProtocolMessage, "recipient">): Promise<void> {
    const agents = this.agentRegistry.listAgents();
    
    for (const agent of agents) {
      if (agent.id !== message.sender) {
        const directMessage: ProtocolMessage = {
          ...message,
          recipient: agent.id,
        };
        this.messageQueue.push(directMessage);
      }
    }

    this.processQueue();
  }

  /**
   * Register a custom message handler
   */
  public registerHandler(
    type: MessageType,
    handler: (message: ProtocolMessage) => Promise<ProtocolMessage | void>
  ): void {
    this.messageHandlers.set(type, handler);
  }

  /**
   * Handle REGISTER message
   */
  private async handleRegister(message: ProtocolMessage): Promise<ProtocolMessage> {
    const agentInfo = message.payload as AgentInfo;
    
    this.agentRegistry.registerAgent(agentInfo);

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        success: true,
        registeredAgents: this.agentRegistry.getAgentCount(),
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle UNREGISTER message
   */
  private async handleUnregister(message: ProtocolMessage): Promise<ProtocolMessage> {
    const { agentId } = message.payload as { agentId: string };
    
    this.agentRegistry.unregisterAgent(agentId);

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        success: true,
        registeredAgents: this.agentRegistry.getAgentCount(),
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle VALIDATE message
   */
  private async handleValidate(message: ProtocolMessage): Promise<ProtocolMessage> {
    const { filePath, content } = message.payload as {
      filePath: string;
      content?: string;
    };

    // This would integrate with the actual validation engine
    const errors: ValidationError[] = [];

    // Simulate validation
    if (filePath.includes("invalid")) {
      errors.push({
        message: "File name contains 'invalid'",
        severity: "High",
        filePath,
        rule: "naming-convention",
        autoFixable: false,
      });
    }

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        valid: errors.length === 0,
        errors,
        filePath,
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle VALIDATE_RESULT message
   */
  private async handleValidateResult(message: ProtocolMessage): Promise<void> {
    const result = message.payload as {
      valid: boolean;
      errors: ValidationError[];
      filePath: string;
    };

    console.log(`[ProtocolHandler] Validation result for ${result.filePath}:`,
      result.valid ? "VALID" : `INVALID (${result.errors.length} errors)`
    );
  }

  /**
   * Handle COORDINATE message
   */
  private async handleCoordinate(message: ProtocolMessage): Promise<ProtocolMessage> {
    const { operation, agents } = message.payload as {
      operation: string;
      agents: string[];
    };

    console.log(`[ProtocolHandler] Coordination request: ${operation} for agents:`, agents);

    // Check if all agents are available
    const unavailableAgents = agents.filter(
      (id) => !this.agentRegistry.isRegistered(id)
    );

    if (unavailableAgents.length > 0) {
      return {
        id: this.generateId(),
        type: "COORDINATE_RESULT",
        sender: "assura-plugin",
        recipient: message.sender,
        payload: {
          success: false,
          error: `Agents not available: ${unavailableAgents.join(", ")}`,
        },
        timestamp: new Date(),
        correlationId: message.id,
      };
    }

    return {
      id: this.generateId(),
      type: "COORDINATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        success: true,
        operation,
        agents,
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle COORDINATE_RESULT message
   */
  private async handleCoordinateResult(message: ProtocolMessage): Promise<void> {
    const result = message.payload as {
      success: boolean;
      operation?: string;
      error?: string;
    };

    if (result.success) {
      console.log(`[ProtocolHandler] Coordination successful: ${result.operation}`);
    } else {
      console.error(`[ProtocolHandler] Coordination failed: ${result.error}`);
    }
  }

  /**
   * Handle STATE_UPDATE message
   */
  private async handleStateUpdate(message: ProtocolMessage): Promise<ProtocolMessage> {
    const { key, value } = message.payload as { key: string; value: unknown };
    
    this.stateManager.setState(key, value, message.sender);

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        success: true,
        key,
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle STATE_REQUEST message
   */
  private async handleStateRequest(message: ProtocolMessage): Promise<ProtocolMessage> {
    const { key } = message.payload as { key: string };
    const entry = this.stateManager.getState(key);

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        key,
        exists: !!entry,
        value: entry?.value,
        owner: entry?.owner,
        updatedAt: entry?.updatedAt,
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Handle ERROR message
   */
  private async handleError(message: ProtocolMessage): Promise<void> {
    const { error, context } = message.payload as {
      error: string;
      context?: Record<string, unknown>;
    };

    console.error(`[ProtocolHandler] Error from ${message.sender}:`, error, context);
  }

  /**
   * Handle HEARTBEAT message
   */
  private async handleHeartbeat(message: ProtocolMessage): Promise<ProtocolMessage> {
    this.agentRegistry.updateAgentStatus(message.sender, "idle");

    return {
      id: this.generateId(),
      type: "VALIDATE_RESULT",
      sender: "assura-plugin",
      recipient: message.sender,
      payload: {
        status: "ack",
        timestamp: new Date().toISOString(),
      },
      timestamp: new Date(),
      correlationId: message.id,
    };
  }

  /**
   * Create error response message
   */
  private createErrorResponse(
    originalMessage: ProtocolMessage,
    error: string
  ): ProtocolMessage {
    return {
      id: this.generateId(),
      type: "ERROR",
      sender: "assura-plugin",
      recipient: originalMessage.sender,
      payload: {
        error,
        originalMessage: originalMessage.id,
      },
      timestamp: new Date(),
      correlationId: originalMessage.id,
    };
  }

  /**
   * Generate unique message ID
   */
  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
  }

  /**
   * Process message queue
   */
  private async processQueue(): Promise<void> {
    if (this.processing || this.messageQueue.length === 0) {
      return;
    }

    this.processing = true;

    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift();
      if (message) {
        await this.handleMessage(message);
      }
    }

    this.processing = false;
  }
}

export default ProtocolHandlerImpl;
