'use client'

import { useState, useEffect, useRef } from 'react'
import { Thread, Message, ToolCall } from '@/types'
import MessageBubble from './MessageBubble'
import ToolCallCard from './ToolCallCard'
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

export default function ChatArea({ thread, onThreadUpdate, onThreadCreated }: ChatAreaProps) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isStreaming, setIsStreaming] = useState(false)
  const [currentAssistantMessage, setCurrentAssistantMessage] = useState('')
  const [toolCalls, setToolCalls] = useState<Map<number, ToolCall>>(new Map())
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (thread) {
      loadMessages()
    } else {
      setMessages([])
    }
  }, [thread])

  useEffect(() => {
    scrollToBottom()
  }, [messages, currentAssistantMessage, toolCalls])

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  const loadMessages = async () => {
    if (!thread) return

    try {
      const response = await fetch(
        `${API_BASE}/threads/${thread._id}/messages?user_id=${USER_ID}`
      )
      const data = await response.json()
      setMessages(data.messages || [])
    } catch (error) {
      console.error('Error loading messages:', error)
    }
  }

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim() || isStreaming) return

    const userMessage = input.trim()
    setInput('')
    setIsStreaming(true)
    setCurrentAssistantMessage('')
    setToolCalls(new Map())

    // Create thread if it doesn't exist
    let activeThread = thread
    if (!activeThread) {
      try {
        const response = await fetch(`${API_BASE}/threads`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            user_id: USER_ID,
            title: userMessage.substring(0, 50) // Use first 50 chars as title
          })
        })
        const data = await response.json()
        activeThread = {
          _id: data.thread_id,
          user_id: USER_ID,
          title: userMessage.substring(0, 50),
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
        // Set as current thread and update sidebar
        onThreadCreated(activeThread)
        onThreadUpdate()
      } catch (error) {
        console.error('Error creating thread:', error)
        setIsStreaming(false)
        alert('Erro ao criar conversa')
        return
      }
    }

    // Add user message optimistically
    const tempUserMsg: Message = {
      _id: Date.now().toString(),
      thread_id: activeThread._id,
      user_id: USER_ID,
      role: 'user',
      content: userMessage,
      created_at: new Date().toISOString()
    }
    setMessages(prev => [...prev, tempUserMsg])

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
            content: userMessage
          })
        }
      )

      if (!response.ok) throw new Error(`HTTP ${response.status}`)

      const reader = response.body?.getReader()
      const decoder = new TextDecoder()
      let buffer = ''

      if (!reader) throw new Error('No reader available')

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.substring(6).trim()
            if (!data) continue

            try {
              const parsed = JSON.parse(data)

              // Handle message chunks
              if (parsed.content !== undefined) {
                setCurrentAssistantMessage(prev => prev + parsed.content)
              }

              // Handle tool calls
              if (parsed.name !== undefined || parsed.arguments !== undefined) {
                const index = parsed.index || 0
                setToolCalls(prev => {
                  const newMap = new Map(prev)
                  const existing = newMap.get(index) || {
                    index,
                    name: '',
                    arguments: '',
                    status: 'running' as const
                  }

                  newMap.set(index, {
                    ...existing,
                    name: parsed.name || existing.name,
                    arguments: existing.arguments + (parsed.arguments || '')
                  })

                  return newMap
                })
              }

              // Handle tool results
              if (parsed.result !== undefined) {
                setToolCalls(prev => {
                  const newMap = new Map(prev)
                  newMap.forEach((tool, index) => {
                    newMap.set(index, {
                      ...tool,
                      status: 'completed',
                      result: parsed.result
                    })
                  })
                  return newMap
                })
              }
            } catch (err) {
              console.error('Error parsing SSE data:', err)
            }
          }
        }
      }

      // Add the complete assistant message to the list before clearing
      if (currentAssistantMessage) {
        const assistantMsg: Message = {
          _id: (Date.now() + 1).toString(),
          thread_id: activeThread._id,
          user_id: 'assistant',
          role: 'assistant',
          content: currentAssistantMessage,
          created_at: new Date().toISOString()
        }
        setMessages(prev => [...prev, assistantMsg])
      }
      
      // Reload messages after stream completes (in background)
      loadMessages()
    } catch (error) {
      console.error('Error sending message:', error)
      alert('Erro ao enviar mensagem. Verifique se a API está rodando.')
    } finally {
      setIsStreaming(false)
      setCurrentAssistantMessage('')
      setToolCalls(new Map())
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

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend(e)
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
        {!thread && messages.length === 0 && !isStreaming && (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <MessageSquareIcon className="w-20 h-20 mb-6 text-praxis-accent opacity-60" />
            <h2 className="text-2xl font-semibold mb-3">Bem-vindo ao Praxis</h2>
            <p className="text-praxis-text-secondary max-w-md">
              Digite sua mensagem abaixo para começar uma nova conversa com o assistente AI
            </p>
          </div>
        )}

        {messages.map((msg) => (
          <MessageBubble key={msg._id} message={msg} />
        ))}

        {/* Tool Calls */}
        {Array.from(toolCalls.values()).map((tool) => (
          <ToolCallCard key={tool.index} toolCall={tool} />
        ))}

        {/* Streaming Assistant Message */}
        {currentAssistantMessage && (
          <MessageBubble
            message={{
              _id: 'streaming',
              thread_id: thread._id,
              user_id: 'assistant',
              role: 'assistant',
              content: currentAssistantMessage,
              created_at: new Date().toISOString()
            }}
          />
        )}

        {/* Typing Indicator */}
        {isStreaming && !currentAssistantMessage && toolCalls.size === 0 && (
          <TypingIndicator />
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
            disabled={isStreaming}
            className="flex-1 bg-praxis-bg-tertiary border border-praxis-border text-praxis-text-primary px-4 py-3 rounded-xl focus:outline-none focus:border-praxis-accent resize-none max-h-32 disabled:opacity-50"
            rows={1}
          />
          <button
            type="submit"
            disabled={!input.trim() || isStreaming}
            className="p-3 bg-praxis-accent text-white rounded-xl hover:bg-praxis-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-all"
          >
            <SendIcon className="w-5 h-5" />
          </button>
        </form>
      </div>
    </main>
  )
}

