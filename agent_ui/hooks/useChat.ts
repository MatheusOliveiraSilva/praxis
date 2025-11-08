import { useState, useCallback, useRef } from 'react'
import { ChatState, UIMessage } from '@/types/events'
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
    messages: [],
    streamingMessage: null,
    toolCalls: [],
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
      
      // Convert DB messages to UI messages
      const uiMessages: UIMessage[] = (data.messages || []).map((msg: Message) => ({
        id: msg._id,
        type: msg.role === 'user' ? 'user' as const : 'assistant' as const,
        content: msg.content,
        timestamp: new Date(msg.created_at)
      }))
      
      setChatState(prev => ({
        ...prev,
        messages: uiMessages,
        streamingMessage: null,
        toolCalls: [],
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
  const sendMessage = useCallback(async (content: string) => {
    if (!content.trim() || chatState.isStreaming) return
    
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
      id: `${Date.now()}-user`,
      type: 'user',
      content,
      timestamp: new Date()
    }
    
    setChatState(prev => ({
      ...prev,
      messages: [...prev.messages, userMessage],
      streamingMessage: null,
      toolCalls: [],
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
            content
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
        messages: [...currentState.messages, userMessage],
        isStreaming: true
      }
      
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        
        const chunk = decoder.decode(value, { stream: true })
        
        // Process chunk and get state updates
        const updates = processor.processChunk(chunk, currentState)
        
        // Apply updates sequentially
        if (updates.length > 0) {
          currentState = updates[updates.length - 1]
          setChatState(currentState)
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
      messages: [],
      streamingMessage: null,
      toolCalls: [],
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
