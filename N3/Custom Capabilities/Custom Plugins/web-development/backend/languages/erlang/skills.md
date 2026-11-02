# erlang: small processes, supervised, with specs
**Done means these pass:** `rebar3 compile` with `warnings_as_errors`, `rebar3 xref`, `rebar3 dialyzer`, `rebar3 eunit`, `rebar3 ct`.
## Rules
1. Pattern-match in function heads; keep clauses ordered from specific to general; use guards; no catch-all that swallows errors silently.
2. Add `-spec` and `-type` to every exported function so Dialyzer can help; export only what is used (`-export([...])`, `-behaviour(...)`).
3. "Let it crash": do not wrap everything in `try`; handle expected errors with tagged tuples (`{ok, V}` / `{error, R}`); catch specific classes (`error:badarg`) rather than `_:_`.
4. All long-lived processes belong in a supervision tree (`supervisor` with an explicit restart strategy and intensity); use OTP behaviours (`gen_server`, `gen_statem`) rather than hand-rolled receive loops; link or monitor every spawned process.
5. Never block a `gen_server` callback on a long call; use `handle_continue`, cast or a worker; set timeouts on `gen_server:call/3` deliberately; avoid large messages and unbounded mailboxes (back-pressure).
6. Atoms are not garbage collected: never create atoms from external input (`list_to_existing_atom` or keep binaries). Prefer binaries for strings; use iolists for output building.
7. Use `maps` rather than records for open data and records (`-record`) for fixed internal shapes; avoid the process dictionary.
8. Make code upgrade-safe and node-safe: no globals, state in processes; use ETS with clear ownership; `application:ensure_all_started` in tests.
9. Tests: EUnit for units, Common Test for systems; PropEr for properties; no `timer:sleep` for synchronisation.
## Pitfalls
Selective receive scanning a long mailbox; forgetting that `=` is a match (a failed match crashes); string as list of integers; unlinked processes leaking; atom table exhaustion.
## Versions
Pinned: Erlang/OTP 28 (28.4.3 (Apr 2026); OTP 29 is announced; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least OTP 26.
Rule: first run `erl -noshell -eval 'io:format("~s~n",[erlang:system_info(otp_release)]),halt().'` and read the project's pin (.tool-versions, rebar.config minimum_otp_vsn). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Learn You Some Erlang, the OTP design principles, Erlang programming rules.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/erlang.md).

Canonical source: `plugins/web-development/backend/languages/erlang/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/erlang.md` (reference; do not author).
