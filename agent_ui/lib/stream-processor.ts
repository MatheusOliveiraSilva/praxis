import { ChatState, SSEEventType, SSEEventData } from '@/types/events'
import { handlerRegistry } from './handlers'

/**
 * SSE Stream Processor
 * Handles parsing and dispatching of Server-Sent Events
 * Uses Strategy Pattern via handler registry
 */
export class StreamProcessor {
  private buffer: string = ''
  
  /**
   * Process incoming SSE data chunk
   * @param chunk Raw text chunk from stream
   * @returns Array of state updates (may be empty)
   */
  processChunk(chunk: string, currentState: ChatState): ChatState[] {
    const updates: ChatState[] = []
    
    // Append to buffer
    this.buffer += chunk
    
    // Split by newlines
    const lines = this.buffer.split('\n')
    
    // Keep last incomplete line in buffer
    this.buffer = lines.pop() || ''
    
    let currentEvent: SSEEventType | null = null
    
    for (const line of lines) {
      // Parse event type
      if (line.startsWith('event: ')) {
        currentEvent = line.substring(7).trim() as SSEEventType
        continue
      }
      
      // Parse data
      if (line.startsWith('data: ')) {
        const data = line.substring(6).trim()
        if (!data || !currentEvent) continue
        
        try {
          const parsed: SSEEventData = JSON.parse(data)
          const handler = handlerRegistry.getHandler(currentEvent)
          
          if (handler) {
            // Dispatch to appropriate handler
            const newState = handler.handle(parsed, currentState)
            updates.push(newState)
            currentState = newState // Chain state for next handler
          }
        } catch (error) {
          console.error('Failed to parse SSE data:', error, data)
        }
        
        currentEvent = null
      }
    }
    
    return updates
  }
  
  /**
   * Reset internal buffer (call when stream ends)
   */
  reset(): void {
    this.buffer = ''
  }
}

