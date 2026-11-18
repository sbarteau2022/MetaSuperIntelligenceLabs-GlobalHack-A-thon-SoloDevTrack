# assembly: ABI-exact, commented and tested from C
**Done means:** it assembles without warnings (macOS arm64: `clang -c file.s`; Linux x86-64: `nasm -f elf64 file.asm` or `gcc -c file.s`), links, runs correctly under a C test harness or the debugger (`lldb`/`gdb`), and `objdump -d` (or `otool -tv`) matches what you intended.
## Rules
1. Name the target first (architecture, OS, syntax, assembler) and follow its calling convention exactly: AArch64 AAPCS64 (args `x0-x7`, return `x0`, callee-saved `x19-x28`, `x29` frame pointer, `x30` link register, `sp` 16-byte aligned) or x86-64 System V (args `rdi, rsi, rdx, rcx, r8, r9`, return `rax`, callee-saved `rbx, rbp, r12-r15`, stack 16-byte aligned at calls).
2. Preserve every callee-saved register you use and restore the stack and frame pointer before returning; keep the stack aligned; never use the red zone in code that can be interrupted unless the ABI allows it.
3. Symbol and relocation syntax is per object format: Mach-O (macOS) addresses data with `adrp x1, sym@PAGE` then `add x1, x1, sym@PAGEOFF` and exports `_main` with `.globl`; ELF (Linux) uses `:got:`/`:lo12:` operands and `adrp x1, sym` forms that do not assemble on macOS. Comment every non-trivial instruction group with its purpose and register roles; define constants with `.equ`/`%define` and give labels meaningful names (`.L` prefix for local labels). Keep one routine per `.globl` symbol with `.type`/`.size` (ELF) or a leading underscore (Mach-O).
4. System calls: use the platform's documented numbers and conventions (macOS arm64: `svc #0x80`, number in `x16`; Linux arm64: `svc #0`, number in `x8`; Linux x86-64: `syscall`, number in `rax`); check return values; prefer libc where portability matters.
5. Check every length and bound before loops; use unsigned compares for sizes; beware sign extension and flag semantics (`cmp` then `b.lo`/`jb` unsigned vs `b.lt`/`jl` signed).
6. Data goes in the right section (`.text`, `.data`, `.rodata`, `.bss`); strings include the terminator when C will read them; align data (`.balign`/`.p2align`).
7. Test it: call it from C with a harness that checks results for edge values (0, 1, max, negative), run under `lldb`, and compare with a C reference implementation.
8. Security: no self-modifying code; validate indices; do not leave sensitive data in registers or stack after use.
5. macOS arm64 system calls take arguments in x0, x1, x2 in order (not shifted), the call number in x16, then `svc #0x80`. write is 4 (`x0`=fd, `x1`=buffer, `x2`=length) and exit is 1 (`x0`=status). Do not put the buffer address in x0 for write. Example: `mov x0,#1; adrp x1,msg@PAGE; add x1,x1,msg@PAGEOFF; mov x2,#3; mov x16,#4; svc #0x80; mov x0,#7; mov x16,#1; svc #0x80`.
## Pitfalls
Wrong register for the Nth argument; unaligned stack crashes in libc calls; clobbering a callee-saved register; Mach-O vs ELF symbol naming; flags overwritten between a compare and its branch; x86 AT&T vs Intel operand order.
## Versions
Pinned: AArch64 (Arm ARM) and x86-64 System V (assembler here is Apple clang 21.0.0; checked 2026-10-07, source: this machine). This skill assumes at least the ABI in the first rule.
Rule: first run `clang --version` and read the project's pin (the target triple in the build). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Arm AAPCS64 and Architecture Reference Manual, the System V x86-64 ABI, Apple's arm64 ABI notes.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/assembly.md).

Canonical source: `plugins/web-development/backend/languages/assembly/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/assembly.md` (reference; do not author).
