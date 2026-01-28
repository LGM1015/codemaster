export interface Message {
  role: 'user' | 'assistant' | 'tool';
  content?: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
  name?: string;
}

export interface ToolCall {
  id: string;
  type: 'function';
  function: {
    name: string;
    arguments: string;
  };
}

// Struct matching Rust AgentEvent
export type AgentEvent = 
  | { type: 'Thinking'; content: string }
  | { type: 'StreamChunk'; content: string }
  | { type: 'StreamEnd'; content: null }
  | { type: 'ToolCall'; content: { name: string; args: string; id: string } }
  | { type: 'ToolResult'; content: { name: string; result: string; id: string } }
  | { type: 'Message'; content: string }
  | { type: 'NewMessage'; content: Message }
  | { type: 'Error'; content: string }
  | { type: 'Done'; content: null };
