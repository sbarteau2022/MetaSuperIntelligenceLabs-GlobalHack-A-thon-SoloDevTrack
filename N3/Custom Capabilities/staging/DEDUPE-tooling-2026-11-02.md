# Tooling dedupe receipt - 2026-11-02

Order: one set of anything. Home base: ~/Desktop/N3/Custom Capabilities/ (Desktop N3 staging only).
Future home n3/hypermind NOT touched. No servers touched (SurrealDB pid 1709 on 8000, oMLX pid 9681 on 8002, verified after).
No AGENTS.md / README.md edits, no commits. No skills dirs touched (staging skills items intact).
Removal method: move to Mac Trash ONLY (~/.Trash), never permanent delete, never rm.
Repoint method: each removed duplicate replaced by a symlink to the canonical home copy, so loaders still resolve and only one set of CONTENT exists.
status-line-gateway.sh: already in Trash per Stewart order - NOT restored, NOT recreated (verified still in Trash).

## Counts

- Duplicates removed to Trash: 22
- Symlinks created: 22
- Canonical copies intact and hash-verified after: 22
- Unique live items left in place: Cline Hooks 195 files, Cline Rules 4 files
- Failures / rollbacks: 0

## Removed duplicates - Trash, repointed by symlink

- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/01-runbook-plan-and-judges.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/01-runbook-plan-and-judges.md
  - sha256: df78f9101d6b0dfe0c6b2f82ee9bf84729e42a462e3dbc5876c170d2cd32df52
  - trash: /Users/stewartbarteau/.Trash/01-runbook-plan-and-judges.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/01-runbook-plan-and-judges.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/01-runbook-plan-and-judges.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/02-two-strike-tactical-escalation.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/02-two-strike-tactical-escalation.md
  - sha256: af64057089455aeaca1b0da7171c46e9ec24c350283d685274ce82beba92c471
  - trash: /Users/stewartbarteau/.Trash/02-two-strike-tactical-escalation.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/02-two-strike-tactical-escalation.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/02-two-strike-tactical-escalation.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/03-kv-cache-and-continuity.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/03-kv-cache-and-continuity.md
  - sha256: 51dc42d49d5b8424057658a452d12570d4f1c89dcf84ef125f7125d0b3d8e6c4
  - trash: /Users/stewartbarteau/.Trash/03-kv-cache-and-continuity.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/03-kv-cache-and-continuity.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/03-kv-cache-and-continuity.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/04-hole-isolation-and-verification.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/04-hole-isolation-and-verification.md
  - sha256: 3e0229de2fc0fe96137e90b52717d72afe76ca1baab16c9a2c3d04f3a424285b
  - trash: /Users/stewartbarteau/.Trash/04-hole-isolation-and-verification.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/04-hole-isolation-and-verification.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/04-hole-isolation-and-verification.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/05-rust-tokio-systems-engineering.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/05-rust-tokio-systems-engineering.md
  - sha256: 3d84729a55da5b609e06b830adc185e42f5ac00bbde60fa0b53f470c3ea5c210
  - trash: /Users/stewartbarteau/.Trash/05-rust-tokio-systems-engineering.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/05-rust-tokio-systems-engineering.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/05-rust-tokio-systems-engineering.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/06-agent-experience-and-tool-design.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/06-agent-experience-and-tool-design.md
  - sha256: 4b499f7212b8429df3c38b52c3ab9e110699c64d7ede8da61efeaa6ddba87fa5
  - trash: /Users/stewartbarteau/.Trash/06-agent-experience-and-tool-design.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/06-agent-experience-and-tool-design.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/06-agent-experience-and-tool-design.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/07-delivery-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/07-delivery-runbook.md
  - sha256: be187f1185642db85fb79a4e5ae98e5957157a4708d5959ed9bafdfe1ca057fb
  - trash: /Users/stewartbarteau/.Trash/07-delivery-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/07-delivery-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/07-delivery-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/08-research-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/08-research-runbook.md
  - sha256: 4df977c9ee4ef24acad03f40672cec69a7dd5553f6f8f64af879a908ed12b631
  - trash: /Users/stewartbarteau/.Trash/08-research-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/08-research-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/08-research-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/09-intake-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/09-intake-runbook.md
  - sha256: 9baf21404fb6de47a9e5e38bb81c1c6e289a699205a772ea4ee1ac63812adff6
  - trash: /Users/stewartbarteau/.Trash/09-intake-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/09-intake-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/09-intake-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/10-pressure-test-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/10-pressure-test-runbook.md
  - sha256: 2a9e7d47a4525a924fe502c55527475750e8e70050715d06cc6b66e2bf3d02e8
  - trash: /Users/stewartbarteau/.Trash/10-pressure-test-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/10-pressure-test-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/10-pressure-test-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/11-release-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/11-release-runbook.md
  - sha256: 244b5c0ebc5155f63a623359ba72c0cdab1cdbdb105bd922c95f94bdaed0010a
  - trash: /Users/stewartbarteau/.Trash/11-release-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/11-release-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/11-release-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/12-incident-runbook.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/12-incident-runbook.md
  - sha256: e3c0f83b10c37e519de22670e75ac076ac200d8bcf4ce53510ae6a851977ca7b
  - trash: /Users/stewartbarteau/.Trash/12-incident-runbook.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/12-incident-runbook.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/12-incident-runbook.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/2026-09-14-runbook-controller.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/2026-09-14-runbook-controller.md
  - sha256: 9046c08a747e7c2bd6d16f22f777b0551f897e11789d5d93cd7a124efd035233
  - trash: /Users/stewartbarteau/.Trash/2026-09-14-runbook-controller.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/2026-09-14-runbook-controller.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/2026-09-14-runbook-controller.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/INDEX.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/INDEX.md
  - sha256: 42b84fb6b07815fbd6c1bb4a6aa6c2157211b62d54830ff46360a8350068d302
  - trash: /Users/stewartbarteau/.Trash/INDEX.md.dedupe-20261102-1
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/INDEX.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/INDEX.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/README.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/README.md
  - sha256: 37292fcd4d909680927f8d1932b2b3c1a6d8d754f6e225aebe0322c71d0fcdc3
  - trash: /Users/stewartbarteau/.Trash/README.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/README.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/README.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/engineering.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/engineering.md
  - sha256: abfd53e890508926e8745c92305d0cef0eabd4a0214f77c35b1667918b669693
  - trash: /Users/stewartbarteau/.Trash/engineering.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/engineering.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/engineering.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/landing-page-checklist.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/landing-page-checklist.md
  - sha256: fbaac00f801e78a901ee886028cc7339ab3939df65fb31cb3f8f788bc59ec061
  - trash: /Users/stewartbarteau/.Trash/landing-page-checklist.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/landing-page-checklist.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/landing-page-checklist.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/swarm-phase0-prereg.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/swarm-phase0-prereg.md
  - sha256: bdac35318e78c36e5b28dc056c9be94fc7fc4a2e942093556562d6aaa5893abe
  - trash: /Users/stewartbarteau/.Trash/swarm-phase0-prereg.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/swarm-phase0-prereg.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/swarm-phase0-prereg.md
