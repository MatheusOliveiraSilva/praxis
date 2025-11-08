import { BaseEventHandler } from './base-handler'
import { ToolCallEventData, ChatState, UIToolCall } from '@/types/events'

/**
 * Handles tool call events (name, arguments streaming)
 * Commits message before starting tool call
 */
export class ToolCallHandler extends BaseEventHandler<ToolCallEventData> {
  handle(data: ToolCallEventData, state: ChatState): ChatState {
    // Commit any streaming message before tool call
    state = this.commitStreamingMessage(state)
    
    const newState = this.cloneState(state)
    const index = data.index ?? 0
    
    // Check if we have a streaming tool call with this index
    if (newState.streamingToolCall && newState.streamingToolCall.index === index) {
      // Update existing streaming tool call
      if (data.id) newState.streamingToolCall.id = data.id
      if (data.name) newState.streamingToolCall.name = data.name
      if (data.arguments) {
        newState.streamingToolCall.arguments += data.arguments
      }
    } else {
      // Commit previous tool call if exists
      if (newState.streamingToolCall) {
        newState.toolCalls.push({ ...newState.streamingToolCall })
      }
      
      // Start new streaming tool call
      newState.streamingToolCall = {
        id: data.id || this.generateId(),
        index,
        name: data.name || '',
        arguments: data.arguments || '',
        status: 'running',
        timestamp: new Date()
      }
    }
    
    return newState
  }
}
