# ruby: idiomatic, explicit about failure, safe with SQL
**Done means these pass:** `rubocop`, `bundle exec rspec`, `ruby -wc` on changed files; for Rails also `brakeman` and `bundler-audit`.
## Rules
1. Start files with `# frozen_string_literal: true`. Use blocks and `Enumerable` (`map`, `select`, `each_with_object`) over index loops. Early returns and guard clauses over deep nesting.
2. Raise specific exceptions subclassing `StandardError`. `rescue` the narrowest class; never `rescue Exception` or a bare `rescue` that hides bugs. Use `ensure` for cleanup.
3. Use `Hash#fetch` for required keys, keyword arguments past two parameters, `&.` for safe navigation, `Struct` or `Data.define` for value objects.
4. Do not monkey-patch core classes (use refinements or wrappers). Metaprogramming sparingly; if you define `method_missing`, also define `respond_to_missing?`.
5. Keep methods short, with `private` for internals. Prefer composition over inheritance; include `Comparable` and `Enumerable` where they fit.
6. Money is `BigDecimal` or integer cents. Time uses `Time.current` (Rails) or explicit zones.
7. Rails: strong parameters; avoid N+1 with `includes`; scopes for queries; never interpolate into SQL (`where("name = ?", name)`); keep `html_safe` for trusted markup only; CSRF protection on; migrations reversible (use `strong_migrations`); background jobs idempotent.
8. RSpec: `describe`/`context`/`it`, `let` over instance variables, no `sleep`, factories kept small, one expectation focus per example.
## Pitfalls
`||=` with a `false` value; `Hash.new([])` shares one array; `==` versus `equal?`; mutating strings with `<<` on a shared object; `rescue` modifiers hiding errors; mass assignment without `permit`.
## Versions
Pinned: Ruby 4.0 (4.0.7 measured 2026-10-30: `ruby 4.0.7 (2026-09-15 revision 229531a6cf) +PRISM [arm64-darwin27]`; gem 4.0.20; rubocop MISSING; rspec MISSING — install gems before gating). This skill assumes at least Ruby 3.2.
Rule: first run `ruby --version` and read the project's pin (.ruby-version, Gemfile ruby line). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Ruby Style Guide, RuboCop docs, Rails Guides (security, active record querying).

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/ruby.md).

Canonical source: `plugins/web-development/backend/languages/ruby/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/ruby.md` (reference; do not author).
