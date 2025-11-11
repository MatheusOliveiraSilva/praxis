import { useState, useCallback, useRef } from 'react'
import { ChatState, UIMessage, ChatItem } from '@/types/events'
import { Message, Thread } from '@/types'
import { StreamProcessor } from '@/lib/stream-processor'

const API_BASE = 'http://localhost:8000'
const USER_ID = 'test_user'

interface UseChatOptions {
  thread: Thread | null
  onThreadCreated: (thread: Thread) => void
  onThreadUpdate: () => void
}

/**
 * Custom hook for chat functionality
 * Manages streaming, state, and API interactions
 */
export function useChat({ thread, onThreadCreated, onThreadUpdate }: UseChatOptions) {
  // Chat state (optimized with single state object)
  const [chatState, setChatState] = useState<ChatState>({
    items: [],
    streamingMessage: null,
    streamingToolCall: null,
    isStreaming: false,
    error: null
  })
  
  const streamProcessorRef = useRef<StreamProcessor>(new StreamProcessor())
  
  /**
   * Load messages from database
   * Only called when selecting existing thread
   */
  const loadMessages = useCallback(async (threadId: string) => {
    try {
      const response = await fetch(
        `${API_BASE}/threads/${threadId}/messages?user_id=${USER_ID}`
      )
      const data = await response.json()
      
      // Convert DB messages to UI items
      const items: ChatItem[] = (data.messages || []).map((msg: Message): UIMessage => {
        // Determine message type based on role and message_type field
        let type: 'user' | 'assistant' | 'reasoning' = 'assistant'
        if (msg.role === 'user') {
          type = 'user'
        } else if (msg.message_type === 'reasoning') {
          type = 'reasoning'
        } else {
          type = 'assistant'
        }
        
        return {
          itemType: 'message',
          id: msg._id,
          type,
          content: msg.content,
          timestamp: new Date(msg.created_at)
        }
      })
      
      setChatState(prev => ({
        ...prev,
        items,
        streamingMessage: null,
        streamingToolCall: null
      }))
    } catch (error) {
      console.error('Error loading messages:', error)
      setChatState(prev => ({
        ...prev,
        error: 'Failed to load messages'
      }))
    }
  }, [])
  
  /**
   * Send message and handle streaming response
   */
  const sendMessage = useCallback(async (content: string, llmConfig?: any) => {
    if (!content.trim() || chatState.isStreaming) return
    
    // Default LLM config
    const defaultConfig = {
      model: "gpt-4o-mini",
      temperature: 0.7,
      max_tokens: 8000
    }
    
    // Create or get thread
    let activeThread = thread
    if (!activeThread) {
      try {
        const response = await fetch(`${API_BASE}/threads`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            user_id: USER_ID,
            title: content.substring(0, 50)
          })
        })
        const data = await response.json()
        activeThread = {
          _id: data.thread_id,
          user_id: USER_ID,
          title: content.substring(0, 50),
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
        onThreadCreated(activeThread)
        onThreadUpdate()
      } catch (error) {
        console.error('Error creating thread:', error)
        setChatState(prev => ({ ...prev, error: 'Failed to create thread' }))
        return
      }
    }
    
    // Add user message
    const userMessage: UIMessage = {
      itemType: 'message',
      id: `${Date.now()}-user`,
      type: 'user',
      content,
      timestamp: new Date()
    }
    
    setChatState(prev => ({
      ...prev,
      items: [...prev.items, userMessage],
      streamingMessage: null,
      streamingToolCall: null,
      isStreaming: true,
      error: null
    }))
    
    // Start streaming
    try {
      const response = await fetch(
        `${API_BASE}/threads/${activeThread._id}/messages`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Accept': 'text/event-stream'
          },
          body: JSON.stringify({
            user_id: USER_ID,
            content,
            llm_config: llmConfig || defaultConfig
          })
        }
      )
      
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      
      const reader = response.body?.getReader()
      const decoder = new TextDecoder()
      
      if (!reader) throw new Error('No reader available')
      
      const processor = streamProcessorRef.current
      processor.reset()
      
      // Process stream with state updates
      let currentState = chatState
      currentState = {
        ...currentState,
        items: [...currentState.items, userMessage],
        isStreaming: true
      }
      
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        
        const chunk = decoder.decode(value, { stream: true })
        
        // Process chunk and get state updates
        const updates = processor.processChunk(chunk, currentState)
        
        // Apply each update immediately for real-time display
        for (const update of updates) {
          currentState = update
          setChatState(currentState)
          
          // Small delay for smoother animation (optional)
          if (updates.length > 1) {
            await new Promise(resolve => setTimeout(resolve, 10))
          }
        }
      }
    } catch (error) {
      console.error('Error sending message:', error)
      setChatState(prev => ({
        ...prev,
        error: 'Failed to send message',
        isStreaming: false
      }))
    } finally {
      setChatState(prev => ({
        ...prev,
        isStreaming: false
      }))
    }
  }, [thread, chatState, onThreadCreated, onThreadUpdate])
  
  /**
   * Clear chat state (when switching threads)
   */
  const clearChat = useCallback(() => {
    setChatState({
      items: [],
      streamingMessage: null,
      streamingToolCall: null,
      isStreaming: false,
      error: null
    })
  }, [])
  
  return {
    chatState,
    sendMessage,
    loadMessages,
    clearChat
  }
}
