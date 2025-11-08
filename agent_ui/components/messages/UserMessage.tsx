import { UIMessage } from '@/types/events'
import { memo } from 'react'

interface UserMessageProps {
  message: UIMessage
}

/**
 * User message component (memoized for performance)
 * Only re-renders when message content changes
 */
export const UserMessage = memo(({ message }: UserMessageProps) => {
  return (
    <div className="mb-6 flex justify-end">
      <div className="max-w-[80%] ml-auto">
        <div className="flex items-center gap-2 mb-2 justify-end">
          <span className="text-xs font-semibold uppercase tracking-wide text-praxis-text-muted">
            Você
          </span>
        </div>
        <div className="px-4 py-3 rounded-2xl bg-praxis-accent text-white">
          {message.content}
        </div>
      </div>
    </div>
  )
})

UserMessage.displayName = 'UserMessage'

