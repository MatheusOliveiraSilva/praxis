import { BaseEventHandler } from './base-handler'
import { ReasoningEventData, ChatState } from '@/types/events'

/**
 * Handles reasoning chunks from assistant
 * Accumulates until type changes, then commits
 */
export class ReasoningHandler extends BaseEventHandler<ReasoningEventData> {
  handle(data: ReasoningEventData, state: ChatState): ChatState {
    return this.updateStreamingMessage(state, 'reasoning', data.content)
  }
}
