# elixir: let it crash, but only on purpose
**Done means these pass:** `mix format --check-formatted`, `mix compile --warnings-as-errors`, `mix credo --strict`, `mix dialyzer`, `mix test`.
## Rules
1. Pattern-match in function heads and use multiple clauses with guards instead of `if`/`cond` chains. Keep functions small; pipe (`|>`) only when the data flows as the first argument.
2. Return tagged tuples (`{:ok, value}` / `{:error, reason}`) for expected failures and match on them (`with` for chains). Bang functions (`fetch!`) raise. Let unexpected failures crash and be supervised; do not rescue broadly.
3. Add `@spec` and `@type` to public functions; write `@moduledoc` and `@doc` with doctests. Let Dialyzer pass.
4. Data is immutable: use structs (`defstruct`, `@enforce_keys`) for known shapes, maps for dynamic data, keyword lists for options. Never use atoms created from user input (`String.to_atom` leaks); use `String.to_existing_atom` or keep strings.
5. Processes: `GenServer` for state, `Task` for one-off async work (`Task.Supervisor` for supervised), `Agent` for tiny state. Everything long-lived sits under a supervision tree with a restart strategy; do not spawn unlinked processes.
6. Keep GenServer callbacks fast; do not block the mailbox; use `handle_continue` for init work; use `call` timeouts deliberately.
7. Ecto/Phoenix: changesets validate; use `Repo.transaction`/`Ecto.Multi`; preload to avoid N+1; parameterised queries only; keep business logic in contexts, not controllers; never trust params.
8. Tests: ExUnit with `async: true` where safe; `describe` blocks; `assert_receive` with timeouts, not `Process.sleep`; property tests with StreamData.
## Pitfalls
`Enum` on huge lists where `Stream` fits; large messages copied between processes; unbounded mailboxes; shadowing in `with`; `==` vs `===` on numbers; binary vs charlist confusion.
## Versions
Pinned: Elixir 1.20 on OTP 28 (1.20.3; OTP 29 support is announced; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Elixir 1.15.
Rule: first run `elixir --version` and read the project's pin (.tool-versions, mix.exs elixir requirement). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Elixir guides, Elixir style guide, "Designing for Scalability with Erlang/OTP".

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/elixir.md).

Canonical source: `plugins/web-development/backend/languages/elixir/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/elixir.md` (reference; do not author).
