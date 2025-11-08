import { BaseEventHandler } from './base-handler'
import { ToolResultEventData, ChatState } from '@/types/events'

/**
 * Handles tool execution results
 * Commits streaming tool call with result
 */
export class ToolResultHandler extends BaseEventHandler<ToolResultEventData> {
  handle(data: ToolResultEventData, state: ChatState): ChatState {
    const newState = this.cloneState(state)
    
    // Commit streaming tool call with result
    if (newState.streamingToolCall) {
      newState.streamingToolCall.status = 'completed'
      newState.streamingToolCall.result = data.result
      newState.toolCalls.push({ ...newState.streamingToolCall })
      newState.streamingToolCall = null
    }
    
    return newState
  }
}
