import { BaseEventHandler } from './base-handler'
import { DoneEventData, ChatState } from '@/types/events'

/**
 * Handles stream completion
 * Commits any remaining streaming message/tool call
 */
export class DoneHandler extends BaseEventHandler<DoneEventData> {
  handle(data: DoneEventData, state: ChatState): ChatState {
    let newState = this.commitStreamingMessage(state)
    
    // Commit streaming tool call if exists
    if (newState.streamingToolCall) {
      newState = this.cloneState(newState)
      if (data.status === 'completed') {
        newState.streamingToolCall.status = 'completed'
      } else {
        newState.streamingToolCall.status = 'error'
      }
      newState.items.push({ ...newState.streamingToolCall })
      newState.streamingToolCall = null
    }
    
    return newState
  }
}
