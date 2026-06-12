import { useState, useEffect } from 'react'
import { api, type StateMachine } from '../../lib/tauri'

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
    <div className="flex flex-col h-full bg-[#0d1117]">
      {/* Mode tabs */}
      <div className="flex border-b border-[#30363d]">
        {(['code', 'log', 'ai'] as const).map(m => (
          <button
            key={m}
            onClick={() => setMode(m)}
            className={`px-4 py-2 text-xs font-medium capitalize transition-colors border-b-2 ${
              mode === m
                ? 'border-[#58a6ff] text-[#58a6ff]'
                : 'border-transparent text-[#8b949e] hover:text-[#e6edf3]'
            }`}
          >
            {m === 'code' ? 'Quellcode' : m === 'log' ? 'Log-Datei' : 'AI-Beschreibung'}
          </button>
        ))}
      </div>

      {mode === 'ai' ? (
        <div className="flex flex-col gap-3 p-4 flex-1">
          <textarea
            value={aiDesc}
            onChange={e => setAiDesc(e.target.value)}
            placeholder="Beschreibe deine State Machine auf Deutsch oder Englisch…&#10;z.B.: Eine Authentifizierungsmaschine mit den Zuständen Idle, Loading, Authenticated und Error."
            className="flex-1 bg-[#161b22] border border-[#30363d] rounded-md p-3 text-sm text-[#e6edf3] font-mono resize-none focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
          />
          {error && <div className="text-xs text-[#f85149] px-1">{error}</div>}
          <button
            onClick={handleAi}
            disabled={loading || !aiDesc.trim()}
            className="px-4 py-2 bg-[#6e40c9] hover:bg-[#8957e5] disabled:opacity-50 text-white text-sm rounded-md transition-colors"
          >
            {loading ? 'Generiere…' : 'Mit AI generieren'}
          </button>
        </div>
      ) : (
        <div className="flex flex-col h-full">
          {mode === 'code' && (
            <div className="flex items-center gap-2 px-4 py-2 border-b border-[#30363d]">
              <span className="text-xs text-[#8b949e]">Sprache:</span>
              {LANGUAGES.map(l => (
                <button
                  key={l}
                  onClick={() => setLang(l)}
                  className={`px-2 py-0.5 text-xs rounded capitalize transition-colors ${
                    lang === l
                      ? 'bg-[#1f6feb] text-white'
                      : 'text-[#8b949e] hover:text-[#e6edf3] hover:bg-[#21262d]'
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
            placeholder={mode === 'log' ? 'Log-Inhalt hier einfügen…' : `${lang}-Code hier einfügen…`}
            className="flex-1 bg-[#0d1117] border-0 p-4 text-sm text-[#e6edf3] font-mono resize-none focus:outline-none placeholder-[#484f58]"
            spellCheck={false}
          />
          {error && <div className="text-xs text-[#f85149] px-4 pb-2">{error}</div>}
          <div className="p-3 border-t border-[#30363d]">
            <button
              onClick={handleParse}
              disabled={loading || !code.trim()}
              className="w-full py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-md transition-colors"
            >
              {loading ? 'Analysiere…' : 'State Machine extrahieren'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
