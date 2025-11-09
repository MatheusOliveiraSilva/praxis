import { Message } from '@/types'

interface MessageBubbleProps {
  message: Message
}

export default function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user'

  return (
    <div className={`mb-6 flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-[80%] ${isUser ? 'ml-auto' : 'mr-auto'}`}>
        <div className="flex items-center gap-2 mb-2">
          <span className="text-xs font-semibold uppercase tracking-wide text-praxis-text-muted">
            {isUser ? 'Você' : 'Assistente'}
          </span>
        </div>
        <div
          className={`px-4 py-3 rounded-2xl whitespace-pre-wrap break-words ${
            isUser
              ? 'bg-praxis-accent text-white'
              : 'bg-praxis-bg-secondary border border-praxis-border text-praxis-text-primary'
          }`}
        >
          {message.content}
        </div>
      </div>
    </div>
  )
}

