import { useState } from 'react'
import { useSettingsStore } from '../../stores/settingsStore'
import { api, type AppSettings } from '../../lib/tauri'
import { useT } from '../../lib/i18n'

export function SettingsView() {
  const { settings, hasKey, setSettings, setHasKey } = useSettingsStore()
  const [draft, setDraft] = useState<AppSettings>({ ...settings })
  const [apiKey, setApiKey] = useState('')
  const [saved, setSaved] = useState(false)
  const [keySaved, setKeySaved] = useState(false)
  const [error, setError] = useState('')
  const t = useT()

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
      <h2 className="text-lg font-semibold text-[#e6edf3] mb-6">{t('settings.title')}</h2>

      {/* AI Backend */}
      <Section title={t('settings.aiBackend')}>
        <Label>{t('settings.backend')}</Label>
        <div className="flex gap-2 mb-4">
          {['claude', 'ollama'].map(b => (
            <button
              key={b}
              onClick={() => set('ai_backend', b)}
              className={`px-4 py-1.5 text-sm rounded capitalize transition-colors ${
                draft.ai_backend === b
                  ? 'bg-[#1f6feb] text-white'
                  : 'bg-sf-surface text-sf-muted hover:text-[#e6edf3]'
              }`}
            >
              {b}
            </button>
          ))}
        </div>

        {draft.ai_backend === 'claude' && (
          <>
            <Label>{t('settings.claudeApiKey')}</Label>
            <div className="flex gap-2">
              <input
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder={hasKey ? t('settings.setPlaceholder') : 'sk-ant-...'}
                className="flex-1 bg-sf-surface border border-sf-border rounded-md px-3 py-1.5 text-sm text-[#e6edf3] focus:outline-hidden focus:border-[#58a6ff] placeholder-[#484f58]"
              />
              <button
                onClick={handleSaveKey}
                disabled={!apiKey.trim()}
                className="px-3 py-1.5 text-sm bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded-md transition-colors"
              >
                {keySaved ? t('settings.saved') : t('settings.save')}
              </button>
            </div>
            <p className="text-xs text-sf-muted mt-1">{t('settings.keychainNote')}</p>
          </>
        )}

        {draft.ai_backend === 'ollama' && (
          <>
            <Label>{t('settings.ollamaUrl')}</Label>
            <Input value={draft.ollama_url} onChange={v => set('ollama_url', v)} />
            <Label>{t('settings.model')}</Label>
            <Input value={draft.ollama_model} onChange={v => set('ollama_model', v)} placeholder="llama3" />
          </>
        )}
      </Section>

      {/* Diagram */}
      <Section title={t('settings.diagram')}>
        <Label>{t('settings.defaultFormat')}</Label>
        <div className="flex gap-2 mb-4">
          {['mermaid', 'graphviz', 'svg'].map(f => (
            <button
              key={f}
              onClick={() => set('default_diagram_format', f)}
              className={`px-3 py-1 text-sm rounded capitalize transition-colors ${
                draft.default_diagram_format === f
                  ? 'bg-[#1f6feb] text-white'
                  : 'bg-sf-surface text-sf-muted hover:text-[#e6edf3]'
              }`}
            >
              {f}
            </button>
          ))}
        </div>

        <Label>{t('settings.autoAiEnhance')}</Label>
        <div className="flex items-center gap-3">
          <button
            onClick={() => set('auto_ai_enhance', !draft.auto_ai_enhance)}
            className={`w-10 h-5 rounded-full transition-colors relative ${
              draft.auto_ai_enhance ? 'bg-[#238636]' : 'bg-sf-border'
            }`}
          >
            <span className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform ${
              draft.auto_ai_enhance ? 'left-5' : 'left-0.5'
            }`} />
          </button>
          <span className="text-sm text-sf-muted">
            {draft.auto_ai_enhance ? t('settings.active') : t('settings.inactive')}
          </span>
        </div>
      </Section>

      {error && <div className="text-xs text-[#f85149] mb-4">{error}</div>}

      <button
        onClick={handleSave}
        className="w-full py-2.5 bg-[#238636] hover:bg-[#2ea043] text-white text-sm rounded-md transition-colors"
      >
        {saved ? t('settings.saved') : t('settings.saveSettings')}
      </button>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-6 bg-sf-surface border border-sf-border rounded-lg p-4">
      <h3 className="text-xs font-semibold text-sf-muted uppercase tracking-wider mb-4">{title}</h3>
      {children}
    </div>
  )
}

function Label({ children }: { children: React.ReactNode }) {
  return <div className="text-xs text-sf-muted mb-1.5">{children}</div>
}

function Input({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder?: string }) {
  return (
    <input
      value={value}
      onChange={e => onChange(e.target.value)}
      placeholder={placeholder}
      className="w-full bg-sf-bg border border-sf-border rounded-md px-3 py-1.5 text-sm text-[#e6edf3] font-mono focus:outline-hidden focus:border-[#58a6ff] mb-3 placeholder-[#484f58]"
    />
  )
}
