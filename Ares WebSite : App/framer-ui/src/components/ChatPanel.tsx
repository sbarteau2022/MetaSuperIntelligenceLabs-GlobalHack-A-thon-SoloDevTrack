import { useState, useRef, useEffect, FormEvent } from 'react'
import { useAgentSession } from '../hooks/useAgentSession'

interface ChatPanelProps {
  sessionId?: string
}

export function ChatPanel({ sessionId = 'default' }: ChatPanelProps) {
  const [input, setInput] = useState('')
  const { session, sendMessage } = useAgentSession(sessionId)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [session.history])

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault()
    if (!input.trim() || session.status !== 'idle') return
    sendMessage(input)
    setInput('')
  }

  return (
    <div
      className="chat-panel"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        background: 'var(--bg-secondary)',
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--border-subtle)',
        overflow: 'hidden',
      }}
    >
      <div
        className="chat-header"
        style={{
          padding: '12px 16px',
          borderBottom: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <h3 style={{ margin: 0, fontSize: '14px', fontWeight: 500 }}>Agent Session</h3>
        <AgentStatusIndicator status={session.status} />
      </div>

      <div
        className="chat-messages"
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: '16px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px',
        }}
      >
        {session.history.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSubmit} style={{
        padding: '12px',
        borderTop: '1px solid var(--border-subtle)',
        display: 'flex',
        gap: '8px',
      }}>
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          placeholder={session.status === 'idle' ? 'Ask the agent...' : 'Agent working...'}
          disabled={session.status !== 'idle'}
          style={{
            flex: 1,
            padding: '10px 14px',
            borderRadius: 'var(--radius-md)',
            background: 'var(--bg-primary)',
            border: '1px solid var(--border-subtle)',
            color: 'var(--fg-primary)',
            fontSize: '13px',
            outline: 'none',
          }}
        />
        <button
          type="submit"
          disabled={session.status !== 'idle' || !input.trim()}
          style={{
            padding: '10px 20px',
            borderRadius: 'var(--radius-md)',
            background: 'var(--accent-blue)',
            color: 'white',
            border: 'none',
            fontSize: '13px',
            fontWeight: 500,
            cursor: 'pointer',
            opacity: session.status !== 'idle' || !input.trim() ? 0.5 : 1,
            transition: 'var(--transition-fast)',
          }}
        >
          Send
        </button>
      </form>
    </div>
  )
}

function MessageBubble({ message }: { message: { role: string; content: string; reasoning?: string } }) {
  const isUser = message.role === 'user'
  return (
    <div
      style={{
        alignSelf: isUser ? 'flex-end' : 'flex-start',
        maxWidth: '85%',
        padding: '10px 14px',
        borderRadius: 'var(--radius-md)',
        background: isUser ? 'var(--accent-blue)' : 'var(--card-bg)',
        color: isUser ? 'white' : 'var(--fg-primary)',
        fontSize: '13px',
        lineHeight: 1.5,
        whiteSpace: 'pre-wrap',
      }}
    >
      {message.content}
      {message.reasoning && (
        <details style={{ marginTop: '8px', fontSize: '11px', color: 'var(--fg-muted)' }}>
          <summary style={{ cursor: 'pointer' }}>Reasoning</summary>
          <pre style={{ marginTop: '4px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
            {message.reasoning}
          </pre>
        </details>
      )}
    </div>
  )
}

function AgentStatusIndicator({ status }: { status: 'idle' | 'thinking' | 'running_tool' | 'awaiting_approval' }) {
  const colors = {
    idle: 'var(--accent-green)',
    thinking: 'var(--accent-amber)',
    running_tool: 'var(--accent-blue)',
    awaiting_approval: 'var(--accent-red)',
  }
  const labels = {
    idle: 'Ready',
    thinking: 'Thinking...',
    running_tool: 'Running tool...',
    awaiting_approval: 'Awaiting approval',
  }
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
      <div
        style={{
          width: '8px',
          height: '8px',
          borderRadius: '50%',
          background: colors[status],
          boxShadow: `0 0 8px ${colors[status]}`,
          animation: status === 'thinking' ? 'pulse 1.5s infinite' : 'none',
        }}
      />
      <span style={{ fontSize: '11px', color: 'var(--fg-muted)', textTransform: 'capitalize' }}>
        {labels[status]}
      </span>
    </div>
  )
}
