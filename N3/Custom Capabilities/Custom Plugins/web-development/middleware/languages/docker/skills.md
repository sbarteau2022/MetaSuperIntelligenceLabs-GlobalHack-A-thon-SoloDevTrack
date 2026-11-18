# docker: pin it, shrink it, run it as nobody
**Done means these pass:** `docker build -t <name> .`, `hadolint Dockerfile`, `docker compose config` (or `docker-compose config`), image scan clean (`trivy image <name>` where available, else Dockerfile secret-denylist grep), container starts and healthcheck turns healthy.
## Rules
1. Never `FROM :latest` — pin digests (`image@sha256:…`) or at minimum minor tags (`node:22.4`); `:latest` is a drift incident waiting for a deploy.
2. One process per container; use compose (multiple services) or a real init/supervisor, never background daemons inside one image.
3. Run as non-root: create a user and set `USER` before `CMD`; root-by-default means any breakout is a host breakout.
4. Ship a `.dockerignore` (`node_modules`, `.git`, build output, secrets) — the build context is a tarball, and bloat slows every build.
5. Multi-stage builds: build in one stage, copy only artifacts into a slim runtime stage; compilers and caches never ship.
6. Compose services declare `healthcheck:` plus `depends_on` with `condition: service_healthy`; startup order is a contract, not a sleep.
7. Secrets via `env_file:` or a secrets manager, never `ENV`/`ARG` baked into layers (layers are forever, `docker history` shows all).
8. Image size discipline: prefer slim/distroless bases, clean package caches in the same `RUN` layer (`apt-get clean && rm -rf /var/lib/apt/lists`).
9. No `privileged: true`, no `--privileged`, no host-socket mounts without a recorded reason and an expiry; each grants host control.
10. Order layers cache-smart: stable steps (deps install) before volatile ones (source copy); one changed line must not rebuild the world.
11. Tag builds reproducibly (`git-sha` or semver, never bare `:latest`); record which tag is deployed where.
## Pitfalls
Layer-cache invalidation order (COPY source before RUN install = full rebuild every edit); build-context bloat (missing `.dockerignore` sends GBs to the daemon); `:latest` drift (works locally, breaks in prod on a different pull); root-by-default (most base images run root unless told otherwise); orphan volumes (`docker compose down` without `-v` leaves data behind — deliberate or deleted?); ENV secrets persisting in layer history; healthcheck-less `depends_on` that only orders start, not readiness.
## Versions
Pinned: Docker 29.7.1 VERIFIED on this machine 2026-10-30 (`docker --version`). NEEDS-VERIFY 2026-10-30: compose plugin (absent — `docker compose` unknown, `docker-compose` not installed), hadolint (not installed), trivy (not installed). Floor: Docker 24 (BuildKit default, compose v2 spec).
Rule: first run `docker --version` and read the project's pins (base-image tags/digests, compose file version). Write only for those versions; if the project pins older ones, follow the project, and if it pins nothing, add pins. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Docker docs (Dockerfile best practices, Compose specification, multi-stage builds).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/docker/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/docker.md` (reference; do not author).
