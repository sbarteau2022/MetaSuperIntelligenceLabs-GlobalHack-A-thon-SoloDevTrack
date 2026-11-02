export function Screen02ToolsMCP() {
  const tools = [
    { category: 'File & Workspace', items: ['read_file', 'write_file', 'list_files'] },
    { category: 'Knowledge & Memory', items: ['kb_search', 'memory_search', 'memory_deposit', 'journal', 'scratch'] },
    { category: 'Code Execution', items: ['run_shell', 'run_python', 'vm_exec'] },
    { category: 'Documents', items: ['read_document', 'write_document'] },
    { category: 'Web & Browser', items: ['web_search', 'web_fetch', 'browser_open', 'browser_click', 'browser_type', 'browser_snapshot', 'browser_scroll', 'browser_back', 'browser_tabs', 'browser_extract'] },
    { category: 'Workflow & Plugin', items: ['skill_read', 'skill_write', 'skill_list', 'workflow_read', 'workflow_write', 'workflow_list', 'plugin_create', 'connector_write'] },
    { category: 'Scheduling', items: ['schedule'] },
    { category: 'Thinking', items: ['think'] },
    { category: 'Data Movement', items: ['clone_in', 'ship_out'] },
    { category: 'Cognitive Map', items: ['cogmap_brief', 'cogmap_map', 'cogmap_predict', 'cogmap_submit'] },
    { category: 'Engineering & Research', items: ['detect_toolchains', 'plan_language_check', 'run_language_check', 'search_arxiv', 'search_huggingface_models'] },
  ]

  return (
    <div className="screen-02" style={{ padding: '40px', maxWidth: '1200px', margin: '0 auto', height: '100vh', overflow: 'auto' }}>
      <header style={{ marginBottom: '32px' }}>
        <h1 style={{ fontSize: '32px', fontWeight: 700, marginBottom: '8px' }}>Tools & MCP</h1>
        <p style={{ color: 'var(--fg-muted)', fontSize: '16px' }}>
          47 distinct tool capabilities across 13 categories — the complete Model Home harness toolset
        </p>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '20px' }}>
        {tools.map((group, i) => (
          <div key={i} style={{
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-lg)',
            padding: '24px',
            transition: 'var(--transition-normal)',
          }}>
            <h3 style={{
              fontSize: '14px',
              fontWeight: 600,
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
              color: 'var(--accent-blue)',
              marginBottom: '16px',
            }}>
              {group.category} ({group.items.length})
            </h3>
            <ul style={{ listStyle: 'none', display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
              {group.items.map((tool, j) => (
                <li key={j}>
                  <code style={{
                    background: 'var(--bg-primary)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-sm)',
                    padding: '4px 10px',
                    fontSize: '12px',
                    fontFamily: 'var(--font-mono)',
                    color: 'var(--fg-secondary)',
                  }}>
                    {tool}
                  </code>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </div>
  )
}
