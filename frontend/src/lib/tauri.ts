import { invoke } from '@tauri-apps/api/core'

// ── Core Types ──────────────────────────────────────────────

export type StateKind = 'initial' | 'normal' | 'final' | 'error' | 'parallel' | 'history'
export type TransitionKind = 'normal' | 'auto' | 'error' | 'timeout' | 'internal'

export interface MachineState {
  id: string
  name: string
  kind: StateKind
  description?: string
  entry_actions: string[]
  exit_actions: string[]
  substates: MachineState[]
  metadata: Record<string, string>
}

export interface Transition {
  id: string
  from_state: string
  to_state: string
  event?: string
  guard?: string
  actions: string[]
  kind: TransitionKind
  probability?: number
}

export type Language = 'swift' | 'kotlin' | 'typescript' | 'go' | 'rust' | 'generic'

export type AnalysisSource =
  | { CodeFile: { path: string; language: Language } }
  | { CodeSnippet: { content: string; language: Language } }
  | { LogFile: { path: string } }
  | { LogContent: { content: string } }
  | 'Manual'

export interface StateMachine {
  id: string
  name: string
  states: MachineState[]
  transitions: Transition[]
  initial_state?: string
  context_type?: string
  source: AnalysisSource
  created_at: string
  updated_at: string
  tags: string[]
  ai_summary?: string
}

export type NodeKind = 'initial' | 'normal' | 'final' | 'error' | 'parallel' | 'group'
export type EdgeKind = 'normal' | 'error' | 'auto' | 'timeout' | 'internal'

export interface DiagramNode {
  id: string
  label: string
  kind: NodeKind
  position?: { x: number; y: number }
  width?: number
  height?: number
  data: Record<string, unknown>
}

export interface DiagramEdge {
  id: string
  source: string
  target: string
  label?: string
  kind: EdgeKind
  data: Record<string, unknown>
}

export interface DiagramGraph {
  nodes: DiagramNode[]
  edges: DiagramEdge[]
  viewport?: { x: number; y: number; zoom: number }
}

export interface AppSettings {
  ai_backend: string
  ollama_url: string
  ollama_model: string
  theme: string
  default_diagram_format: string
  auto_ai_enhance: boolean
}

export interface MachineAnalysis {
  state_count: number
  transition_count: number
  is_deterministic: boolean
  unreachable_states: string[]
  error_states: string[]
  has_initial_state: boolean
  has_final_state: boolean
}

// ── API ─────────────────────────────────────────────────────

export const api = {
  // Parser
  parseCode: (content: string, language: string, sourcePath?: string) =>
    invoke<StateMachine>('parse_code', { content, language, sourcePath }),
  parseLog: (content: string, sourcePath?: string) =>
    invoke<StateMachine>('parse_log', { content, sourcePath }),
  detectLanguage: (content: string) =>
    invoke<string>('detect_language', { content }),

  // Diagram
  renderDiagram: (machineId: string, format: string, direction?: string) =>
    invoke<string>('render_diagram', { machineId, format, direction }),
  renderDiagramFromMachine: (machine: StateMachine, format: string) =>
    invoke<string>('render_diagram_from_machine', { machine, format }),
  getDiagramGraph: (machineId: string) =>
    invoke<DiagramGraph>('get_diagram_graph', { machineId }),
  updateMachineFromGraph: (machineId: string, graph: DiagramGraph) =>
    invoke<StateMachine>('update_machine_from_graph', { machineId, graph }),

  // Generator
  generateCode: (machineId: string, language: string) =>
    invoke<string>('generate_code', { machineId, language }),
  supportedLanguages: () => invoke<string[]>('supported_languages'),

  // Machines
  listMachines: () => invoke<StateMachine[]>('list_machines'),
  getMachine: (machineId: string) => invoke<StateMachine | null>('get_machine', { machineId }),
  saveMachine: (machine: StateMachine) => invoke<void>('save_machine', { machine }),
  deleteMachine: (machineId: string) => invoke<void>('delete_machine', { machineId }),
  newMachine: (name: string) => invoke<StateMachine>('new_machine', { name }),
  addState: (machineId: string, name: string, kind: StateKind) =>
    invoke<StateMachine>('add_state', { machineId, name, kind }),
  addTransition: (machineId: string, fromState: string, toState: string, event?: string, guard?: string) =>
    invoke<StateMachine>('add_transition', { machineId, fromState, toState, event, guard }),
  analyzeMachine: (machine: StateMachine) =>
    invoke<MachineAnalysis>('analyze_machine', { machine }),

  // Settings
  getSettings: () => invoke<AppSettings>('get_settings'),
  saveSettings: (settings: AppSettings) => invoke<void>('save_settings', { settings }),
  saveApiKey: (key: string) => invoke<void>('save_api_key', { key }),
  hasApiKey: () => invoke<boolean>('has_api_key'),

  // AI
  aiEnhanceMachine: (machineId: string) => invoke<StateMachine>('ai_enhance_machine', { machineId }),
  aiFromDescription: (description: string) => invoke<StateMachine>('ai_from_description', { description }),
  checkAiAvailable: () => invoke<boolean>('check_ai_available'),
}
