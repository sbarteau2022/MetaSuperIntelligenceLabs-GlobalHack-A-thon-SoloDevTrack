# hlsl: correct semantics, packed constants, compiled with dxc
**Done means these pass:** `dxc -T ps_6_0 -E main -Zi -Qembed_debug shader.hlsl` (use the stage and shader model you target: `vs_6_0`, `cs_6_0`) compiles with no warnings, and the shader behaves correctly in the target pipeline (D3D12, or Vulkan via `-spirv`).
## Rules
1. Use DXC and Shader Model 6+, not FXC, for new code. Give every entry point the right stage attribute or `-T` profile; mark compute shaders with `[numthreads(x, y, z)]`.
2. Semantics are part of the interface: `SV_Position`, `SV_Target0`, `SV_DispatchThreadID`, `SV_GroupIndex`; vertex-to-pixel structs must match in order and types; never reuse or omit a required semantic.
3. Constant buffers pack into 16-byte registers: a `float3` followed by a `float` shares a register, but a member never straddles a 16-byte boundary. Order members to avoid padding, add explicit padding where needed, and mirror the layout in C++ (`static_assert` sizes). Use `register(b0, space0)` explicitly and keep a root signature in sync.
4. Matrices: `mul(v, M)` vs `mul(M, v)` depends on row/column major packing (`#pragma pack_matrix`/`-Zpr`/`-Zpc`); choose one convention project-wide and comment it.
5. Control flow: use `[branch]`, `[flatten]`, `[unroll]`, `[loop]` deliberately; avoid divergent branches around derivative-using instructions (`Sample` needs uniform flow, else use `SampleLevel`/`SampleGrad`); beware `discard`/`clip` and early-out effects on helper lanes.
6. Maths safety: `saturate` instead of `clamp(x, 0, 1)`; guard `normalize`, `rsqrt`, `pow`, `1/x`; use `min16float` only where precision is proven; use `asuint`/`asfloat` for bit tricks; avoid integer divide by non-constant in hot loops.
7. Compute: group-shared memory (`groupshared`) needs `GroupMemoryBarrierWithGroupSync()` between write and read; guard against out-of-range thread ids (`if (id.x >= count) return;`); avoid races on UAV writes (use `Interlocked*` where needed); dispatch sizes round up.
8. Resources: use typed bindings (`Texture2D<float4>`, `StructuredBuffer<T>`, `RWByteAddressBuffer`) and bindless via `ResourceDescriptorHeap` (SM 6.6) only with a validated layout.
9. Name constants and use `#define` for permutations sparingly (prefer specialisation constants/permutation keys); compile with `-WX` in CI.
## Pitfalls
Implicit truncation warnings (float4 to float3); row-major vs column-major mismatches; padding differences vs C++ structs; sampling inside non-uniform control flow; forgetting barriers for `groupshared`.
## Versions
Pinned: Shader Model 6.8 with DXC (newer shader models exist; check what the target runtime supports; checked 2026-10-07, source: not verified online; from knowledge). This skill assumes at least Shader Model 6.0.
Rule: first run `dxc --version` and read the project's pin (the -T target profile, the D3D12 Agility SDK version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Microsoft HLSL documentation, the DirectX Shader Compiler wiki, D3D12 constant-buffer packing rules.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/hlsl.md).

Canonical source: `plugins/web-development/backend/languages/hlsl/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/hlsl.md` (reference; do not author).
