# glsl: portable, precise and cheap on the GPU
**Done means these pass:** `glslangValidator -V shader.frag` (or `glslc`) compiles each stage without warnings, `spirv-val` accepts the SPIR-V (Vulkan), and the shader renders correctly on a real target (WebGL2 / OpenGL ES / desktop GL).
## Rules
1. First line `#version` (e.g. `#version 300 es` for WebGL2, `#version 450` for Vulkan/GL 4.5; WebGL1 is `#version 100`, with `attribute`/`varying`/`gl_FragColor`). Do not mix dialects.
2. In OpenGL ES and WebGL fragment shaders, declare default precision (`precision highp float;` for position math; `mediump` only where proven safe). Declare `out vec4 fragColor;` explicitly in 300 es/330+.
3. Guard maths: `normalize` only non-zero vectors, avoid `pow(x, y)` with `x <= 0`, `sqrt` of negatives, division by zero (`max(d, 1e-5)`), `acos/asin` outside [-1, 1] (clamp); use `mix`, `clamp`, `smoothstep`, `step` instead of `if` where possible; remember `smoothstep` needs `edge0 < edge1`.
4. Keep fragment shaders cheap: no loops with non-constant bounds in WebGL1; unroll or bound loops; avoid dynamic indexing of arrays/samplers where unsupported; move invariant work to the vertex shader or CPU; minimise texture fetches and dependent reads; avoid branching on varying values in hot paths (divergence).
5. Uniforms and varyings: name and group them (`u_time`, `u_resolution`); in Vulkan GLSL use `layout(set=..., binding=...)` and `layout(std140)` blocks with correct padding (a `vec3` aligns to 16 bytes); location-qualify inputs/outputs (`layout(location = 0)`).
6. Colour: shade in linear space and convert to sRGB at output unless the framebuffer is sRGB; premultiplied alpha is a convention you must keep consistent; clamp outputs for HDR-to-LDR.
7. Derivatives (`dFdx`, `fwidth`) and `texture` implicit LOD need uniform control flow; use `textureLod`/`textureGrad` inside branches.
8. Noise/random: hash-based pseudo-random is fine for visuals; never for security; avoid precision-sensitive large arguments to `sin` for hashing on mobile.
9. Comment the maths (units, ranges) and keep one idea per function; test with extreme inputs (0, 1, NaN-prone values, resolution 1x1).
## Pitfalls
Missing precision qualifier (compile error on ES); integer vs float literal mixing (`1` vs `1.0`); `vec3` padding in uniform blocks; `mod` of negatives differs from C `%`; reading uninitialised `out` variables; Y-axis and origin differences between APIs.
## Versions
Pinned: GLSL 4.60 and GLSL ES 3.20 (WebGL2 uses GLSL ES 3.00; checked 2026-10-07, source: not verified online; from knowledge). This skill assumes at least GLSL ES 3.00.
Rule: first run `glslangValidator --version` and read the project's pin (the `#version` line). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Khronos GLSL specifications, the WebGL2 spec, the Vulkan GLSL guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/glsl.md).

Canonical source: `plugins/web-development/frontend/languages/glsl/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/glsl.md` (reference; do not author).
