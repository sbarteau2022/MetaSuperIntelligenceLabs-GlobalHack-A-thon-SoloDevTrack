import { ChatPanel } from '../components/ChatPanel'

export function Screen08IDEChat() {
  return (
    <div className="screen-08" style={{ display: 'grid', gridTemplateColumns: '1fr 420px', height: '100vh' }}>
      {/* Left: IDE / Code View */}
      <div style={{
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border-subtle)', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', color: 'var(--fg-muted)', fontFamily: 'var(--font-mono)' }}>project/src/agent.ts</span>
          <span style={{ background: 'var(--accent-green)', color: 'var(--bg-primary)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontSize: '10px', fontWeight: 600 }}>ACTIVE SESSION</span>
        </div>
        <pre style={{ flex: 1, padding: '16px', margin: 0, overflow: 'auto', fontSize: '12px', lineHeight: 1.6, fontFamily: 'var(--font-mono)', color: 'var(--fg-secondary)' }}>
{`// Atoma Framer — Agent Loop with Multi-Turn Persistence
import { Agent, Checkpoint, ToolRegistry } from 'home-core';
import { PostgresPool } from 'sqlx';

async function runAgentTurn(
  pool: PostgresPool,
  sessionId: string,
  userInput: string
): Promise<Checkpoint> {
  // 1. Load checkpoint from Postgres (survives restarts)
  const checkpoint = await loadCheckpoint(pool, sessionId);

  // 2. Create agent with full 47-tool registry
  const toolRegistry = ToolRegistry::default();
  const agent = Agent::new(pool.clone(), toolRegistry).await?;

  // 3. Run one turn — agent resumes from checkpoint
  //    Uses PostgresSaver for LangGraph state persistence
  const result = await agent.run(&sessionId, userInput, |delta| {
    // Stream thinking tokens to UI
    emit('thinking', delta);
  }).await?;

  // 4. Checkpoint auto-saved after each turn
  //    Next call with same sessionId continues seamlessly
  return result.checkpoint;
}

// Thread isolation: multiple sessions share the same
// Postgres DB but different thread_ids = different contexts
// Zed/Claude Desktop connect via MCP to ws://localhost:47831/mcp`}
        </pre>
      </div>

      {/* Right: Agent Chat Panel */}
      <div style={{ background: 'var(--bg-secondary)', display: 'flex', flexDirection: 'column' }}>
        <ChatPanel sessionId="ide-chat-session" />
      </div>
    </div>
  )
}
