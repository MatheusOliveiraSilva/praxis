import { EventHandler } from './base-handler'
import { MessageHandler } from './message-handler'
import { ReasoningHandler } from './reasoning-handler'
import { ToolCallHandler } from './tool-call-handler'
import { ToolResultHandler } from './tool-result-handler'
import { DoneHandler } from './done-handler'
import { SSEEventType, SSEEventData } from '@/types/events'

/**
 * Handler Registry (Singleton Pattern)
 * Maps event types to handlers for O(1) dispatch
 */
class HandlerRegistry {
  private static instance: HandlerRegistry
  private handlers: Map<SSEEventType, EventHandler>
  
  private constructor() {
    this.handlers = new Map([
      ['message', new MessageHandler()],
      ['reasoning', new ReasoningHandler()],
      ['tool_call', new ToolCallHandler()],
      ['tool_result', new ToolResultHandler()],
      ['done', new DoneHandler()],
    ])
  }
  
  static getInstance(): HandlerRegistry {
    if (!HandlerRegistry.instance) {
      HandlerRegistry.instance = new HandlerRegistry()
    }
    return HandlerRegistry.instance
  }
  
  getHandler(eventType: SSEEventType): EventHandler | undefined {
    return this.handlers.get(eventType)
  }
  
  registerHandler(eventType: SSEEventType, handler: EventHandler): void {
    this.handlers.set(eventType, handler)
  }
}

export const handlerRegistry = HandlerRegistry.getInstance()
export { MessageHandler, ReasoningHandler, ToolCallHandler, ToolResultHandler, DoneHandler }