- Cline Workflows -> home Custom Runbooks: /Users/stewartbarteau/Documents/Cline/Workflows/web-design.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/web-design.md
  - sha256: a1869a895351b9bb8cbb705a473a8d5981476838e5493c8bc20bee39b12a8fd7
  - trash: /Users/stewartbarteau/.Trash/web-design.md
  - symlink: /Users/stewartbarteau/Documents/Cline/Workflows/web-design.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Runbooks/web-design.md
- plugin commands -> home Custom Tools: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/commands/harness-run.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Tools/atoma-harness-commands/harness-run.md
  - sha256: c5e445fa9d2b469fe4786003432459082b6fd2a57bdcd9d97e9dffc44a974237
  - trash: /Users/stewartbarteau/.Trash/harness-run.md
  - symlink: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/commands/harness-run.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Tools/atoma-harness-commands/harness-run.md
- plugin commands -> home Custom Tools: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/commands/harness-status.md
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Tools/atoma-harness-commands/harness-status.md
  - sha256: 685d1ae08a9640a581b8157effbd2342db2fc91271f457177469e85bc298eff1
  - trash: /Users/stewartbarteau/.Trash/harness-status.md
  - symlink: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/commands/harness-status.md -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Tools/atoma-harness-commands/harness-status.md
- plugin hooks -> home Custom Hooks: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/hooks/hooks.json
  - canonical: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Hooks/atoma-harness-hooks.json
  - sha256: 1e472c6951fc940276f075e751f253006aaeb1c10be162358efbef114aa1d1ee
  - trash: /Users/stewartbarteau/.Trash/hooks.json
  - symlink: /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Plugins/atoma-harness/hooks/hooks.json -> /Users/stewartbarteau/Desktop/N3/Custom Capabilities/Custom Hooks/atoma-harness-hooks.json

## Notes

- INDEX.md from Cline Workflows collided with a pre-existing INDEX.md in Trash; its trash copy is INDEX.md.dedupe-20261102-1. All other trash names are unmodified.
- Every canonical hash was verified identical before the move and unchanged after; every symlink was verified to resolve to its canonical and return the same hash.

## Left alone, with reason

- Cline Hooks (195 files, incl. compiled/ and scripts/): unique live-loaded kit, no duplicate elsewhere - canonical at that path.
- Cline Rules (AGENTS.md, ENGINEERING-GOVERNANCE.md, global-engineering-law.md, opendesign-global-rules.md): unique live-loaded globals, no duplicate elsewhere - canonical at that path.
- Repo-anchored design docs (MasterTooling ToolingC01-C07 + audit docs, MasterDesignDocument C22/C28/C29, MasterToolingIndex): repo records, not duplicate content - left.
- Whole trees (N3 web-development kit, gateway-3p): trees, not duplicate files - left for the later Rust/TypeScript 7.1 conversion decision.
- Fleet launchers (rapid-up.sh, OpenDesign-fixed.command), hypermind-nested-kv/tools, Elle/tools: live or repo-embedded - left.
- N3 root AGENTS.md and MCP config backups: not tooling duplicates - left.
- Vendor gateway-3p: third-party - left.

