import { UIToolCall } from '@/types/events'
import { CogIcon } from '../icons/CogIcon'
import { CheckIcon } from '../icons/CheckIcon'
import { memo } from 'react'

interface ToolCallRendererProps {
  toolCall: UIToolCall
}

/**
 * Tool call renderer component (memoized)
 * Displays tool execution status and results
 */
export const ToolCallRenderer = memo(({ toolCall }: ToolCallRendererProps) => {
  const statusColors = {
    running: 'bg-praxis-warning/15 text-praxis-warning',
    completed: 'bg-praxis-success/15 text-praxis-success',
    error: 'bg-praxis-error/15 text-praxis-error'
  }

  return (
    <div className="mb-4 max-w-[80%]">
      <div className="bg-praxis-bg-tertiary border border-praxis-border rounded-xl p-4">
        <div className="flex items-center gap-3 mb-3">
          {toolCall.status === 'running' ? (
            <CogIcon className="w-5 h-5 text-praxis-warning animate-spin" />
          ) : (
            <CheckIcon className="w-5 h-5 text-praxis-success" />
          )}
          <span className="font-semibold text-sm text-praxis-text-primary flex-1">
            {toolCall.name || 'Carregando...'}
          </span>
          <span
            className={`text-xs font-medium px-2 py-1 rounded ${
              statusColors[toolCall.status]
            }`}
          >
            {toolCall.status === 'running' && 'Executando'}
            {toolCall.status === 'completed' && 'Completo'}
            {toolCall.status === 'error' && 'Erro'}
          </span>
        </div>

        {toolCall.arguments && (
          <div className="mb-2 p-2 bg-praxis-bg-primary rounded text-xs font-mono text-praxis-text-secondary overflow-x-auto">
            {toolCall.arguments}
          </div>
        )}

        {toolCall.result && (
          <div className="p-3 bg-praxis-bg-primary rounded text-sm text-praxis-text-secondary max-h-48 overflow-y-auto scrollbar-thin">
            {(() => {
              try {
                const parsed = JSON.parse(toolCall.result)
                return parsed.text || toolCall.result
              } catch {
                return toolCall.result.substring(0, 200) +
                  (toolCall.result.length > 200 ? '...' : '')
              }
            })()}
          </div>
        )}
      </div>
    </div>
  )
})

ToolCallRenderer.displayName = 'ToolCallRenderer'

