export default function TypingIndicator() {
  return (
    <div className="mb-6 flex justify-start">
      <div className="max-w-[80%]">
        <div className="flex items-center gap-2 mb-2">
          <span className="text-xs font-semibold uppercase tracking-wide text-praxis-text-muted">
            Assistente
          </span>
        </div>
        <div className="px-4 py-3 bg-praxis-bg-secondary border border-praxis-border rounded-2xl flex gap-1">
          <span className="w-2 h-2 bg-praxis-text-muted rounded-full animate-bounce" style={{ animationDelay: '0ms' }}></span>
          <span className="w-2 h-2 bg-praxis-text-muted rounded-full animate-bounce" style={{ animationDelay: '150ms' }}></span>
          <span className="w-2 h-2 bg-praxis-text-muted rounded-full animate-bounce" style={{ animationDelay: '300ms' }}></span>
        </div>
      </div>
    </div>
  )
}

