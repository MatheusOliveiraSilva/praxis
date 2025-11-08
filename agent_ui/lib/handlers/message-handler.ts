import { BaseEventHandler } from './base-handler'
import { MessageEventData, ChatState } from '@/types/events'

/**
 * Handles regular message chunks from assistant
 * Accumulates until type changes, then commits
 */
export class MessageHandler extends BaseEventHandler<MessageEventData> {
  handle(data: MessageEventData, state: ChatState): ChatState {
    return this.updateStreamingMessage(state, 'assistant', data.content)
  }
}
