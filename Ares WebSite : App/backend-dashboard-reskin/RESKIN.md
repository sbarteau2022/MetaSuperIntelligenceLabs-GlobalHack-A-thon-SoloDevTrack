# Backend dashboard reskin — IDE/chat, Research Hub, Swarm board

Source: trashed Ares dashboard components (Trash/ares-frontend-only-2026-11-04/src),
copied to a working copy. NOT copied back into the emptied Ares folder.

## Scope
- IDE/chat: components/cloud/AtomaCloudApp.tsx (IDE panes, chat, terminal)
- Research Hub: components/cloud/CloudResearchHubView.tsx
- Swarm board: components/cloud/CloudSwarmAndMindView.tsx
- Shared: components/cloud/JudgeWalkthroughAndReceipts.tsx, components/layout/*,
  components/ui/*, index.css, lib/*, store/useAtomaStore.ts, pages/AtomaPages.tsx

## What changed — palette (Stewart slate/amber lock)
- Page #26262b, panels #2e2e33, editor #34343a, hairlines #45454d
- White #ffffff + amber #f59e0b do the main work, grey #71717a secondary only
- Green VERIFIED / red BLOCKED kept for status only
- Removed: blue #389cff (141 uses), cyan #06b6d4, purple #a855f7, near-black
  obsidian surfaces, Framer dispatch orange/cream tokens, obsidian/cyan token names

## What changed — less busy
- All blue glow shadows removed (shadow-none), no animate-ping dots
- All gradients flattened to flat slate panels
- Chart/graph accents reduced to white / amber / grey only
- Swarm worker status 'blue' renamed to 'working' (solid amber dot, no ping)
- Primary buttons now amber with dark text, not blue with a cyan glow

## What did NOT change
- Layout columns, routes, and component logic — this is a reskin, not a redesign
- Data: dashboard still reads from useAtomaStore / CLOUD_MODELS (demo data in
  the store — do NOT film those numbers as live backend data). No new /api/
  wiring was added; live backend on 4810 still serves /api/status etc. separately.
- Dead route note: lib/router.ts still lists a 'dispatch' route whose page was
  trashed with the Nemotron clone. That route is dead until its page is rebuilt.

## Verification
- grep for #389cff, #06b6d4, #a855f7, cyan, purple, obsidian: zero hits
- grep for bg-gradient, animate-ping: zero hits
- No files were written back to the Mac Ares folder.
