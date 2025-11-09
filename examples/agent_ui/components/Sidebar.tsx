import { Thread } from '@/types'
import { PlusIcon } from './icons/PlusIcon'

interface SidebarProps {
  threads: Thread[]
  currentThread: Thread | null
  onSelectThread: (thread: Thread) => void
  onCreateThread: () => void
  onDeleteThread: (threadId: string) => void
  isLoading: boolean
}

export default function Sidebar({
  threads,
  currentThread,
  onSelectThread,
  onCreateThread,
  isLoading
}: SidebarProps) {
  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('pt-BR', {
      day: '2-digit',
      month: 'short'
    })
  }

  return (
    <aside className="w-80 bg-praxis-bg-secondary border-r border-praxis-border flex flex-col">
      {/* Header */}
      <div className="p-5 flex items-center justify-between border-b border-praxis-border">
        <h1 className="text-xl font-semibold bg-gradient-to-r from-praxis-accent to-purple-500 bg-clip-text text-transparent">
          Praxis
        </h1>
        <button
          onClick={onCreateThread}
          className="p-2 bg-praxis-bg-tertiary border border-praxis-border rounded-lg hover:bg-praxis-bg-hover hover:border-praxis-accent transition-all"
          title="Nova conversa"
        >
          <PlusIcon className="w-5 h-5" />
        </button>
      </div>

      {/* Threads List */}
      <div className="flex-1 overflow-y-auto scrollbar-thin p-3">
        {isLoading ? (
          <div className="text-center py-8 text-praxis-text-muted text-sm">
            Carregando conversas...
          </div>
        ) : threads.length === 0 ? (
          <div className="text-center py-8 text-praxis-text-muted text-sm">
            Nenhuma conversa ainda
          </div>
        ) : (
          threads.map((thread) => (
            <button
              key={thread._id}
              onClick={() => onSelectThread(thread)}
              className={`w-full text-left p-3 mb-2 rounded-lg border transition-all ${
                currentThread?._id === thread._id
                  ? 'bg-praxis-accent border-praxis-accent text-white'
                  : 'bg-praxis-bg-tertiary border-praxis-border hover:bg-praxis-bg-hover hover:border-praxis-accent'
              }`}
            >
              <div className="font-medium text-sm mb-1 truncate">
                {thread.title}
              </div>
              <div className={`text-xs ${
                currentThread?._id === thread._id
                  ? 'text-white/70'
                  : 'text-praxis-text-muted'
              }`}>
                {formatDate(thread.created_at)}
              </div>
            </button>
          ))
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-praxis-border">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-praxis-accent flex items-center justify-center text-sm font-semibold">
            U
          </div>
          <span className="text-sm text-praxis-text-secondary">test_user</span>
        </div>
      </div>
    </aside>
  )
}

