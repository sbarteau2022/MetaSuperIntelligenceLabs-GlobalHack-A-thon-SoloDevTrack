# assembly / loops

Date: 2026-10-30. Status: PROPOSED. Scope: `plugins/web-development/backend/languages/assembly/`.

- Loop: delivery runbook (07) → research runbook (08) → pressure-test (10) → release runbook (11). Route/handler/service/validator contract on disk before fan-out (PARTITION); critic pass required (PROVE).
- Runbooks live in `N3/runbooks/` (canonical). This file names the loop; it never duplicates runbook bodies (RETRIEVE: link, never duplicate).
- Stop-gate veto is final within a session: pending checker/contract issues are fixed or each recorded with a dated reason before done is claimed.
