'use client'

import { useState, useEffect } from 'react'
import Sidebar from '@/components/Sidebar'
import ChatArea from '@/components/ChatArea'
import { Thread } from '@/types'

const API_BASE = 'http://localhost:8000'
const USER_ID = 'test_user'

export default function Home() {
  const [threads, setThreads] = useState<Thread[]>([])
  const [currentThread, setCurrentThread] = useState<Thread | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    loadThreads()
  }, [])

  const loadThreads = async () => {
    try {
      const response = await fetch(`${API_BASE}/threads?user_id=${USER_ID}`)
      const data = await response.json()
      setThreads(data.threads || [])
    } catch (error) {
      console.error('Error loading threads:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const createThread = async () => {
    try {
      const response = await fetch(`${API_BASE}/threads`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: USER_ID,
          title: 'Nova Conversa'
        })
      })
      const data = await response.json()
      await loadThreads()
      setCurrentThread({
        _id: data.thread_id,
        user_id: USER_ID,
        title: 'Nova Conversa',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      })
    } catch (error) {
      console.error('Error creating thread:', error)
    }
  }

  const deleteThread = async (threadId: string) => {
    try {
      await fetch(`${API_BASE}/threads/${threadId}?user_id=${USER_ID}`, {
        method: 'DELETE'
      })
      await loadThreads()
      if (currentThread?._id === threadId) {
        setCurrentThread(null)
      }
    } catch (error) {
      console.error('Error deleting thread:', error)
    }
  }

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar
        threads={threads}
        currentThread={currentThread}
        onSelectThread={setCurrentThread}
        onCreateThread={createThread}
        onDeleteThread={deleteThread}
        isLoading={isLoading}
      />
      <ChatArea
        thread={currentThread}
        onThreadUpdate={loadThreads}
        onThreadCreated={setCurrentThread}
      />
    </div>
  )
}

