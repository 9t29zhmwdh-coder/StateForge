import { useEffect, useState } from 'react'
import { api } from './lib/tauri'
import { useMachineStore } from './stores/machineStore'
import { useSettingsStore } from './stores/settingsStore'
import { useT, useLangStore } from './lib/i18n'
import { DiagramEditor } from './components/Editor/DiagramEditor'
import { SplitView } from './components/Editor/SplitView'
import { AnalysisView } from './components/Analyzer/AnalysisView'
import { GeneratorView } from './components/Generator/GeneratorView'
import { SettingsView } from './components/Settings/SettingsView'

type Tab = 'editor' | 'split' | 'analyze' | 'generate' | 'settings'

export default function App() {
  const [tab, setTab] = useState<Tab>('split')
  const { setMachines, machines, active, setActive } = useMachineStore()
  const { setSettings, setHasKey } = useSettingsStore()
  const t = useT()
  const { lang, toggle } = useLangStore()

  useEffect(() => {
    api.listMachines().then(setMachines).catch(console.error)
    api.getSettings().then(setSettings).catch(console.error)
    api.hasApiKey().then(setHasKey).catch(console.error)
  }, [])

  const handleNewMachine = async () => {
    const name = prompt(t('nav.newMachinePrompt'))
    if (!name) return
    const m = await api.newMachine(name)
    setMachines([...machines, m])
    setActive(m)
  }

  const tabs: { id: Tab; label: string }[] = [
    { id: 'split',    label: t('nav.tabSplit') },
    { id: 'editor',   label: t('nav.tabEditor') },
    { id: 'analyze',  label: t('nav.tabAnalyze') },
    { id: 'generate', label: t('nav.tabGenerate') },
    { id: 'settings', label: t('nav.tabSettings') },
  ]

  return (
    <div className="flex h-screen bg-[#0d1117] text-[#e6edf3] overflow-hidden">
      {/* Sidebar */}
      <aside className="w-52 flex-shrink-0 border-r border-[#30363d] flex flex-col">
        <div className="px-3 py-4 border-b border-[#30363d]">
          <div className="text-sm font-bold text-[#58a6ff] tracking-widest uppercase">StateForge</div>
          <div className="text-xs text-[#8b949e] mt-0.5">{t('nav.tagline')}</div>
        </div>

        {/* Machine list */}
        <div className="flex-1 overflow-y-auto py-2">
          <div className="px-3 py-1 text-xs text-[#8b949e] uppercase tracking-wider">{t('nav.machines')}</div>
          {machines.map(m => (
            <button
              key={m.id}
              onClick={() => setActive(m)}
              className={`w-full text-left px-3 py-1.5 text-sm truncate hover:bg-[#161b22] transition-colors ${
                active?.id === m.id ? 'bg-[#1c2128] text-[#58a6ff] border-l-2 border-[#58a6ff]' : 'text-[#c9d1d9]'
              }`}
            >
              {m.name}
            </button>
          ))}
          <button
            onClick={handleNewMachine}
            className="w-full text-left px-3 py-1.5 text-sm text-[#8b949e] hover:text-[#58a6ff] hover:bg-[#161b22] transition-colors"
          >
            {t('nav.newMachine')}
          </button>
        </div>

        {/* Nav tabs */}
        <nav className="border-t border-[#30363d] py-2">
          {tabs.map(tabItem => (
            <button
              key={tabItem.id}
              onClick={() => setTab(tabItem.id)}
              className={`w-full text-left px-3 py-1.5 text-sm transition-colors ${
                tab === tabItem.id
                  ? 'bg-[#1c2128] text-[#58a6ff]'
                  : 'text-[#8b949e] hover:text-[#e6edf3] hover:bg-[#161b22]'
              }`}
            >
              {tabItem.label}
            </button>
          ))}
        </nav>

        {/* Language toggle */}
        <button
          onClick={toggle}
          className="border-t border-[#30363d] px-3 py-2 text-xs text-[#8b949e] hover:text-[#58a6ff] hover:bg-[#161b22] transition-colors text-left"
        >
          {lang === 'en' ? 'DE' : 'EN'}
        </button>
      </aside>

      {/* Main */}
      <main className="flex-1 overflow-hidden">
        {tab === 'editor'   && <DiagramEditor />}
        {tab === 'split'    && <SplitView />}
        {tab === 'analyze'  && <AnalysisView />}
        {tab === 'generate' && <GeneratorView />}
        {tab === 'settings' && <SettingsView />}
      </main>
    </div>
  )
}
