# lua: local, nil-aware and 1-based
**Done means these pass:** `luacheck .` (no warnings), `stylua --check .`, and the tests (`busted`).
## Rules
1. Declare everything `local` (variables and functions); globals are bugs (luacheck flags them). Return a module table from each file (`local M = {} ... return M`); no work at `require` time beyond defining things.
2. Tables are 1-based; the length operator `#` is only reliable on sequences without holes: never store `nil` in an array part; use `table.insert`/`table.remove`; use `table.unpack`/`select('#', ...)` and be careful with `nil` in varargs (`table.pack`).
3. Only `nil` and `false` are falsy (`0` and `""` are true). Use `x == nil` when `false` is a valid value; `x = x or default` breaks for `false`.
4. Errors: `error(msg, level)` with a message (tables for structured errors); catch with `pcall`/`xpcall` (with a traceback handler) at boundaries and return `nil, err` for expected failures; always check results of `io.open` and friends.
5. Strings are immutable: build with `table.concat` rather than repeated `..` in loops; patterns are not regex (use `%` escapes, `string.find(s, p, 1, true)` for plain search); use `string.format`.
6. Do not rely on `pairs` ordering; use an array plus index for ordered data. Do not modify a table while iterating it (except assigning existing fields).
7. Metatables/OOP kept small and explicit (`setmetatable({}, {__index = Class})`); avoid deep inheritance; use `rawget`/`rawset` knowingly.
8. Numbers: Lua 5.3+ has integers and floats; `//` is floor division; `math.tointeger`; comparing floats with `==` is unsafe. Targets differ (5.1, 5.4, LuaJIT, Luau): state which one and avoid features it lacks (`goto`, integer division, `bit` library).
9. Coroutines for cooperative work: check `coroutine.status`, propagate errors from `coroutine.resume` (`coroutine.wrap` rethrows).
10. Tests with `busted` (`describe`/`it`/`assert`), cover empty tables and `nil`.
## Pitfalls
Accidental globals from a typo; `#t` on tables with holes; `a and b or c` when `b` is falsy; string-to-number coercion (`"10" + 1`); forgetting `local` in recursive local function declarations (use `local function`).
## Versions
Pinned: Lua 5.4 (from knowledge; Lua 5.5 may have been released, and LuaJIT and Luau are 5.1-based; checked 2026-10-07, source: not verified online). This skill assumes at least Lua 5.1 semantics where LuaJIT is the target.
Rule: first run `lua -v` and read the project's pin (.luarc.json, the rockspec lua constraint). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Programming in Lua, the Lua reference manual, the luacheck and StyLua documentation.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/lua.md).

Canonical source: `plugins/web-development/backend/languages/lua/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/lua.md` (reference; do not author).
