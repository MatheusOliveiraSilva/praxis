'use client'

interface ModelConfig {
  model: string
  temperature: number
  max_tokens: number
  reasoning_effort?: string
}

interface ModelSelectorProps {
  value: ModelConfig
  onChange: (config: ModelConfig) => void
  disabled?: boolean
}

const MODELS = [
  { value: 'gpt-4o-mini', label: 'GPT-4o Mini', isO1: false },
  { value: 'gpt-4o', label: 'GPT-4o', isO1: false },
  { value: 'gpt-5', label: 'GPT-5', isO1: true },
]

const REASONING_EFFORTS = ['low', 'medium', 'high']

export default function ModelSelector({ value, onChange, disabled }: ModelSelectorProps) {
  const selectedModel = MODELS.find(m => m.value === value.model) || MODELS[0]
  const isO1Model = selectedModel.isO1

  return (
    <div className="flex gap-2 items-center flex-wrap">
      {/* Model selector */}
      <select
        value={value.model}
        onChange={(e) => onChange({ ...value, model: e.target.value })}
        disabled={disabled}
        className="px-3 py-1.5 text-sm bg-praxis-bg-tertiary border border-praxis-border text-praxis-text-primary rounded-lg focus:outline-none focus:border-praxis-accent disabled:opacity-50"
      >
        {MODELS.map(model => (
          <option key={model.value} value={model.value}>
            {model.label}
          </option>
        ))}
      </select>

      {/* Reasoning effort (only for o1 models) */}
      {isO1Model && (
        <select
          value={value.reasoning_effort || 'medium'}
          onChange={(e) => onChange({ ...value, reasoning_effort: e.target.value })}
          disabled={disabled}
          className="px-3 py-1.5 text-sm bg-praxis-bg-tertiary border border-praxis-border text-praxis-text-primary rounded-lg focus:outline-none focus:border-praxis-accent disabled:opacity-50"
          title="Reasoning Effort"
        >
          {REASONING_EFFORTS.map(effort => (
            <option key={effort} value={effort}>
              {effort.charAt(0).toUpperCase() + effort.slice(1)} Effort
            </option>
          ))}
        </select>
      )}

      {/* Temperature (only for non-o1 models) */}
      {!isO1Model && (
        <div className="flex items-center gap-2">
          <span className="text-xs text-praxis-text-secondary">Temp:</span>
          <input
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={value.temperature}
            onChange={(e) => onChange({ ...value, temperature: parseFloat(e.target.value) })}
            disabled={disabled}
            className="w-20 accent-praxis-accent disabled:opacity-50"
          />
          <span className="text-xs text-praxis-text-secondary w-8">{value.temperature.toFixed(1)}</span>
        </div>
      )}
    </div>
  )
}

