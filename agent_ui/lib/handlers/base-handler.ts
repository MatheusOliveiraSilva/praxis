import { SSEEventData, ChatState, UIMessage, StreamingMessage, MessageType, ChatItem } from '@/types/events'

/**
 * Base handler interface for SSE events
 * Uses Strategy Pattern for flexible event handling
 */
export interface EventHandler<T extends SSEEventData = SSEEventData> {
  handle(data: T, state: ChatState): ChatState
}

/**
 * Abstract base class with common utilities
 */
export abstract class BaseEventHandler<T extends SSEEventData = SSEEventData> 
  implements EventHandler<T> {
  
  abstract handle(data: T, state: ChatState): ChatState
  
  /**
   * Creates immutable state update (O(1) reference copy)
   */
  protected cloneState(state: ChatState): ChatState {
    return {
      ...state,
      items: [...state.items]
    }
  }
  
  /**
   * Generate unique ID (timestamp + random for collision avoidance)
   */
  protected generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }
  
  /**
   * Commit streaming message to items array
   * Called when type changes or stream ends
   */
  protected commitStreamingMessage(state: ChatState): ChatState {
    if (!state.streamingMessage || !state.streamingMessage.content) {
      return state
    }
    
    const newMessage: UIMessage = {
      itemType: 'message',
      id: this.generateId(),
      type: state.streamingMessage.type,
      content: state.streamingMessage.content,
      timestamp: new Date()
    }
    
    return {
      ...state,
      items: [...state.items, newMessage],
      streamingMessage: null
    }
  }
  
  /**
   * Start or append to streaming message
   */
  protected updateStreamingMessage(
    state: ChatState, 
    type: MessageType, 
    content: string
  ): ChatState {
    // If type changed, commit previous message first
    if (state.streamingMessage && state.streamingMessage.type !== type) {
      state = this.commitStreamingMessage(state)
    }
    
    // Start new or append to existing
    if (!state.streamingMessage || state.streamingMessage.type !== type) {
      return {
        ...state,
        streamingMessage: { type, content }
      }
    } else {
      return {
        ...state,
        streamingMessage: {
          type,
          content: state.streamingMessage.content + content
        }
      }
    }
  }
}
