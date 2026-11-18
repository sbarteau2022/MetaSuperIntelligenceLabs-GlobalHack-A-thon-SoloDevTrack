# shell: quote everything and fail fast
**Done means these pass:** `shellcheck -x script.sh`, `shfmt -d script.sh`, `bash -n script.sh`; tests with `bats` if the script is more than a few lines.
## Rules
1. Start with `#!/usr/bin/env bash` and `set -euo pipefail`. Know that `set -e` is ignored in conditions and in `local x=$(cmd)`.
2. Quote every expansion: `"$var"`, `"${arr[@]}"`, `"$(cmd)"`. Use `[[ ... ]]` in bash. Use `$(...)`, never backticks.
3. Never parse `ls`. Use globs, or `find ... -print0` with `while IFS= read -r -d '' f`. End options with `--` before user-supplied names.
4. Temp files: `tmp=$(mktemp -d)` plus `trap 'rm -rf "$tmp"' EXIT`. Guard destructive paths: `rm -rf "${dir:?}/"`. `cd "$dir" || exit 1`.
5. Validate arguments, print a usage message on error, send errors to stderr, and return meaningful exit codes. Check required tools with `command -v`.
6. Functions use `local`. Use `printf`, not `echo`, for data. No `eval`. Use arrays for lists of arguments instead of word-splitting a string.
7. Secrets never go in arguments (visible in `ps`) or in logs. Use environment files with tight permissions or a secret manager.
8. `curl --fail --show-error --location --max-time 30`; verify downloads (checksums) before executing them.
9. Portability: if the shebang is `sh`, no arrays, no `[[`, no `local` assumptions; macOS tools differ from GNU (`sed -i ''`). macOS ships bash 3.2: unless you have checked `bash --version`, avoid `mapfile`/`readarray`, `declare -A`, negative array indexes (`${a[-1]}`), `${v,,}`/`${v^^}` and `|&`, or start with a guard such as `(( BASH_VERSINFO[0] >= 4 )) || { echo 'needs bash 4+' >&2; exit 1; }`. If a script grows past about 100 lines or needs real data structures, rewrite it in Python.
10. Make scripts idempotent: safe to run twice.
10. On a bash 3.2 target (macOS `/bin/bash`) never write `mapfile`, `declare -A`, or a negative index. Keep a stack as a string or a plain array with a separate length variable, and test the script with `/bin/bash` before finishing.
## Pitfalls
`cmd | while read` runs in a subshell (variables are lost); `[ $x = y ]` with an empty `$x`; a glob that matches nothing stays literal; word splitting on spaces; `set -e` hiding failures inside `$(...)`.
## Versions
Pinned: Bash 5.3 (macOS /bin/bash here is 3.2.57; Bash 5.3 is from knowledge; checked 2026-10-07, source: this machine; 5.3 not verified online). This skill assumes at least Bash 3.2.
Rule: first run `bash --version` and read the project's pin (the script's shebang, CI image). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Bash manual, ShellCheck wiki, Google Shell Style Guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/shell.md).

Canonical source: `plugins/web-development/middleware/languages/shell/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/shell.md` (reference; do not author).
