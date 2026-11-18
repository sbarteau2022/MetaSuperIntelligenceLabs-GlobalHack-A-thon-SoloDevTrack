# powershell: strict mode, advanced functions, object pipeline
**Done means these pass:** `Invoke-ScriptAnalyzer -Path . -Recurse -Severity Warning,Error` (no findings) and `Invoke-Pester -CI`.
## Rules
1. Start scripts with `Set-StrictMode -Version Latest` and `$ErrorActionPreference = 'Stop'`. Use `[CmdletBinding()]` and a `param()` block with types, `[Parameter(Mandatory)]`, `[ValidateSet()]`/`[ValidateNotNullOrEmpty()]`.
2. Verb-Noun function names with approved verbs (`Get-Verb`), singular nouns. Support `-WhatIf`/`-Confirm` (`SupportsShouldProcess`) on anything that changes state.
3. Output objects, not formatted text: never `Write-Host` for data (use `Write-Output`/return objects, `Write-Verbose`, `Write-Warning`, `Write-Error`); do not use `Format-*` cmdlets inside functions that return data; `Select-Object` and `Where-Object` filter early (`-Filter` parameters over `Where-Object` where available).
4. Quote and splat: use splatting (`@params`) for long calls; use single quotes unless interpolating; compare with `-eq`, `-ceq`; `$null` goes on the left (`$null -eq $x`); beware arrays unrolling in the pipeline (`,@($x)` or `Write-Output -NoEnumerate`).
5. Errors: `try`/`catch`/`finally` with `-ErrorAction Stop` on the commands inside; catch specific exception types; use `$PSCmdlet.ThrowTerminatingError`; do not swallow with empty `catch {}`; check `$LASTEXITCODE` after native commands.
6. Security: never `Invoke-Expression` on input; avoid plaintext secrets (use `SecretManagement`, `[SecureString]`, `Get-Credential`); sign scripts where policy requires; pass native-command arguments as arrays, not concatenated strings; validate paths with `-LiteralPath`.
7. Portability: target PowerShell 7 and state if Windows PowerShell 5.1 must work; use `Join-Path`, `[IO.Path]`, and `$IsWindows`; avoid aliases (`gci`, `%`, `?`) in scripts.
8. Modules: a `.psd1` manifest, exported functions listed explicitly, comment-based help (`.SYNOPSIS`, `.EXAMPLE`).
9. Tests: Pester v5 (`Describe`/`It`/`Should`, `Mock`, `BeforeAll`); no real network or registry in unit tests.
## Pitfalls
Single-item vs array results (`@()` them); `-eq` on collections filters instead of compares; `$_` scope inside nested pipelines; `-Force` hiding problems; string vs object comparison; `Out-Null` hiding errors.
## Versions
Pinned: PowerShell 7.6 LTS (7.6.4 LTS; 7.5.9 is the current stable per Microsoft's lifecycle page; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least PowerShell 7.2.
Rule: first run `pwsh --version` and read the project's pin (#Requires -Version, the module manifest PowerShellVersion). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the PowerShell Practice and Style guide, PSScriptAnalyzer rules, Microsoft's cmdlet development guidelines.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/powershell.md).

Canonical source: `plugins/web-development/middleware/languages/powershell/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/powershell.md` (reference; do not author).
