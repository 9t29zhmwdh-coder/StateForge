import { useState } from 'react'
import { api, type StateMachine } from '../../lib/tauri'
import { useT } from '../../lib/i18n'

const LANGUAGES = ['swift', 'kotlin', 'typescript', 'go', 'rust'] as const
type Lang = typeof LANGUAGES[number]

interface Props {
  onParsed: (m: StateMachine) => void
}

export function CodePanel({ onParsed }: Props) {
  const [code, setCode] = useState('')
  const [lang, setLang] = useState<Lang>('typescript')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [aiDesc, setAiDesc] = useState('')
  const [mode, setMode] = useState<'code' | 'log' | 'ai'>('code')
  const t = useT()

  const detectLang = async (content: string) => {
    if (content.length < 20) return
    const detected = await api.detectLanguage(content).catch(() => null)
    if (detected && LANGUAGES.includes(detected as Lang)) setLang(detected as Lang)
  }

  const handleParse = async () => {
    if (!code.trim()) return
    setLoading(true); setError('')
    try {
      let m: StateMachine
      if (mode === 'log') {
        m = await api.parseLog(code)
      } else {
        m = await api.parseCode(code, lang)
      }
      onParsed(m)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const handleAi = async () => {
    if (!aiDesc.trim()) return
    setLoading(true); setError('')
    try {
      const m = await api.aiFromDescription(aiDesc)
      onParsed(m)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex flex-col h-full bg-sf-bg">
      {/* Mode tabs */}
      <div className="flex border-b border-sf-border">
        {(['code', 'log', 'ai'] as const).map(m => (
          <button
            key={m}
            onClick={() => setMode(m)}
            className={`px-4 py-2 text-xs font-medium capitalize transition-colors border-b-2 ${
              mode === m
                ? 'border-[#58a6ff] text-[#58a6ff]'
                : 'border-transparent text-sf-muted hover:text-[#e6edf3]'
            }`}
          >
            {m === 'code' ? t('codePanel.code') : m === 'log' ? t('codePanel.log') : t('codePanel.ai')}
          </button>
        ))}
      </div>

      {mode === 'ai' ? (
        <div className="flex flex-col gap-3 p-4 flex-1">
          <textarea
            value={aiDesc}
            onChange={e => setAiDesc(e.target.value)}
            placeholder={t('codePanel.aiPlaceholder')}
            className="flex-1 bg-sf-surface border border-sf-border rounded-md p-3 text-sm text-[#e6edf3] font-mono resize-none focus:outline-hidden focus:border-[#58a6ff] placeholder-[#484f58]"
          />
          {error && <div className="text-xs text-[#f85149] px-1">{error}</div>}
          <button
            onClick={handleAi}
            disabled={loading || !aiDesc.trim()}
            className="px-4 py-2 bg-[#6e40c9] hover:bg-[#8957e5] disabled:opacity-50 text-white text-sm rounded-md transition-colors"
          >
            {loading ? t('codePanel.generating') : t('codePanel.generateWithAi')}
          </button>
        </div>
      ) : (
        <div className="flex flex-col h-full">
          {mode === 'code' && (
            <div className="flex items-center gap-2 px-4 py-2 border-b border-sf-border">
              <span className="text-xs text-sf-muted">{t('codePanel.language')}</span>
              {LANGUAGES.map(l => (
                <button
                  key={l}
                  onClick={() => setLang(l)}
                  className={`px-2 py-0.5 text-xs rounded-sm capitalize transition-colors ${
                    lang === l
                      ? 'bg-[#1f6feb] text-white'
                      : 'text-sf-muted hover:text-[#e6edf3] hover:bg-[#21262d]'
                  }`}
                >
                  {l}
                </button>
              ))}
            </div>
          )}
          <textarea
            value={code}
            onChange={e => { setCode(e.target.value); detectLang(e.target.value) }}
            placeholder={mode === 'log' ? t('codePanel.logPlaceholder') : `${lang}${t('codePanel.codePlaceholderSuffix')}`}
            className="flex-1 bg-sf-bg border-0 p-4 text-sm text-[#e6edf3] font-mono resize-none focus:outline-hidden placeholder-[#484f58]"
            spellCheck={false}
          />
          {error && <div className="text-xs text-[#f85149] px-4 pb-2">{error}</div>}
          <div className="p-3 border-t border-sf-border">
            <button
              onClick={handleParse}
              disabled={loading || !code.trim()}
              className="w-full py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-md transition-colors"
            >
              {loading ? t('codePanel.analyzing') : t('codePanel.extractMachine')}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
