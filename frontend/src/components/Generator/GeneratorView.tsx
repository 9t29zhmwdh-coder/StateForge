import { useState, useEffect } from 'react'
import { useMachineStore } from '../../stores/machineStore'
import { api } from '../../lib/tauri'

const TARGETS = [
  { id: 'swift',      label: 'Swift',      desc: 'enum + Reducer' },
  { id: 'kotlin',     label: 'Kotlin',     desc: 'sealed class + ViewModel' },
  { id: 'typescript', label: 'TypeScript', desc: 'XState + Redux' },
  { id: 'go',         label: 'Go',         desc: 'iota + FSM struct' },
  { id: 'rust',       label: 'Rust',       desc: 'enum + match' },
]

export function GeneratorView() {
  const { active } = useMachineStore()
  const [target, setTarget] = useState('typescript')
  const [code, setCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [copied, setCopied] = useState(false)

  const generate = async () => {
    if (!active) return
    setLoading(true)
    try {
      const result = await api.generateCode(active.id, target)
      setCode(result)
    } catch (e) {
      setCode(`// Fehler: ${String(e)}`)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (active) generate()
  }, [active?.id, target])

  const handleCopy = () => {
    navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleDownload = () => {
    if (!active) return
    const exts: Record<string, string> = {
      swift: 'swift', kotlin: 'kt', typescript: 'ts', go: 'go', rust: 'rs',
    }
    const blob = new Blob([code], { type: 'text/plain' })
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = `${active.name}.${exts[target] ?? 'txt'}`
    a.click()
  }

  if (!active) {
    return (
      <div className="flex items-center justify-center h-full text-[#8b949e]">
        Keine State Machine ausgewählt
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Target selector */}
      <div className="flex items-center gap-4 px-6 py-3 border-b border-[#30363d]">
        <span className="text-xs text-[#8b949e] uppercase tracking-wider">Zielsprache:</span>
        <div className="flex gap-2">
          {TARGETS.map(t => (
            <button
              key={t.id}
              onClick={() => setTarget(t.id)}
              title={t.desc}
              className={`px-3 py-1 text-xs rounded transition-colors ${
                target === t.id
                  ? 'bg-[#1f6feb] text-white'
                  : 'text-[#8b949e] hover:text-[#e6edf3] bg-[#161b22] hover:bg-[#21262d]'
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>
        <div className="ml-auto flex gap-2">
          <button
            onClick={handleCopy}
            className="px-3 py-1 text-xs text-[#8b949e] hover:text-[#e6edf3] bg-[#161b22] hover:bg-[#21262d] rounded transition-colors"
          >
            {copied ? 'Kopiert!' : 'Kopieren'}
          </button>
          <button
            onClick={handleDownload}
            className="px-3 py-1 text-xs text-[#8b949e] hover:text-[#e6edf3] bg-[#161b22] hover:bg-[#21262d] rounded transition-colors"
          >
            Download
          </button>
        </div>
      </div>

      {/* Code output */}
      <div className="flex-1 overflow-auto">
        {loading ? (
          <div className="flex items-center justify-center h-full text-[#8b949e] text-sm">Generiere…</div>
        ) : (
          <pre className="p-6 text-sm font-mono text-[#e6edf3] whitespace-pre-wrap leading-relaxed">
            {code || '// Code wird generiert…'}
          </pre>
        )}
      </div>
    </div>
  )
}
