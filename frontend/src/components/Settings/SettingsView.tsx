import { useState } from 'react'
import { useSettingsStore } from '../../stores/settingsStore'
import { api, type AppSettings } from '../../lib/tauri'

export function SettingsView() {
  const { settings, hasKey, setSettings, setHasKey } = useSettingsStore()
  const [draft, setDraft] = useState<AppSettings>({ ...settings })
  const [apiKey, setApiKey] = useState('')
  const [saved, setSaved] = useState(false)
  const [keySaved, setKeySaved] = useState(false)
  const [error, setError] = useState('')

  const set = <K extends keyof AppSettings>(k: K, v: AppSettings[K]) =>
    setDraft(d => ({ ...d, [k]: v }))

  const handleSave = async () => {
    setError('')
    try {
      await api.saveSettings(draft)
      setSettings(draft)
      setSaved(true)
      setTimeout(() => setSaved(false), 1500)
    } catch (e) {
      setError(String(e))
    }
  }

  const handleSaveKey = async () => {
    if (!apiKey.trim()) return
    setError('')
    try {
      await api.saveApiKey(apiKey.trim())
      setHasKey(true)
      setApiKey('')
      setKeySaved(true)
      setTimeout(() => setKeySaved(false), 1500)
    } catch (e) {
      setError(String(e))
    }
  }

  return (
    <div className="p-6 max-w-lg mx-auto overflow-y-auto h-full">
      <h2 className="text-lg font-semibold text-[#e6edf3] mb-6">Einstellungen</h2>

      {/* AI Backend */}
      <Section title="KI-Backend">
        <Label>Backend</Label>
        <div className="flex gap-2 mb-4">
          {['claude', 'ollama'].map(b => (
            <button
              key={b}
              onClick={() => set('ai_backend', b)}
              className={`px-4 py-1.5 text-sm rounded capitalize transition-colors ${
                draft.ai_backend === b
                  ? 'bg-[#1f6feb] text-white'
                  : 'bg-[#161b22] text-[#8b949e] hover:text-[#e6edf3]'
              }`}
            >
              {b}
            </button>
          ))}
        </div>

        {draft.ai_backend === 'claude' && (
          <>
            <Label>Claude API-Schlüssel</Label>
            <div className="flex gap-2">
              <input
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder={hasKey ? '••••••••••••• (gesetzt)' : 'sk-ant-…'}
                className="flex-1 bg-[#161b22] border border-[#30363d] rounded-md px-3 py-1.5 text-sm text-[#e6edf3] focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
              />
              <button
                onClick={handleSaveKey}
                disabled={!apiKey.trim()}
                className="px-3 py-1.5 text-sm bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded-md transition-colors"
              >
                {keySaved ? 'Gespeichert!' : 'Sichern'}
              </button>
            </div>
            <p className="text-xs text-[#8b949e] mt-1">Wird im System-Keychain gespeichert.</p>
          </>
        )}

        {draft.ai_backend === 'ollama' && (
          <>
            <Label>Ollama URL</Label>
            <Input value={draft.ollama_url} onChange={v => set('ollama_url', v)} />
            <Label>Modell</Label>
            <Input value={draft.ollama_model} onChange={v => set('ollama_model', v)} placeholder="llama3" />
          </>
        )}
      </Section>

      {/* Diagram */}
      <Section title="Diagramm">
        <Label>Standard-Format</Label>
        <div className="flex gap-2 mb-4">
          {['mermaid', 'graphviz', 'svg'].map(f => (
            <button
              key={f}
              onClick={() => set('default_diagram_format', f)}
              className={`px-3 py-1 text-sm rounded capitalize transition-colors ${
                draft.default_diagram_format === f
                  ? 'bg-[#1f6feb] text-white'
                  : 'bg-[#161b22] text-[#8b949e] hover:text-[#e6edf3]'
              }`}
            >
              {f}
            </button>
          ))}
        </div>

        <Label>Auto-AI-Anreicherung</Label>
        <div className="flex items-center gap-3">
          <button
            onClick={() => set('auto_ai_enhance', !draft.auto_ai_enhance)}
            className={`w-10 h-5 rounded-full transition-colors relative ${
              draft.auto_ai_enhance ? 'bg-[#238636]' : 'bg-[#30363d]'
            }`}
          >
            <span className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform ${
              draft.auto_ai_enhance ? 'left-5' : 'left-0.5'
            }`} />
          </button>
          <span className="text-sm text-[#8b949e]">
            {draft.auto_ai_enhance ? 'Aktiv' : 'Inaktiv'}
          </span>
        </div>
      </Section>

      {error && <div className="text-xs text-[#f85149] mb-4">{error}</div>}

      <button
        onClick={handleSave}
        className="w-full py-2.5 bg-[#238636] hover:bg-[#2ea043] text-white text-sm rounded-md transition-colors"
      >
        {saved ? 'Gespeichert!' : 'Einstellungen speichern'}
      </button>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-6 bg-[#161b22] border border-[#30363d] rounded-lg p-4">
      <h3 className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider mb-4">{title}</h3>
      {children}
    </div>
  )
}

function Label({ children }: { children: React.ReactNode }) {
  return <div className="text-xs text-[#8b949e] mb-1.5">{children}</div>
}

function Input({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder?: string }) {
  return (
    <input
      value={value}
      onChange={e => onChange(e.target.value)}
      placeholder={placeholder}
      className="w-full bg-[#0d1117] border border-[#30363d] rounded-md px-3 py-1.5 text-sm text-[#e6edf3] font-mono focus:outline-none focus:border-[#58a6ff] mb-3 placeholder-[#484f58]"
    />
  )
}
