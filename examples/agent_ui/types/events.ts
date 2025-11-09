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

// UI Types
export type MessageType = 'user' | 'assistant' | 'reasoning'
export type ItemType = 'message' | 'tool_call'

export interface UIMessage {
  itemType: 'message'
  id: string
  type: MessageType
  content: string
  timestamp: Date
}

export interface UIToolCall {
  itemType: 'tool_call'
  id: string
  index: number
  name: string
  arguments: string
  result?: string
  status: 'running' | 'completed' | 'error'
  timestamp: Date
}

// Union type for all items
export type ChatItem = UIMessage | UIToolCall

// Streaming state
export interface StreamingMessage {
  type: MessageType
  content: string
}

export interface ChatState {
  // Unified items array (chronologically ordered)
  items: ChatItem[]
  // Currently streaming message (being accumulated)
  streamingMessage: StreamingMessage | null
  // Currently streaming tool call
  streamingToolCall: UIToolCall | null
  // Stream state
  isStreaming: boolean
  error: string | null
}
