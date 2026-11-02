# Global Rules

Consolidated summary of every `README.md` under `web-development/`. One canonical scope reference; do not duplicate.

## Root — Web Development
Scaffold for end-to-end web work. Adapter #1 of the invariant core (DECLARE, RETRIEVE, FORBID, GATE, PARTITION, PROVE, RECORD, WAIVE-OR-OBEY). Read `GLOBAL-LOGIC.md` first, then `RULES.md`. Each subfolder ships `README.md` (scope) + `SKILL.md` (operating contract). Keep shared contracts (API shapes, data models, design tokens) in one canonical location and link; never duplicate a schema. `kimi-frontend-kit/` is vendored reference (read-only). `docs/` holds the five-file workstream set.

## Subfolder Scopes
- **frontend-dev** — client-side code: components, pages/routes, state, styles, assets, build config. Inner: `components/`, `pages/`, `hooks/`, `store/`, `styles/`, `assets/`, `lib/`.
- **backend-systems** — server-side code: routes/controllers, services, workers/jobs, domain logic, config. Inner: `routes/`, `controllers/`, `services/`, `jobs/`, `config/`, `utils/`.
- **middleware** — request/response pipeline: auth guards, validation, logging, CORS, rate-limiting, error handlers, compression. Keep each small, named, tested; document execution order.
- **ui-ux-flow** — design source of truth: user flows, wireframes, mockups, tokens, component specs, a11y checklist. Inner: `flows/`, `wireframes/`, `tokens/`, `prototypes/`, `a11y.md`.
- **frontend-design-language** — canonical visual/interaction system: 4-tier tokens, type scale, color, motion, component specs. Components consume semantic tokens only; never redefine them.
- **testing-qa** — unit, integration, e2e tests, fixtures, QA checklists, regression notes.
- **database-data** — schemas, migrations, seeds, indexes, queries, caching, data lifecycle. DB profiles: postgres (relational/migrations), sqlite (local/tests), mongodb (documents), redis (cache/TTL, never system of record).
- **api-integrations** — API contracts (OpenAPI/GraphQL), versioning notes, third-party SDKs, webhooks, mock fixtures.
- **devops-deployment** — environments, CI/CD pipelines, Docker/infra, env-var matrix, hosting, logs/monitoring/runbooks.
- **security-auth** — AuthN/AuthZ (sessions, JWT/OAuth), password policy, secrets management, OWASP checklist, audit notes.
- **kimi-frontend-kit** — vendored reference (read-only): workflows, subagents, SKILL packs, hooks, templates for studio-grade frontend design. Not in audit scope.

## Rules of Placement
- Code goes in the matching subsystem folder; if it fits nowhere, create the folder and update this file in the same change.
- Source code never in `docs/`, `templates/`, or `frontend-design-language/references/`.
- No secrets, `.env`, keys, or tokens committed (`.env.example` only).
- Every directory stays non-empty in git via `.gitkeep`.
- Shared contracts live in one canonical location; consumers link, never copy.
