/**
 * State Manager Tests
 * 
 * Tests for shared state management and conflict resolution
 */

import { describe, it, expect, beforeEach } from "bun:test";
import { StateManagerImpl } from "../src/index";

describe("State Manager", () => {
  let stateManager: StateManagerImpl;

  beforeEach(() => {
    stateManager = new StateManagerImpl();
  });

  it("should set and get state", () => {
    stateManager.setState("key1", "value1", "agent-1");
    
    const entry = stateManager.getState("key1");
    expect(entry).toBeDefined();
    expect(entry?.value).toBe("value1");
    expect(entry?.owner).toBe("agent-1");
    expect(entry?.version).toBe(1);
  });

  it("should update existing state from same owner", () => {
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key1", "value2", "agent-1");
    
    const entry = stateManager.getState("key1");
    expect(entry?.value).toBe("value2");
    expect(entry?.version).toBe(2);
  });

  it("should resolve conflict from different owners (last-write-wins)", () => {
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key1", "value2", "agent-2");
    
    const entry = stateManager.getState("key1");
    // Last write wins by default
    expect(entry?.value).toBe("value2");
    expect(entry?.owner).toBe("agent-2");
  });

  it("should delete state by owner", () => {
    stateManager.setState("key1", "value1", "agent-1");
    const deleted = stateManager.deleteState("key1", "agent-1");
    
    expect(deleted).toBe(true);
    expect(stateManager.getState("key1")).toBeUndefined();
  });

  it("should not delete state owned by different agent", () => {
    stateManager.setState("key1", "value1", "agent-1");
    const deleted = stateManager.deleteState("key1", "agent-2");
    
    expect(deleted).toBe(false);
    expect(stateManager.getState("key1")).toBeDefined();
  });

  it("should list all keys", () => {
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key2", "value2", "agent-1");
    stateManager.setState("key3", "value3", "agent-2");
    
    const keys = stateManager.listKeys();
    expect(keys.length).toBe(3);
    expect(keys.sort()).toEqual(["key1", "key2", "key3"]);
  });

  it("should get state by owner", () => {
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key2", "value2", "agent-1");
    stateManager.setState("key3", "value3", "agent-2");
    
    const agent1State = stateManager.getStateByOwner("agent-1");
    expect(agent1State.length).toBe(2);
    expect(agent1State.map((s) => s.key).sort()).toEqual(["key1", "key2"]);
  });

  it("should clear all state", () => {
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key2", "value2", "agent-1");
    
    stateManager.clearState();
    
    expect(stateManager.listKeys().length).toBe(0);
    expect(stateManager.getStateSize()).toBe(0);
  });

  it("should notify on conflict", () => {
    const conflicts: Array<{ agents: string[] }> = [];
    stateManager.subscribeToConflicts((conflict) => {
      conflicts.push(conflict);
    });
    
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key1", "value2", "agent-2");
    
    expect(conflicts.length).toBe(1);
    expect(conflicts[0].agents.sort()).toEqual(["agent-1", "agent-2"]);
  });

  it("should allow unsubscribing from conflicts", () => {
    const conflicts: Array<{ agents: string[] }> = [];
    const unsubscribe = stateManager.subscribeToConflicts((conflict) => {
      conflicts.push(conflict);
    });
    
    unsubscribe();
    
    stateManager.setState("key1", "value1", "agent-1");
    stateManager.setState("key1", "value2", "agent-2");
    
    expect(conflicts.length).toBe(0);
  });

  it("should check if key exists", () => {
    stateManager.setState("key1", "value1", "agent-1");
    
    expect(stateManager.hasKey("key1")).toBe(true);
    expect(stateManager.hasKey("key2")).toBe(false);
  });

  it("should track version numbers", () => {
    stateManager.setState("key1", "value1", "agent-1");
    expect(stateManager.getState("key1")?.version).toBe(1);
    
    stateManager.setState("key1", "value2", "agent-1");
    expect(stateManager.getState("key1")?.version).toBe(2);
    
    stateManager.setState("key1", "value3", "agent-2");
    expect(stateManager.getState("key1")?.version).toBe(3);
  });

  it("should merge states with merge strategy", () => {
    stateManager.setConflictStrategy("merge");
    
    stateManager.setState("key1", { a: 1, b: 2 }, "agent-1");
    stateManager.setState("key1", { b: 3, c: 4 }, "agent-2");
    
    const entry = stateManager.getState("key1");
    expect(entry?.value).toEqual({ a: 1, b: 3, c: 4 });
  });
});
