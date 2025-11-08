import { UIMessage } from '@/types/events'
import { memo } from 'react'

interface ReasoningMessageProps {
  message: UIMessage
}

/**
 * Reasoning message component (memoized)
 * Displays internal reasoning from o1/o3 models
 */
export const ReasoningMessage = memo(({ message }: ReasoningMessageProps) => {
  return (
    <div className="mb-6 flex justify-start">
      <div className="max-w-[80%] mr-auto">
        <div className="flex items-center gap-2 mb-2">
          <svg className="w-4 h-4 text-praxis-warning" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          <span className="text-xs font-semibold uppercase tracking-wide text-praxis-warning">
            Reasoning
          </span>
        </div>
        <div className="px-4 py-3 rounded-2xl whitespace-pre-wrap break-words bg-praxis-warning/10 border border-praxis-warning/30 text-praxis-text-secondary text-sm italic">
          {message.content}
        </div>
      </div>
    </div>
  )
})

ReasoningMessage.displayName = 'ReasoningMessage'

