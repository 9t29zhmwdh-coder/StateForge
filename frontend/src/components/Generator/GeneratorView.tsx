import { useState, useEffect } from 'react'
import { useMachineStore } from '../../stores/machineStore'
import { api } from '../../lib/tauri'
import { useT } from '../../lib/i18n'

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
  const t = useT()

  const generate = async () => {
    if (!active) return
    setLoading(true)
    try {
      const result = await api.generateCode(active.id, target)
      setCode(result)
    } catch (e) {
      setCode(`${t('generator.errorPrefix')} ${String(e)}`)
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
      <div className="flex items-center justify-center h-full text-sf-muted">
        {t('generator.noMachineSelected')}
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Target selector */}
      <div className="flex items-center gap-4 px-6 py-3 border-b border-sf-border">
        <span className="text-xs text-sf-muted uppercase tracking-wider">{t('generator.targetLanguage')}</span>
        <div className="flex gap-2">
          {TARGETS.map(target_ => (
            <button
              key={target_.id}
              onClick={() => setTarget(target_.id)}
              title={target_.desc}
              className={`px-3 py-1 text-xs rounded transition-colors ${
                target === target_.id
                  ? 'bg-[#1f6feb] text-white'
                  : 'text-sf-muted hover:text-[#e6edf3] bg-sf-surface hover:bg-[#21262d]'
              }`}
            >
              {target_.label}
            </button>
          ))}
        </div>
        <div className="ml-auto flex gap-2">
          <button
            onClick={handleCopy}
            className="px-3 py-1 text-xs text-sf-muted hover:text-[#e6edf3] bg-sf-surface hover:bg-[#21262d] rounded-sm transition-colors"
          >
            {copied ? t('generator.copied') : t('generator.copy')}
          </button>
          <button
            onClick={handleDownload}
            className="px-3 py-1 text-xs text-sf-muted hover:text-[#e6edf3] bg-sf-surface hover:bg-[#21262d] rounded-sm transition-colors"
          >
            {t('generator.download')}
          </button>
        </div>
      </div>

      {/* Code output */}
      <div className="flex-1 overflow-auto">
        {loading ? (
          <div className="flex items-center justify-center h-full text-sf-muted text-sm">{t('generator.generating')}</div>
        ) : (
          <pre className="p-6 text-sm font-mono text-[#e6edf3] whitespace-pre-wrap leading-relaxed">
            {code || t('generator.generatingComment')}
          </pre>
        )}
      </div>
    </div>
  )
}
