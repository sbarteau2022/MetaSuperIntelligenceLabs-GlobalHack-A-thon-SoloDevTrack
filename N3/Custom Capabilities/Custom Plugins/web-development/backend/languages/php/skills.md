# php: strict types, prepared statements, escaped output
**Done means these pass:** `php -l` on changed files, `phpstan analyse` (high level) or `psalm`, `php-cs-fixer` or `phpcs` (PSR-12), `phpunit` or `pest`.
## Rules
1. Every file starts with `declare(strict_types=1);`. Use PHP 8.2+ features: typed and `readonly` properties, constructor promotion, enums, `match`, named arguments, union types.
2. Compare with `===`; use `in_array($x, $list, true)`. Never silence errors with `@`. Use `JSON_THROW_ON_ERROR`. Use `?->` for nullable chains.
3. Follow PSR-12 for style and PSR-4 autoloading through Composer. Commit `composer.lock`; run `composer validate` and `composer audit`.
4. Security: PDO prepared statements (real, not emulated); `password_hash` and `password_verify` (argon2id); `random_bytes` for tokens; `hash_equals` to compare secrets; `htmlspecialchars($s, ENT_QUOTES)` for HTML output; CSRF tokens on state-changing requests; `session_regenerate_id` at login.
5. Never `eval`, never `unserialize` untrusted data (use JSON), never `include` a path built from input. Validate uploads (type, size, content) and store them outside the web root.
6. Validate and filter input (`filter_var`, a validation library); do not read `$_GET` or `$_POST` deep in the code.
7. Use exceptions, not error codes. Use `DateTimeImmutable` with an explicit time zone; money as integers or `bcmath`.
8. Dependency injection, small value objects, no globals. Production: errors logged, not displayed.
9. Laravel and Symfony: use the framework's validation; avoid N+1 (`with()`); guard mass assignment (`$fillable`); queue jobs idempotently.
10. Tests: PHPUnit or Pest with data providers for edge tables; no real network or clock.
## Pitfalls
`foreach` by reference leaking the last element; `empty("0")` is true; array copy-on-assignment surprises; type juggling in `switch`; `==` on numeric strings.
## Versions
Pinned: PHP 8.5 (8.5.9 (Aug 2026); 8.6 is in alpha; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least PHP 8.2.
Rule: first run `php --version` and read the project's pin (composer.json require.php). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: PHP The Right Way, PHP manual, PSR-12, OWASP PHP guidance.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/php.md).

Canonical source: `plugins/web-development/backend/languages/php/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/php.md` (reference; do not author).
