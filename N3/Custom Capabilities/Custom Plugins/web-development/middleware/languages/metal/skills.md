# metal: address spaces, threadgroups and tight argument layouts
**Done means these pass:** `xcrun -sdk macosx metal -std=metal3.0 -Wall -Wextra -c shader.metal -o shader.air` compiles cleanly (then `xcrun -sdk macosx metallib shader.air -o shader.metallib`), and the shader gives correct results when run with a Metal pipeline or compute test (Xcode GPU frame capture / validation layer on).
## Rules
1. Declare every pointer parameter with its address space: `device` (read/write buffers), `constant` (small read-only data), `threadgroup` (shared within a group), `thread` (private). Use `constant` for uniform structs and `const device` for read-only large buffers.
2. Entry points: `vertex`, `fragment`, `kernel`. Bind with attributes: `[[buffer(n)]]`, `[[texture(n)]]`, `[[sampler(n)]]`, `[[stage_in]]`, `[[position]]`, `[[thread_position_in_grid]]`, `[[thread_position_in_threadgroup]]`, `[[threads_per_threadgroup]]`. Keep the indices in sync with the Swift/Objective-C side (share a header of structs and enums).
3. Shared structs between CPU and GPU must have identical layout: use `simd` types on the host, mind alignment (`float3` occupies 16 bytes; use `packed_float3` when you need 12), and assert `MemoryLayout<T>.stride` against the Metal size.
4. Compute: guard out-of-range threads (`if (gid >= count) return;`); use `threadgroup_barrier(mem_flags::mem_threadgroup)` between writes and reads of threadgroup memory; choose threadgroup sizes that are multiples of the SIMD width (`threadExecutionWidth`, typically 32); avoid `threadgroup` memory over the device limit (32 KB).
5. Maths: use `half` where precision allows (faster, lower bandwidth), `float` for positions and accumulation; `fast::` functions only when error tolerance is acceptable (fast-math is on by default; use `-fno-fast-math` or `precise::` when NaN/Inf handling matters); clamp and guard `normalize`/`rsqrt`/division.
6. Textures: use the right access (`access::sample`, `read`, `write`, `read_write`); samplers with explicit address and filter modes; `texture2d<half>` vs float formats must match the pixel format; sample in fragment functions only (compute uses `read`/`sample` with explicit level).
7. Avoid divergent branches and dynamic indexing into large local arrays; prefer `select`, `mix`, `clamp`; mark functions `inline` and use `constexpr` constants; use function constants (`[[function_constant(n)]]`) for permutations instead of runtime branching.
8. Atomics: `atomic_uint` in `device`/`threadgroup`, with `memory_order_relaxed` unless ordering is required; do not assume ordering between threadgroups.
9. Debug and test: enable the Metal validation layer and shader validation; label functions/buffers; compare compute output with a CPU reference on edge inputs.
## Pitfalls
`float3` packing mismatches; missing barrier on threadgroup memory; out-of-bounds buffer reads in the last threadgroup; forgetting the address space; integer vs float vertex attribute formats.
## Versions
Pinned: Metal Shading Language 4.0 (Apple metal compiler 32023.921 installed (Xcode 27.0); flags below use -std=metal3.0 as the floor; checked 2026-10-07, source: this machine). This skill assumes at least Metal 3.0.
Rule: first run `xcrun -sdk macosx metal --version` and read the project's pin (-std=metalX.Y flags, the deployment target). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Metal Shading Language Specification, Apple's Metal Best Practices Guide, the Metal Feature Set Tables.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/metal.md).

Canonical source: `plugins/web-development/middleware/languages/metal/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/metal.md` (reference; do not author).
