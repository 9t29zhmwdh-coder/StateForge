import { create } from 'zustand'
import type { StateMachine, DiagramGraph } from '../lib/tauri'

interface MachineStore {
  machines: StateMachine[]
  active?: StateMachine
  graph?: DiagramGraph
  diagram: string
  diagramFormat: string
  setMachines: (m: StateMachine[]) => void
  setActive: (m: StateMachine | undefined) => void
  setGraph: (g: DiagramGraph | undefined) => void
  setDiagram: (d: string, format: string) => void
  updateMachine: (m: StateMachine) => void
}

export const useMachineStore = create<MachineStore>((set) => ({
  machines: [],
  active: undefined,
  graph: undefined,
  diagram: '',
  diagramFormat: 'mermaid',

  setMachines:  (machines) => set({ machines }),
  setActive:    (active) => set({ active }),
  setGraph:     (graph) => set({ graph }),
  setDiagram:   (diagram, diagramFormat) => set({ diagram, diagramFormat }),
  updateMachine: (m) => set((s) => ({
    machines: s.machines.map(x => x.id === m.id ? m : x),
    active: s.active?.id === m.id ? m : s.active,
  })),
}))
