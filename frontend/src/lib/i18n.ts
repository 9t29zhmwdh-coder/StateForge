import { create } from 'zustand'

export type Lang = 'en' | 'de'

const STORAGE_KEY = 'stateforge_lang'

interface Dict {
  [key: string]: string | Dict
}

const translations: Record<Lang, Dict> = {
  en: {
    nav: {
      tagline: 'State Machine Studio',
      machines: 'Machines', newMachine: '+ New',
      tabSplit: 'Code ↔ Diagram', tabEditor: 'Diagram Editor',
      tabAnalyze: 'Analyze', tabGenerate: 'Generator', tabSettings: 'Settings',
      newMachinePrompt: 'Name of the state machine:',
    },
    split: {
      input: 'Input', diagram: 'Diagram', interactive: 'Interactive', export: 'Export',
      noMachineLoaded: 'No machine loaded',
    },
    codePanel: {
      code: 'Source', log: 'Log File', ai: 'AI Description',
      aiPlaceholder: 'Describe your state machine in English or German...\ne.g.: An authentication machine with states Idle, Loading, Authenticated and Error.',
      generating: 'Generating...', generateWithAi: 'Generate with AI',
      language: 'Language:',
      logPlaceholder: 'Paste log content here...',
      codePlaceholderSuffix: '-code here...',
      analyzing: 'Analyzing...', extractMachine: 'Extract State Machine',
    },
    diagram: {
      noMachineSelected: 'No state machine selected',
      save: 'Save',
    },
    analysis: {
      noMachineSelected: 'No state machine selected',
      states: 'States', transitions: 'Transitions',
      deterministic: 'Deterministic', yes: 'Yes', no: 'No',
      initialState: 'Initial State', present: 'Present', missing: 'Missing',
      unreachableStates: 'Unreachable States',
      aiSummary: 'AI Summary',
      analyzing: 'AI analyzing...', enhanceWithAi: 'Enhance with AI',
    },
    generator: {
      noMachineSelected: 'No state machine selected',
      targetLanguage: 'Target Language:',
      copied: 'Copied!', copy: 'Copy', download: 'Download',
      generating: 'Generating...', generatingComment: '// Generating code...',
      errorPrefix: '// Error:',
    },
    settings: {
      title: 'Settings',
      aiBackend: 'AI Backend', backend: 'Backend',
      claudeApiKey: 'Claude API Key', setPlaceholder: '••••••••••••• (set)',
      saved: 'Saved!', save: 'Save',
      keychainNote: 'Stored in the system keychain.',
      ollamaUrl: 'Ollama URL', model: 'Model',
      diagram: 'Diagram', defaultFormat: 'Default Format',
      autoAiEnhance: 'Auto AI Enhance', active: 'Active', inactive: 'Inactive',
      saveSettings: 'Save Settings',
    },
  },
  de: {
    nav: {
      tagline: 'State Machine Studio',
      machines: 'Maschinen', newMachine: '+ Neu',
      tabSplit: 'Code ↔ Diagramm', tabEditor: 'Diagramm-Editor',
      tabAnalyze: 'Analyse', tabGenerate: 'Generator', tabSettings: 'Einstellungen',
      newMachinePrompt: 'Name der State Machine:',
    },
    split: {
      input: 'Eingabe', diagram: 'Diagramm', interactive: 'Interaktiv', export: 'Export',
      noMachineLoaded: 'Keine Maschine geladen',
    },
    codePanel: {
      code: 'Quellcode', log: 'Log-Datei', ai: 'AI-Beschreibung',
      aiPlaceholder: 'Beschreibe deine State Machine auf Deutsch oder Englisch...\nz.B.: Eine Authentifizierungsmaschine mit den Zuständen Idle, Loading, Authenticated und Error.',
      generating: 'Generiere...', generateWithAi: 'Mit AI generieren',
      language: 'Sprache:',
      logPlaceholder: 'Log-Inhalt hier einfügen...',
      codePlaceholderSuffix: '-Code hier einfügen...',
      analyzing: 'Analysiere...', extractMachine: 'State Machine extrahieren',
    },
    diagram: {
      noMachineSelected: 'Keine State Machine ausgewählt',
      save: 'Speichern',
    },
    analysis: {
      noMachineSelected: 'Keine State Machine ausgewählt',
      states: 'Zustände', transitions: 'Transitionen',
      deterministic: 'Deterministisch', yes: 'Ja', no: 'Nein',
      initialState: 'Initialzustand', present: 'Vorhanden', missing: 'Fehlt',
      unreachableStates: 'Nicht erreichbare Zustände',
      aiSummary: 'AI-Zusammenfassung',
      analyzing: 'AI analysiert...', enhanceWithAi: 'Mit AI anreichern',
    },
    generator: {
      noMachineSelected: 'Keine State Machine ausgewählt',
      targetLanguage: 'Zielsprache:',
      copied: 'Kopiert!', copy: 'Kopieren', download: 'Download',
      generating: 'Generiere...', generatingComment: '// Code wird generiert...',
      errorPrefix: '// Fehler:',
    },
    settings: {
      title: 'Einstellungen',
      aiBackend: 'KI-Backend', backend: 'Backend',
      claudeApiKey: 'Claude API-Schlüssel', setPlaceholder: '••••••••••••• (gesetzt)',
      saved: 'Gespeichert!', save: 'Sichern',
      keychainNote: 'Wird im System-Keychain gespeichert.',
      ollamaUrl: 'Ollama URL', model: 'Modell',
      diagram: 'Diagramm', defaultFormat: 'Standard-Format',
      autoAiEnhance: 'Auto-AI-Anreicherung', active: 'Aktiv', inactive: 'Inaktiv',
      saveSettings: 'Einstellungen speichern',
    },
  },
}

interface LangState {
  lang: Lang
  setLang: (lang: Lang) => void
  toggle: () => void
}

export const useLangStore = create<LangState>((set) => ({
  lang: (localStorage.getItem(STORAGE_KEY) as Lang) || 'en',
  setLang: (lang) => {
    localStorage.setItem(STORAGE_KEY, lang)
    set({ lang })
  },
  toggle: () =>
    set((s) => {
      const next: Lang = s.lang === 'en' ? 'de' : 'en'
      localStorage.setItem(STORAGE_KEY, next)
      return { lang: next }
    }),
}))

export function getLang(): Lang {
  return useLangStore.getState().lang
}

function resolve(dict: Dict, path: string): string {
  const parts = path.split('.')
  let node: string | Dict | undefined = dict
  for (const p of parts) {
    node = typeof node === 'object' ? node[p] : undefined
  }
  return typeof node === 'string' ? node : path
}

export function t(path: string): string {
  return resolve(translations[getLang()], path)
}

export function useT() {
  const lang = useLangStore((s) => s.lang)
  return (path: string) => resolve(translations[lang], path)
}

export function dateLocale(): string {
  return getLang() === 'de' ? 'de-CH' : 'en-US'
}
