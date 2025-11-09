import { UIMessage } from '@/types/events'
import { memo } from 'react'

interface AssistantMessageProps {
  message: UIMessage
}

/**
 * Assistant message component (memoized)
 * Renders regular assistant responses
 */
export const AssistantMessage = memo(({ message }: AssistantMessageProps) => {
  return (
    <div className="mb-6 flex justify-start">
      <div className="max-w-[80%] mr-auto">
        <div className="flex items-center gap-2 mb-2">
          <span className="text-xs font-semibold uppercase tracking-wide text-praxis-text-muted">
            Assistente
          </span>
        </div>
        <div className="px-4 py-3 rounded-2xl whitespace-pre-wrap break-words bg-praxis-bg-secondary border border-praxis-border text-praxis-text-primary">
          {message.content}
        </div>
      </div>
    </div>
  )
})

AssistantMessage.displayName = 'AssistantMessage'

