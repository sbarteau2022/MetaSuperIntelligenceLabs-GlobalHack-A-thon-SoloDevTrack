# zig: explicit allocation, errors as values, no hidden control flow
**Done means these pass:** `zig fmt --check .`, `zig build test` (or `zig test file.zig`), and a release-safe build `zig build -Doptimize=ReleaseSafe` runs the tests without safety panics.
## Rules
1. Allocators are explicit parameters: functions that allocate take an `std.mem.Allocator`, and the caller decides (`GeneralPurposeAllocator`/`DebugAllocator` in debug, arenas for batch work). Every allocation has a matching `defer allocator.free(...)` or `errdefer` placed immediately after it.
2. Errors are values: use error unions (`!T`) and `try`; define precise error sets for public APIs; handle with `catch |err| switch (err) { ... }`; never `catch unreachable` unless the invariant is proven (and commented); do not use `@panic` for expected failures.
3. Use `defer` for cleanup in the success path and `errdefer` for cleanup only on error; order matters (they run in reverse).
4. Optionals (`?T`) with `orelse`/`if (x) |v|`, tagged unions with exhaustive `switch`, `enum`s with explicit tag types; slices (`[]const u8`) over pointers; sentinel-terminated (`[:0]`) only for C interop.
5. Bounds and overflow are safety-checked in Debug/ReleaseSafe: use `+%`/`+|` or `std.math.add` when wrapping/saturating/checked arithmetic is intended; use `@intCast`/`@as` deliberately and prefer `std.math.cast` for fallible casts.
6. `comptime` for generics and constants, but keep it readable; no hidden allocations or control flow; avoid global mutable state.
7. Tests live next to the code in `test "name" { ... }` blocks using `std.testing` (`expectEqual`, `expectError`), and use `std.testing.allocator` so leaks fail the test.
8. C interop: `@cImport` once per module, `extern`/`callconv(.C)`, own the lifetime of any pointer you hand out.
9. Concurrency: `std.Thread` with explicit synchronisation (`Mutex`, atomics); no data races on plain variables.
## Pitfalls
Returning a pointer to stack memory; use-after-free after `defer free`; forgetting `errdefer` on a partially-built structure; slices invalidated by `ArrayList` growth; implicit integer widening assumptions.
## Versions
Pinned: Zig 0.16 (0.16.0 (2026-04-14); the language is pre-1.0 and changes between releases; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Zig 0.13.
Rule: first run `zig version` and read the project's pin (build.zig.zon minimum_zig_version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Zig language reference and standard library docs, the Zig style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/zig.md).

Canonical source: `plugins/web-development/backend/languages/zig/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/zig.md` (reference; do not author).
