/**
 * State Manager
 * 
 * Manages shared state between agents with conflict resolution.
 */

import type {
  StateEntry,
  StateManager,
  ConflictResolution,
  ResolutionStrategy,
} from "./types";

/**
 * In-memory state manager implementation
 */
export class StateManagerImpl implements StateManager {
  private state: Map<string, StateEntry> = new Map();
  private conflictListeners: Array<(conflict: ConflictResolution) => void> = [];
  private defaultStrategy: ResolutionStrategy = "last-write-wins";

  /**
   * Get state entry by key
   */
  public getState(key: string): StateEntry | undefined {
    return this.state.get(key);
  }

  /**
   * Set state entry
   */
  public setState(key: string, value: unknown, owner: string): void {
    const existing = this.state.get(key);
    
    if (existing && existing.owner !== owner) {
      // Potential conflict - resolve
      const resolution = this.resolveConflict(existing, {
        key,
        value,
        owner,
        updatedAt: new Date(),
        version: (existing?.version || 0) + 1,
      });
      
      this.state.set(key, resolution.resolved);
      this.notifyConflictListeners(resolution);
    } else {
      // No conflict or same owner
      this.state.set(key, {
        key,
        value,
        owner,
        updatedAt: new Date(),
        version: (existing?.version || 0) + 1,
      });
    }
  }

  /**
   * Delete state entry
   */
  public deleteState(key: string, owner: string): boolean {
    const existing = this.state.get(key);
    
    if (!existing) {
      return false;
    }

    if (existing.owner !== owner) {
      // Cannot delete state owned by another agent
      console.warn(`[StateManager] Agent ${owner} cannot delete state owned by ${existing.owner}`);
      return false;
    }

    return this.state.delete(key);
  }

  /**
   * List all state keys
   */
  public listKeys(): string[] {
    return Array.from(this.state.keys());
  }

  /**
   * Get all state entries
   */
  public getAllState(): StateEntry[] {
    return Array.from(this.state.values());
  }

  /**
   * Get state by owner
   */
  public getStateByOwner(owner: string): StateEntry[] {
    return this.getAllState().filter((entry) => entry.owner === owner);
  }

  /**
   * Clear all state
   */
  public clearState(): void {
    this.state.clear();
  }

  /**
   * Set default conflict resolution strategy
   */
  public setConflictStrategy(strategy: ResolutionStrategy): void {
    this.defaultStrategy = strategy;
  }

  /**
   * Subscribe to conflict events
   */
  public subscribeToConflicts(
    listener: (conflict: ConflictResolution) => void
  ): () => void {
    this.conflictListeners.push(listener);
    return () => {
      const index = this.conflictListeners.indexOf(listener);
      if (index > -1) {
        this.conflictListeners.splice(index, 1);
      }
    };
  }

  /**
   * Resolve conflict between two state entries
   */
  private resolveConflict(
    existing: StateEntry,
    incoming: StateEntry
  ): ConflictResolution {
    const strategy = this.defaultStrategy;
    let resolved: StateEntry;

    switch (strategy) {
      case "last-write-wins":
        resolved = incoming;
        break;
      case "first-write-wins":
        resolved = existing;
        break;
      case "merge":
        resolved = this.mergeStates(existing, incoming);
        break;
      case "manual":
        // In manual mode, keep existing and let external handler decide
        resolved = existing;
        break;
      default:
        resolved = incoming;
    }

    return {
      resolved,
      agents: [existing.owner, incoming.owner],
      strategy,
      resolvedAt: new Date(),
    };
  }

  /**
   * Merge two state entries
   */
  private mergeStates(existing: StateEntry, incoming: StateEntry): StateEntry {
    const existingValue = existing.value as Record<string, unknown> || {};
    const incomingValue = incoming.value as Record<string, unknown> || {};

    return {
      key: existing.key,
      value: { ...existingValue, ...incomingValue },
      owner: incoming.owner, // Newest owner wins
      updatedAt: new Date(),
      version: Math.max(existing.version, incoming.version) + 1,
    };
  }

  /**
   * Notify all conflict listeners
   */
  private notifyConflictListeners(conflict: ConflictResolution): void {
    for (const listener of this.conflictListeners) {
      try {
        listener(conflict);
      } catch (error) {
        console.error("[StateManager] Error in conflict listener:", error);
      }
    }
  }

  /**
   * Get state size (for monitoring)
   */
  public getStateSize(): number {
    return this.state.size;
  }

  /**
   * Check if key exists
   */
  public hasKey(key: string): boolean {
    return this.state.has(key);
  }
}

export default StateManagerImpl;
