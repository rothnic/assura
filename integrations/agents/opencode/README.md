# Assura OpenCode Plugin

OpenCode plugin for Assura constraint validation system. Provides file system validation and naming convention enforcement for OpenCode agents.

## Features

- **Pre-tool validation**: Intercept and validate tool calls before execution
- **Post-tool validation**: Validate results and affected files after execution
- **Multi-agent support**: Coordinate validation across multiple agents
- **State management**: Shared state with conflict resolution
- **Protocol handling**: Agent communication protocol implementation

## Installation

```bash
bun install @assura/opencode-plugin
```

## Usage

### Basic Usage

```typescript
import { createPlugin } from '@assura/opencode-plugin';

const plugin = createPlugin({
  validation: {
    strict: true,
    severityThreshold: 'High',
    autoFix: false,
  },
  agent: {
    multiAgent: true,
    coordinationMode: 'peer',
  },
});

await plugin.initialize();

// Use hooks
const result = await plugin.preToolUse({
  callId: '123',
  toolName: 'WriteFile',
  args: { 
    filePath: 'src/my-file.ts', 
    content: 'console.log("test");' 
  },
  agentId: 'agent-1',
  sessionId: 'session-1',
  timestamp: new Date(),
});

if (!result.proceed) {
  console.error('Validation failed:', result.errors);
}

await plugin.shutdown();
```

### Configuration Options

```typescript
interface PluginConfig {
  name: string;
  version: string;
  description?: string;
  hooks?: {
    preToolUse?: boolean;
    postToolUse?: boolean;
    onError?: boolean;
  };
  validation?: {
    strict: boolean;
    severityThreshold: 'Critical' | 'High' | 'Medium' | 'Low';
    autoFix: boolean;
  };
  agent?: {
    multiAgent: boolean;
    coordinationMode: 'leader' | 'peer' | 'hierarchical';
    stateNamespace?: string;
  };
}
```

## Hooks

### preToolUse

Validates tool calls before execution. Can block operations based on validation results.

```typescript
const result = await plugin.preToolUse(context);
if (!result.proceed) {
  // Operation blocked
  console.log('Blocked:', result.errors);
}
```

### postToolUse

Validates results after tool execution. Reports violations in created/modified files.

```typescript
const result = await plugin.postToolUse(context, toolResult);
if (result.errors.length > 0) {
  console.log('Violations found:', result.errors);
}
```

## Multi-Agent Support

### Agent Registration

```typescript
plugin.getAgentRegistry().registerAgent({
  id: 'my-agent',
  name: 'My Agent',
  role: 'worker',
  capabilities: ['file-read', 'file-write'],
  status: 'idle',
  lastActivity: new Date(),
});
```

### State Management

```typescript
// Set state
plugin.getStateManager().setState('key', 'value', 'agent-id');

// Get state
const entry = plugin.getStateManager().getState('key');

// Subscribe to conflicts
plugin.getStateManager().subscribeToConflicts((conflict) => {
  console.log('Conflict resolved:', conflict);
});
```

## Validation Rules

### Naming Conventions

- **kebab-case**: Default for files and directories (e.g., `my-file.ts`)
- **PascalCase**: Component names (e.g., `MyComponent.tsx`)
- **camelCase**: Variable names

### File Content

- No trailing whitespace
- No tabs (use spaces)
- Max line length: 120 characters
- No console.log in production code
- No TODO/FIXME comments

### Security

- Blocked paths: `.env`, `.git/config`, `.ssh`, `.aws`
- Dangerous command detection in Bash operations
- Sensitive file access warnings

## Testing

```bash
# Run all tests
bun test

# Run specific test file
bun test tests/plugin.test.ts

# Run with coverage
bun test --coverage
```

## Building

```bash
# Build TypeScript
bun run build

# Development mode with watch
bun run dev

# Lint
bun run lint
```

## Architecture

```
src/
├── index.ts          # Main exports
├── types.ts          # Type definitions
├── plugin.ts         # Main plugin implementation
├── agent-registry.ts # Agent registration and discovery
├── state-manager.ts  # Shared state management
├── protocol.ts       # Agent communication protocol
├── validation.ts     # Validation engine
└── hooks.ts          # Hook implementations
```

## License

MIT
