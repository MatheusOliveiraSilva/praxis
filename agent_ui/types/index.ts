export interface Thread {
  _id: string
  user_id: string
  title: string
  created_at: string
  updated_at: string
}

export interface Message {
  _id: string
  thread_id: string
  user_id: string
  role: 'user' | 'assistant'
  content: string
  created_at: string
}

export interface ToolCall {
  index: number
  name: string
  arguments: string
  status: 'running' | 'completed' | 'error'
  result?: string
}

export interface StreamEvent {
  type: 'info' | 'message' | 'tool_call' | 'tool_result' | 'done' | 'error'
  data: any
}

