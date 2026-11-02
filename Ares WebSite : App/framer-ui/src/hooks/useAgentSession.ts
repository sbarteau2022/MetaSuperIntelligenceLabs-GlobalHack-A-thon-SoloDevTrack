import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface Message {
  role: 'user' | 'assistant' | 'tool'
  content: string
  reasoning?: string
}

interface Checkpoint {
  thread: string
  messages: Message[]
  step: number
  tool_calls: number
}

interface ToolResult {
  output: string
  checkpoint: Checkpoint
}

type AgentStatus = 'idle' | 'thinking' | 'running_tool' | 'awaiting_approval'

interface AgentSession {
  id: string
  history: Message[]
  status: AgentStatus
  checkpoint: Checkpoint | null
}

export function useAgentSession(sessionId: string) {
  const [session, setSession] = useState<AgentSession>({
    id: sessionId,
    history: [],
    status: 'idle',
    checkpoint: null,
  })

  // Load checkpoint on mount
  useEffect(() => {
    let mounted = true
    invoke<Checkpoint>('get_checkpoint', { sessionId })
      .then(cp => {
        if (mounted) {
          setSession(s => ({
            ...s,
            checkpoint: cp,
            history: cp.messages.map(m => ({
              role: (m.role as Message['role']) || 'assistant',
              content: m.content,
              reasoning: m.reasoning,
            })),
          }))
        }
      })
      .catch(() => {
        // New session, no checkpoint
      })
    return () => { mounted = false }
  }, [sessionId])

  const sendMessage = useCallback(async (content: string) => {
    if (!content.trim() || session.status !== 'idle') return

    const userMessage: Message = { role: 'user', content }
    setSession(s => ({
      ...s,
      status: 'thinking',
      history: [...s.history, userMessage],
    }))

    try {
      const results = await invoke<ToolResult[]>('send_message', { sessionId, message: content })

      for (const result of results) {
        const assistantMessage: Message = {
          role: 'assistant',
          content: result.output,
        }
        setSession(s => ({
          ...s,
          history: [...s.history, assistantMessage],
          status: 'idle',
          checkpoint: result.checkpoint,
        }))
        // Auto-save checkpoint
        await invoke('save_checkpoint', { sessionId, checkpoint: result.checkpoint })
      }
    } catch (e) {
      setSession(s => ({ ...s, status: 'idle' }))
      console.error('Agent error:', e)
    }
  }, [sessionId, session.status])

  return { session, sendMessage }
}
