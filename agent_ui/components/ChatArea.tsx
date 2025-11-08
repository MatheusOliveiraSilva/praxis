'use client'

import { useState, useEffect, useRef, KeyboardEvent } from 'react'
import { Thread } from '@/types'
import { UIMessage, UIToolCall } from '@/types/events'
import { useChat } from '@/hooks/useChat'
import { MessageRenderer } from './messages/MessageRenderer'
import { ToolCallRenderer } from './tools/ToolCallRenderer'
import TypingIndicator from './TypingIndicator'
import { TrashIcon } from './icons/TrashIcon'
import { SendIcon } from './icons/SendIcon'
import { MessageSquareIcon } from './icons/MessageSquareIcon'

const API_BASE = 'http://localhost:8000'
const USER_ID = 'test_user'

interface ChatAreaProps {
  thread: Thread | null
  onThreadUpdate: () => void
  onThreadCreated: (thread: Thread) => void
}

/**
 * Main chat area component
 * Handles message display, input, and thread management
 */
export default function ChatArea({ thread, onThreadUpdate, onThreadCreated }: ChatAreaProps) {
  const [input, setInput] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  
  // Use custom hook for chat functionality
  const { chatState, sendMessage, loadMessages, clearChat } = useChat({
    thread,
    onThreadCreated,
    onThreadUpdate
  })
  
  // Load messages when thread changes
  useEffect(() => {
    if (thread) {
      loadMessages(thread._id)
    } else {
      clearChat()
    }
  }, [thread, loadMessages, clearChat])
  
  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [chatState.items, chatState.streamingMessage, chatState.streamingToolCall])
  
  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim() || chatState.isStreaming) return
    
    const message = input.trim()
    setInput('')
    
    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto'
    }
    
    await sendMessage(message)
  }
  
  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend(e)
    }
  }
  
  const handleDelete = async () => {
    if (!thread || !confirm('Tem certeza que deseja excluir esta conversa?')) {
      return
    }
    
    try {
      await fetch(`${API_BASE}/threads/${thread._id}?user_id=${USER_ID}`, {
        method: 'DELETE'
      })
      onThreadUpdate()
    } catch (error) {
      console.error('Error deleting thread:', error)
    }
  }
  
  return (
    <main className="flex-1 flex flex-col bg-praxis-bg-primary">
      {/* Header */}
      {thread ? (
        <div className="px-6 py-4 border-b border-praxis-border flex items-center justify-between">
          <h2 className="text-lg font-medium">{thread.title}</h2>
          <button
            onClick={handleDelete}
            className="p-2 text-praxis-text-secondary hover:text-praxis-error hover:bg-praxis-error/10 border border-praxis-border hover:border-praxis-error rounded-lg transition-all"
            title="Excluir conversa"
          >
            <TrashIcon className="w-4 h-4" />
          </button>
        </div>
      ) : (
        <div className="px-6 py-4 border-b border-praxis-border">
          <h2 className="text-lg font-medium text-praxis-text-secondary">Nova Conversa</h2>
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto scrollbar-thin px-6 py-6">
        {/* Welcome screen (only if no thread and no items) */}
        {!thread && chatState.items.length === 0 && !chatState.isStreaming && (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <MessageSquareIcon className="w-20 h-20 mb-6 text-praxis-accent opacity-60" />
            <h2 className="text-2xl font-semibold mb-3">Bem-vindo ao Praxis</h2>
            <p className="text-praxis-text-secondary max-w-md">
              Digite sua mensagem abaixo para começar uma nova conversa com o assistente AI
            </p>
          </div>
        )}

        {/* Render all items in chronological order */}
        {chatState.items.map((item) => {
          if (item.itemType === 'message') {
            return <MessageRenderer key={item.id} message={item as UIMessage} />
          } else {
            return <ToolCallRenderer key={item.id} toolCall={item as UIToolCall} />
          }
        })}

        {/* Render streaming tool call */}
        {chatState.streamingToolCall && (
          <ToolCallRenderer key="streaming-tool" toolCall={chatState.streamingToolCall} />
        )}

        {/* Render streaming message */}
        {chatState.streamingMessage && chatState.streamingMessage.content && (
          <MessageRenderer 
            key="streaming-message" 
            message={{
              itemType: 'message',
              id: 'streaming',
              type: chatState.streamingMessage.type,
              content: chatState.streamingMessage.content,
              timestamp: new Date()
            }}
          />
        )}

        {/* Typing indicator (only if nothing is streaming yet) */}
        {chatState.isStreaming && 
         !chatState.streamingMessage && 
         !chatState.streamingToolCall &&
         chatState.items.length > 0 && 
         chatState.items[chatState.items.length - 1].itemType === 'message' &&
         (chatState.items[chatState.items.length - 1] as UIMessage).type === 'user' && (
          <TypingIndicator />
        )}

        {/* Error display */}
        {chatState.error && (
          <div className="p-4 bg-praxis-error/10 border border-praxis-error/30 rounded-lg text-praxis-error text-sm">
            {chatState.error}
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="p-6 border-t border-praxis-border bg-praxis-bg-secondary">
        <form onSubmit={handleSend} className="flex gap-3 items-end">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={thread ? "Digite sua mensagem..." : "Digite sua primeira mensagem para começar..."}
            disabled={chatState.isStreaming}
            className="flex-1 bg-praxis-bg-tertiary border border-praxis-border text-praxis-text-primary px-4 py-3 rounded-xl focus:outline-none focus:border-praxis-accent resize-none max-h-32 disabled:opacity-50"
            rows={1}
          />
          <button
            type="submit"
            disabled={!input.trim() || chatState.isStreaming}
            className="p-3 bg-praxis-accent text-white rounded-xl hover:bg-praxis-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-all"
          >
            <SendIcon className="w-5 h-5" />
          </button>
        </form>
      </div>
    </main>
  )
}
