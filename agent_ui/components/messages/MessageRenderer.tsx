import { UIMessage } from '@/types/events'
import { UserMessage } from './UserMessage'
import { AssistantMessage } from './AssistantMessage'
import { ReasoningMessage } from './ReasoningMessage'
import { memo } from 'react'

interface MessageRendererProps {
  message: UIMessage
}

/**
 * Message renderer component
 * Uses Strategy Pattern to render different message types
 * Memoized for optimal re-render performance
 */
export const MessageRenderer = memo(({ message }: MessageRendererProps) => {
  switch (message.type) {
    case 'user':
      return <UserMessage message={message} />
    case 'assistant':
      return <AssistantMessage message={message} />
    case 'reasoning':
      return <ReasoningMessage message={message} />
    default:
      return null
  }
})

MessageRenderer.displayName = 'MessageRenderer'

