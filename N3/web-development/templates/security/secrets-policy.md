# Secrets Policy (template)

Status: PROPOSED. Date: 2026-10-30. Rules: RULES.md 9/44/53-57.

1. Store every secret in a manager or `.env` outside version control.
2. Never commit `.env`, keys, tokens, or certs; `.env.example` only.
3. Rotate session identifiers on login, logout, and role change.
4. Rotate service secrets every 90 days, or immediately on exposure.
5. Never log secrets, session tokens, or full card numbers.
6. Review checklist: no secret in code, logs, URLs, errors, or screenshots.
7. Review checklist: cookies carry HttpOnly, Secure, and SameSite.
8. Review checklist: passwords use Argon2id/bcrypt with per-user salt.
9. Incident step 1 — REVOKE the exposed credential everywhere first.
10. Incident step 2 — ROTATE to a fresh secret via the manager.
11. Incident step 3 — AUDIT-LOG scope: who, what, when, blast radius.
12. Record the incident in the workstream Failures.md within 24 hours.
13. Run OWASP Top 10 review before any public release.
14. Document what was NOT checked in every incident note.
15. Append dated corrections; never rewrite an incident record in place.
