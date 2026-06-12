import { useEffect, useCallback, useRef } from 'react'
import {
  ReactFlow, Background, Controls, MiniMap, Panel,
  addEdge, applyNodeChanges, applyEdgeChanges,
  type Node, type Edge, type OnConnect, type OnNodesChange, type OnEdgesChange,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useMachineStore } from '../../stores/machineStore'
import { api, type DiagramGraph, type DiagramNode, type DiagramEdge } from '../../lib/tauri'

function toFlowNodes(nodes: DiagramNode[]): Node[] {
  return nodes.map(n => ({
    id: n.id,
    position: n.position ?? { x: 0, y: 0 },
    data: { label: n.label, kind: n.kind, ...n.data },
    type: 'default',
    style: nodeStyle(n.kind),
    width: n.width ?? 120,
    height: n.height ?? 40,
  }))
}

function toFlowEdges(edges: DiagramEdge[]): Edge[] {
  return edges.map(e => ({
    id: e.id,
    source: e.source,
    target: e.target,
    label: e.label,
    style: edgeStyle(e.kind),
    animated: e.kind === 'auto',
  }))
}

function nodeStyle(kind: string): React.CSSProperties {
  const base: React.CSSProperties = {
    background: '#1c2128', border: '1px solid #30363d',
    borderRadius: '6px', color: '#e6edf3', fontSize: '12px',
  }
  if (kind === 'initial')  return { ...base, border: '2px solid #3fb950', borderRadius: '50%' }
  if (kind === 'final')    return { ...base, border: '3px double #58a6ff' }
  if (kind === 'error')    return { ...base, border: '1px solid #f85149', background: '#2d0f0f' }
  if (kind === 'parallel') return { ...base, border: '1px dashed #d2a8ff' }
  return base
}

function edgeStyle(kind: string): React.CSSProperties {
  if (kind === 'error')   return { stroke: '#f85149' }
  if (kind === 'timeout') return { stroke: '#d29922', strokeDasharray: '5,3' }
  if (kind === 'auto')    return { stroke: '#3fb950' }
  return { stroke: '#58a6ff' }
}

export function DiagramEditor() {
  const { active, graph, setGraph, updateMachine } = useMachineStore()
  const nodesRef = useRef<Node[]>([])
  const edgesRef = useRef<Edge[]>([])

  const [nodes, setNodes] = [
    nodesRef.current,
    (v: Node[] | ((p: Node[]) => Node[])) => {
      nodesRef.current = typeof v === 'function' ? v(nodesRef.current) : v
    }
  ]
  const [edges, setEdges] = [
    edgesRef.current,
    (v: Edge[] | ((p: Edge[]) => Edge[])) => {
      edgesRef.current = typeof v === 'function' ? v(edgesRef.current) : v
    }
  ]

  // Use proper React state
  const [flowNodes, setFlowNodes] = [nodes, setNodes] as unknown as [Node[], React.Dispatch<React.SetStateAction<Node[]>>]
  const [flowEdges, setFlowEdges] = [edges, setEdges] as unknown as [Edge[], React.Dispatch<React.SetStateAction<Edge[]>>]

  useEffect(() => {
    if (!active) return
    api.getDiagramGraph(active.id).then(g => {
      setGraph(g)
    }).catch(console.error)
  }, [active?.id])

  useEffect(() => {
    if (!graph) return
    setFlowNodes(toFlowNodes(graph.nodes))
    setFlowEdges(toFlowEdges(graph.edges))
  }, [graph])

  const onNodesChange: OnNodesChange = useCallback(
    (changes) => setFlowNodes((nds) => applyNodeChanges(changes, nds)),
    []
  )
  const onEdgesChange: OnEdgesChange = useCallback(
    (changes) => setFlowEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  )
  const onConnect: OnConnect = useCallback(
    (params) => setFlowEdges((eds) => addEdge(params, eds)),
    []
  )

  const handleSave = async () => {
    if (!active || !graph) return
    const updatedGraph: DiagramGraph = {
      ...graph,
      nodes: flowNodes.map(n => ({
        id: n.id, label: String(n.data.label ?? n.id),
        kind: (n.data.kind as string ?? 'normal') as DiagramNode['kind'],
        position: n.position, data: {},
      })),
    }
    const updated = await api.updateMachineFromGraph(active.id, updatedGraph)
    updateMachine(updated)
  }

  if (!active) {
    return (
      <div className="flex-1 flex items-center justify-center text-[#8b949e]">
        Keine State Machine ausgewählt
      </div>
    )
  }

  return (
    <div className="w-full h-full">
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        fitView
      >
        <Background color="#30363d" gap={20} />
        <Controls />
        <MiniMap nodeColor={() => '#58a6ff'} maskColor="rgba(0,0,0,0.7)" />
        <Panel position="top-right">
          <div className="flex gap-2">
            <button
              onClick={handleSave}
              className="px-3 py-1.5 text-xs bg-[#238636] hover:bg-[#2ea043] text-white rounded-md transition-colors"
            >
              Speichern
            </button>
          </div>
        </Panel>
        <Panel position="top-left">
          <div className="text-sm font-semibold text-[#e6edf3] bg-[#161b22] border border-[#30363d] px-3 py-1.5 rounded-md">
            {active.name}
          </div>
        </Panel>
      </ReactFlow>
    </div>
  )
}
