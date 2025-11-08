// SSE Event Types from Backend
export type SSEEventType = 'info' | 'message' | 'reasoning' | 'tool_call' | 'tool_result' | 'done' | 'error'

export interface SSEEvent {
  event: SSEEventType
  data: SSEEventData
}

export type SSEEventData = 
  | InfoEventData
  | MessageEventData
  | ReasoningEventData
  | ToolCallEventData
  | ToolResultEventData
  | DoneEventData
  | ErrorEventData

export interface InfoEventData {
  [key: string]: unknown
}

export interface MessageEventData {
  content: string
}

export interface ReasoningEventData {
  content: string
}

export interface ToolCallEventData {
  index?: number
  id?: string
  name?: string
  arguments?: string
}

export interface ToolResultEventData {
  result: string
}

export interface DoneEventData {
  status: 'completed' | 'failed'
}

export interface ErrorEventData {
  error: string
  [key: string]: unknown
}

// UI State Types
export type MessageType = 'user' | 'assistant' | 'reasoning'

export interface UIMessage {
  id: string
  type: MessageType
  content: string
  timestamp: Date
}

export interface UIToolCall {
  id: string
  index: number
  name: string
  arguments: string
  result?: string
  status: 'running' | 'completed' | 'error'
  timestamp: Date
}

// Streaming state
export interface StreamingMessage {
  type: MessageType
  content: string
}

export interface ChatState {
  // Committed messages (finalized)
  messages: UIMessage[]
  // Currently streaming message (being accumulated)
  streamingMessage: StreamingMessage | null
  // Tool calls (separate from messages)
  toolCalls: UIToolCall[]
  // Currently streaming tool call
  streamingToolCall: UIToolCall | null
  // Stream state
  isStreaming: boolean
  error: string | null
}
