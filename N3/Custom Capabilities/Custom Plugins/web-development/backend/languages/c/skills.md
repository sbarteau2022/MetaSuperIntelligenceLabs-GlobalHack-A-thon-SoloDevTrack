# c: assume nothing, check everything
**Done means:** it compiles clean with `-std=c17 -Wall -Wextra -Wpedantic -Werror`, tests pass under `-fsanitize=address,undefined -g` (or `valgrind`), and `clang-tidy` and `cppcheck` report nothing new.
## Rules
1. Check every `malloc`, `calloc`, `realloc`, `fopen`, `fread` and system call result. `realloc` into a temporary, then assign. `free` once and set the pointer to `NULL`.
2. Pass a size with every pointer (`ptr, len`), type sizes as `size_t`, and check multiplication for overflow before allocating (`n > SIZE_MAX / sizeof *p`); `calloc` helps.
3. Strings: never `gets`, `strcpy`, `strcat` or `sprintf`. Use `snprintf` and check its return. Always NUL-terminate and track lengths. Never pass user input as a format string.
4. Integers: signed overflow is undefined behaviour. Use unsigned for bit work, check conversions, do not compare signed with unsigned.
5. Undefined behaviour to avoid: out-of-bounds, use-after-free, double free, uninitialised reads, strict-aliasing violations (use `memcpy`), misaligned access, null dereference.
6. Ownership is documented per function (who frees). Use one cleanup path (`goto cleanup`) for functions that acquire several resources.
7. `const` everything that does not change; `static` for file-local functions; header guards; no definitions in headers; `static inline` instead of function-like macros with side effects.
8. No variable-length arrays; initialise every variable; `static_assert` for layout assumptions; `enum` for named constants.
9. Concurrency: no mutable globals; use `_Atomic` or mutexes; run `-fsanitize=thread` on threaded code.
10. Fuzz parsers and decoders; test edge sizes (0, 1, max); build hardened (`-D_FORTIFY_SOURCE=2 -fstack-protector-strong`).
11. Under `-Werror`, unused parameters, variables and static functions fail the build: omit the parameter name (`int main(void)`) or write `(void)arg;`, and delete dead helpers. Compile after every edit with the full flag set.
## Pitfalls
`sizeof` on a pointer parameter; off-by-one in loops and buffers; plain `char` signedness; macro arguments evaluated twice; `strncpy` does not always NUL-terminate.
## Versions
Pinned: C23 (ISO/IEC 9899:2024) (Apple clang 21.0.0 installed; flags below use -std=c17 as the floor, use -std=c23 where the compiler supports it; checked 2026-10-07, source: this machine). This skill assumes at least C17.
Rule: first run `clang --version` and read the project's pin (CMAKE_C_STANDARD, Makefile -std flags). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: SEI CERT C Coding Standard, ISO C17, MISRA C (as a guide).

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/c.md).

Canonical source: `plugins/web-development/backend/languages/c/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/c.md` (reference; do not author).
