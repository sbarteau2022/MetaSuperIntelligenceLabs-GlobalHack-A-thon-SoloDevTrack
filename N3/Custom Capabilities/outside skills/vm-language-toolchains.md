---
title: VM language toolchains and harness gates
status: implemented-with-explicit-platform-exceptions
date: 2026-10-08
---

# VM language toolchains and harness gates

## Purpose

The `languages` plugin contains 41 language skills. A language named in a task, or written to disk, is activated by the Rust harness. Before it can finish, the model must successfully call that language's declared runner. A source-file write additionally requires a successful configured checker. A failed checker never satisfies the gate, so the model is sent back to revise and re-run it until it passes or the bounded agent budget stops the turn.

`vm_exec`, `run_shell`, and `run_python` expose nonzero process exits as failed tool calls while preserving the command's diagnostic tail. Consequently a failed test cannot be counted as either a runner use or a passing checker.

## Provisioning

`vm/provision-language-toolchains.sh` installs the redistributable Linux toolchains into a Lima Ubuntu guest. It is idempotent and writes `/etc/atoma/language-toolchains.txt`. Run it as root with `ATOMA_VM_USER=stewartbarteau` (the guest's account name, not its home-directory suffix). The image needs at least 6 CPUs, 16 GiB RAM, and an 80 GiB disk for compilers and build caches.

Metal cannot be compiled in a Linux VM: its compiler is Xcode's macOS-only `xcrun metal`. MATLAB is licensed software and cannot be silently installed; Octave is installed for open `.m` test coverage. Dart, PowerShell (the supplied Linux package is x86_64-only), Swift, Zig, Terraform, HLSL DXC, and Solidity need their upstream ARM64 distribution before a project depends on them. The harness must report such unavailable runners/checkers, never claim they passed.

## Verification

Run the provisioning script in the VM, then record the output of the relevant `--version` command and the project's test command. The core harness test suite includes a 41-language plugin consistency check and gate tests covering: runner required, checker required after a write, and a failed checker not satisfying the gate.
