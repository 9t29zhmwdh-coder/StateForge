import { useState, useEffect } from 'react'
import { useMachineStore } from '../../stores/machineStore'
import { api, type MachineAnalysis } from '../../lib/tauri'

export function AnalysisView() {
  const { active, updateMachine } = useMachineStore()
  const [analysis, setAnalysis] = useState<MachineAnalysis | null>(null)
  const [enhancing, setEnhancing] = useState(false)
  const [error, setError] = useState('')

  useEffect(() => {
    if (!active) return
    api.analyzeMachine(active).then(setAnalysis).catch(console.error)
  }, [active?.id])

  const handleEnhance = async () => {
    if (!active) return
    setEnhancing(true); setError('')
    try {
      const m = await api.aiEnhanceMachine(active.id)
      updateMachine(m)
    } catch (e) {
      setError(String(e))
    } finally {
      setEnhancing(false)
    }
  }

  if (!active) {
    return (
      <div className="flex items-center justify-center h-full text-[#8b949e]">
        Keine State Machine ausgewählt
      </div>
    )
  }

  return (
    <div className="p-6 max-w-2xl mx-auto overflow-y-auto h-full">
      <h2 className="text-lg font-semibold text-[#e6edf3] mb-6">{active.name}</h2>

      {/* Stats */}
      {analysis && (
        <div className="grid grid-cols-2 gap-3 mb-6">
          <Stat label="Zustände" value={analysis.state_count} />
          <Stat label="Transitionen" value={analysis.transition_count} />
          <Stat label="Deterministisch" value={analysis.is_deterministic ? 'Ja' : 'Nein'}
            color={analysis.is_deterministic ? 'text-[#3fb950]' : 'text-[#f85149]'} />
          <Stat label="Initialzustand" value={analysis.has_initial_state ? 'Vorhanden' : 'Fehlt'}
            color={analysis.has_initial_state ? 'text-[#3fb950]' : 'text-[#d29922]'} />
        </div>
      )}

      {/* States list */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold text-[#8b949e] uppercase tracking-wider mb-3">Zustände</h3>
        <div className="space-y-1">
          {active.states.map(s => (
            <div key={s.id} className="flex items-center gap-3 px-3 py-2 bg-[#161b22] rounded-md border border-[#30363d]">
              <span className={`w-2 h-2 rounded-full flex-shrink-0 ${kindColor(s.kind)}`} />
              <span className="text-sm font-mono text-[#e6edf3]">{s.name}</span>
              <span className="text-xs text-[#8b949e] capitalize ml-auto">{s.kind}</span>
              {s.description && (
                <span className="text-xs text-[#8b949e] italic truncate max-w-[200px]">{s.description}</span>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Transitions list */}
      <section className="mb-6">
        <h3 className="text-sm font-semibold text-[#8b949e] uppercase tracking-wider mb-3">Transitionen</h3>
        <div className="space-y-1">
          {active.transitions.map(t => (
            <div key={t.id} className="flex items-center gap-2 px-3 py-2 bg-[#161b22] rounded-md border border-[#30363d] text-sm font-mono">
              <span className="text-[#8b949e]">{t.from_state}</span>
              <span className="text-[#30363d]">→</span>
              <span className="text-[#e6edf3]">{t.to_state}</span>
              {t.event && <span className="ml-auto text-xs text-[#58a6ff]">[{t.event}]</span>}
              {t.guard && <span className="text-xs text-[#d29922]">{'{'}guard{'}'}</span>}
            </div>
          ))}
        </div>
      </section>

      {/* Warnings */}
      {analysis && analysis.unreachable_states.length > 0 && (
        <section className="mb-6 p-3 bg-[#2d1b00] border border-[#d29922] rounded-md">
          <div className="text-xs font-semibold text-[#d29922] mb-2">Nicht erreichbare Zustände</div>
          {analysis.unreachable_states.map(s => (
            <div key={s} className="text-xs font-mono text-[#e6edf3]">{s}</div>
          ))}
        </section>
      )}

      {/* AI Summary */}
      {active.ai_summary && (
        <section className="mb-6 p-4 bg-[#161b22] border border-[#30363d] rounded-md">
          <div className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider mb-2">AI-Zusammenfassung</div>
          <p className="text-sm text-[#e6edf3]">{active.ai_summary}</p>
        </section>
      )}

      {error && <div className="text-xs text-[#f85149] mb-4">{error}</div>}

      <button
        onClick={handleEnhance}
        disabled={enhancing}
        className="w-full py-2.5 bg-[#6e40c9] hover:bg-[#8957e5] disabled:opacity-50 text-white text-sm rounded-md transition-colors"
      >
        {enhancing ? 'AI analysiert…' : 'Mit AI anreichern'}
      </button>
    </div>
  )
}

function Stat({ label, value, color = 'text-[#e6edf3]' }: { label: string; value: string | number; color?: string }) {
  return (
    <div className="bg-[#161b22] border border-[#30363d] rounded-md px-4 py-3">
      <div className="text-xs text-[#8b949e] mb-1">{label}</div>
      <div className={`text-xl font-bold ${color}`}>{value}</div>
    </div>
  )
}

function kindColor(kind: string) {
  if (kind === 'initial')  return 'bg-[#3fb950]'
  if (kind === 'final')    return 'bg-[#58a6ff]'
  if (kind === 'error')    return 'bg-[#f85149]'
  if (kind === 'parallel') return 'bg-[#d2a8ff]'
  return 'bg-[#8b949e]'
}
