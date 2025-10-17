import { useEffect } from 'react'
import ReactFlow, {
  Node,
  Edge,
  Background,
  Controls,
  NodeTypes,
  useNodesState,
  useEdgesState,
} from 'reactflow'
import 'reactflow/dist/style.css'
import { Box, Chip, Stack, IconButton, Collapse } from '@mui/material'
import {
  ExpandMore as ExpandMoreIcon,
  TrendingUp as TrendingUpIcon,
} from '@mui/icons-material'
import type { ReasoningResult } from '../types/api'

// Minimal concept node
function ConceptNode({ data }: { data: { label: string; isAnswer?: boolean } }) {
  return (
    <Box
      sx={{
        px: 2,
        py: 1,
        bgcolor: data.isAnswer ? 'success.main' : 'primary.main',
        color: 'white',
        borderRadius: 2,
        fontSize: '0.875rem',
        fontWeight: 500,
        boxShadow: 1,
      }}
    >
      {data.label}
    </Box>
  )
}

const nodeTypes: NodeTypes = {
  concept: ConceptNode,
}


interface Props {
  reasoning: ReasoningResult
  expanded: boolean
  onToggle: () => void
}

export default function InlineReasoningGraph({ reasoning, expanded, onToggle }: Props) {
  const [nodes, setNodes, onNodesChange] = useNodesState([])
  const [edges, setEdges, onEdgesChange] = useEdgesState([])

  // Safety check for reasoning paths
  if (!reasoning?.paths?.length) {
    return null
  }

  useEffect(() => {
    if (!expanded || !reasoning.paths?.length) return

    const allNodes: Node[] = []
    const allEdges: Edge[] = []

    // Convert API path format to graph nodes/edges
    reasoning.paths.forEach((path, index) => {
      const yOffset = index * 120
      
      // Create nodes for each concept in the path
      path.concepts.forEach((conceptId, i) => {
        allNodes.push({
          id: `${index}-${i}`,
          type: 'concept',
          data: { 
            label: conceptId, 
            isAnswer: i === path.concepts.length - 1 
          },
          position: { x: i * 200, y: yOffset },
        })
        
        // Create edge to next concept
        if (i < path.concepts.length - 1) {
          allEdges.push({
            id: `${index}-${i}-${index}-${i+1}`,
            source: `${index}-${i}`,
            target: `${index}-${i+1}`,
            label: `${(path.confidence * 100).toFixed(0)}%`,
            animated: true,
            style: { stroke: '#6750A4', strokeWidth: 2 },
            labelStyle: { fontSize: 11, fill: '#666' },
            labelBgStyle: { fill: '#FEF7FF' },
          })
        }
      })
    })

    setNodes(allNodes)
    setEdges(allEdges)
  }, [expanded, reasoning])

  return (
    <Box sx={{ mt: 1 }}>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 1 }}>
        <IconButton
          size="small"
          onClick={onToggle}
          sx={{
            transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: '0.2s',
          }}
        >
          <ExpandMoreIcon fontSize="small" />
        </IconButton>
        <Chip
          icon={<TrendingUpIcon />}
          label={`${(reasoning.confidence * 100).toFixed(0)}% confidence`}
          size="small"
          color="success"
          variant="outlined"
        />
        <Chip
          label={`${reasoning.paths?.length || 0} path${reasoning.paths?.length !== 1 ? 's' : ''}`}
          size="small"
          variant="outlined"
        />
      </Stack>

      <Collapse in={expanded}>
        <Box
          sx={{
            height: 300,
            bgcolor: 'grey.50',
            borderRadius: 2,
            border: '1px solid',
            borderColor: 'divider',
          }}
        >
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            nodeTypes={nodeTypes}
            fitView
            minZoom={0.2}
            maxZoom={1.5}
            nodesDraggable={false}
            nodesConnectable={false}
            elementsSelectable={false}
          >
            <Background />
            <Controls showInteractive={false} />
          </ReactFlow>
        </Box>
      </Collapse>
    </Box>
  )
}
