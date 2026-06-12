import { useState, useEffect } from 'react'
import { CodePanel } from './CodePanel'
import { DiagramEditor } from './DiagramEditor'
import { useMachineStore } from '../../stores/machineStore'
import { api, type StateMachine } from '../../lib/tauri'

type DiagramFmt = 'flow' | 'mermaid' | 'svg'

export function SplitView() {
  const { active, setActive, machines, setMachines, diagram, diagramFormat, setDiagram } = useMachineStore()
  const [fmt, setFmt] = useState<DiagramFmt>('flow')
  const [mermaidHtml, setMermaidHtml] = useState('')

  const handleParsed = async (m: StateMachine) => {
    setActive(m)
    if (!machines.find(x => x.id === m.id)) {
      setMachines([...machines, m])
    }
    if (fmt !== 'flow') {
      const d = await api.renderDiagram(m.id, fmt).catch(() => '')
      setDiagram(d, fmt)
    }
  }

  useEffect(() => {
    if (!active || fmt === 'flow') return
    api.renderDiagram(active.id, fmt).then(d => setDiagram(d, fmt)).catch(console.error)
  }, [active?.id, fmt])

  useEffect(() => {
    if (fmt !== 'mermaid' || !diagram) return
    // Dynamic mermaid render
    import('mermaid').then(({ default: mermaid }) => {
      mermaid.initialize({ startOnLoad: false, theme: 'dark', securityLevel: 'loose' })
      mermaid.render('sf-mermaid', diagram).then(({ svg }) => setMermaidHtml(svg)).catch(console.error)
    })
  }, [diagram, fmt])

  return (
    <div className="flex h-full">
      {/* Left: Code Input */}
      <div className="w-[420px] flex-shrink-0 border-r border-[#30363d] flex flex-col">
        <div className="px-4 py-2 border-b border-[#30363d] flex items-center justify-between">
          <span className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider">Eingabe</span>
          {active && (
            <span className="text-xs text-[#58a6ff] truncate max-w-[180px]">{active.name}</span>
          )}
        </div>
        <div className="flex-1 overflow-hidden">
          <CodePanel onParsed={handleParsed} />
        </div>
      </div>

      {/* Right: Diagram */}
      <div className="flex-1 flex flex-col">
        <div className="px-4 py-2 border-b border-[#30363d] flex items-center gap-3">
          <span className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider">Diagramm</span>
          <div className="flex gap-1">
            {(['flow', 'mermaid', 'svg'] as const).map(f => (
              <button
                key={f}
                onClick={() => setFmt(f)}
                className={`px-2.5 py-0.5 text-xs rounded transition-colors ${
                  fmt === f
                    ? 'bg-[#1f6feb] text-white'
                    : 'text-[#8b949e] hover:text-[#e6edf3] hover:bg-[#21262d]'
                }`}
              >
                {f === 'flow' ? 'Interaktiv' : f.toUpperCase()}
              </button>
            ))}
          </div>
          {active && fmt !== 'flow' && (
            <button
              onClick={() => {
                const blob = new Blob([diagram], { type: fmt === 'svg' ? 'image/svg+xml' : 'text/plain' })
                const a = document.createElement('a')
                a.href = URL.createObjectURL(blob)
                a.download = `${active.name}.${fmt === 'mermaid' ? 'mmd' : 'svg'}`
                a.click()
              }}
              className="ml-auto text-xs text-[#8b949e] hover:text-[#e6edf3] px-2 py-0.5 hover:bg-[#21262d] rounded"
            >
              Export
            </button>
          )}
        </div>

        <div className="flex-1 overflow-hidden">
          {fmt === 'flow' ? (
            <DiagramEditor />
          ) : fmt === 'mermaid' ? (
            <div className="w-full h-full overflow-auto p-4">
              {mermaidHtml ? (
                <div dangerouslySetInnerHTML={{ __html: mermaidHtml }} className="[&>svg]:max-w-full" />
              ) : (
                <pre className="text-xs text-[#e6edf3] font-mono whitespace-pre-wrap">{diagram}</pre>
              )}
            </div>
          ) : (
            <div className="w-full h-full overflow-auto p-4">
              {diagram ? (
                <div dangerouslySetInnerHTML={{ __html: diagram }} className="[&>svg]:max-w-full" />
              ) : (
                <div className="flex items-center justify-center h-full text-[#8b949e] text-sm">
                  Keine Maschine geladen
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
