# Agent Harness Audit

Complete inventory of the Atoma agent harness: tools, languages, skills, plugins, installation, and gaps.

---

## 1. Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Atoma (Tauri desktop app)                                               │
│  macOS/iOS · Vite + TS frontend · seatbelt sandbox                       │
├──────────────────────────────────────────────────────────────────────────┤
│  Rust core (home-core)                                                   │
│  agent · tools · sandbox · browser · cron · journal · vmlog              │
│  connectors · pluginadmin · approval · enforce · think                   │
├──────────────────────────────────────────────────────────────────────────┤
│  Python harness (home/)                                                  │
│  CLI · agent · ingest · store (hybrid RAG) · memory · docgen             │
│  embed (model2vec, local) · flows (LangGraph)                            │
├──────────────────────────────────────────────────────────────────────────┤
│  Postgres 17 + pgvector                                                  │
│  127.0.0.1:5433 · data in ~/Library/Application Support/ModelHome/pg     │
│  (launchd job com.home.postgres)                                         │
├──────────────────────────────────────────────────────────────────────────┤
│  Lima microVM (6 CPU, 16 GiB, 80 GiB)                                    │
│  Ubuntu 24.04 ARM · 41 language toolchains · isolated build              │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Agentic Tools — Complete Inventory

### 2.1 File and Workspace

| Tool | Origin | Description |
|:---|:---|:---|
| `read_file` | Rust | Read a UTF-8 text file from the workspace (path-confined via `confine()`). |
| `write_file` | Rust | Write (overwrite) a text file in the workspace; creates parent dirs. |
| `list_files` | Rust | List files under a workspace directory (recursive). |

### 2.2 Knowledge and Memory

| Tool | Origin | Description |
|:---|:---|:---|
| `kb_search` | Rust | Hybrid semantic + keyword search over the Postgres knowledge base (dense cosine HNSW + full-text GIN, fused by reciprocal rank). Supports `collection` filter. |
| `memory` | Rust | Persistent facts, preferences, episodes, session summaries. CRUD via `home mem add/search/update/forget/pinned`. Hybrid-searchable. |
| `journal` | Rust | Write journal entries. Stored in Postgres (`rs_journal`). Author can be "model" or "user". |
| `journal_read` | Rust | Read journal entries. |
| `scratch` | Rust | Ephemeral scratchpad for transient notes. Stored in Postgres (`rs_scratch`). |

### 2.3 Code Execution (Sandboxed)

| Tool | Origin | Description |
|:---|:---|:---|
| `run_shell` | Rust | Run an allowlisted program (`ls`, `cat`, `head`, `tail`, `wc`, `grep`, `python3`, `pytest`) in the workspace. No shell metacharacters. 60 s timeout. |
| `run_python` | Rust | Run a short Python 3 program in the sandbox and get what it prints. One call instead of write + shell. 100 KB max, 60 s limit. |
| `vm_exec` | Rust | Run commands inside the Lima microVM (full toolchains, no workspace mount — files move via approved tools). |

### 2.4 Documents and Artifacts

| Tool | Origin | Description |
|:---|:---|:---|
| `read_document` | Rust | Read any supported document (PDF with page markers, DOCX, PPTX, XLSX, EPUB, HTML, text) as text. Invokes `home.cli parse`. |
| `write_document` | Rust | Create a document or artifact: Markdown, HTML, DOCX, or PDF (rendered via VM browser). Files go in `artifacts/` and appear in the chat. |

### 2.5 Web and Browser

| Tool | Origin | Description |
|:---|:---|:---|
| `web_search` | Rust | Search via SearXNG (`HOME_SEARCH_URL`). Returns top 8 results. |
| `web_fetch` | Rust | Hardened HTTP fetch: SSRF-checked, per-hop redirects (max 5), 2 MB size cap, HTML-to-text extraction. |
| `browser_open` | Rust | Open a web page in the browser (Chromium + Playwright in the Lima VM). Returns interactive elements (numbered refs) and visible text. |
| `browser_snapshot` | Rust | Re-read the current page: URL, title, numbered interactive elements and text. |
| `browser_click` | Rust | Click an element by its ref from the latest snapshot. Irreversible clicks (buy, delete, send, submit) require user approval. |
| `browser_type` | Rust | Type text into an input by ref. Password fields are refused. |
| `browser_press` | Rust | Press a key (Enter, Escape, Tab, ArrowDown, PageDown, etc.). |
| `browser_scroll` | Rust | Scroll the page (up, down, top, bottom). |
| `browser_back` | Rust | Go back one page in history. |
| `browser_tabs` | Rust | Manage tabs: list, switch, close, or open new. |
| `browser_extract` | Rust | Return the text of part of the page by CSS selector. |

### 2.6 Knowledge Base Management

| Tool | Origin | Description |
|:---|:---|:---|
| `ingest` | Rust | Index a directory into a knowledge-base collection. Idempotent: unchanged files skipped, changed files replaced. Supports `.md`, `.txt`, `.py`, `.rs`, `.ts`, `.html`, `.pdf`, `.docx`, `.pptx`, `.xlsx`, `.epub`. |

### 2.7 Workflow and Plugin System

| Tool | Origin | Description |
|:---|:---|:---|
| `skill_read` | Rust | Read a skill file from an enabled plugin or the `skills/` directory. |
| `skill_write` | Rust | Write a skill file (editable persona, custom skills). |
| `skill_list` | Rust | List installed plugins and their skills. |
| `workflow_read` | Rust | Read a workflow definition from `<workspace>/flows/`. |
| `workflow_write` | Rust | Write a workflow definition (compiled LangGraph). |
| `workflow_list` | Rust | List all LangGraph workflows in `<workspace>/flows/`. |
| `plugin_create` | Rust | Create a new plugin (self-built tool). |
| `connector_write` | Rust | Add a tested, approved Python connector (tool) to a plugin. |

### 2.8 Scheduling

| Tool | Origin | Description |
|:---|:---|:---|
| `schedule` | Rust | Create a recurring cron job. Schedules live in Postgres (`rs_crons`). Polls every 30 s. |

### 2.9 Thinking and Budget Control

| Tool | Origin | Description |
|:---|:---|:---|
| `think` | Rust | Allocate a larger thinking budget before costly, hard-to-undo decisions, multi-step plans, tricky bugs, or after two failures. The model's next step gets a much larger thinking budget (see §3). |

### 2.10 Data Movement (Approval-Gated)

| Tool | Origin | Description |
|:---|:---|:---|
| `clone_in` | Rust | Copy files into the workspace (approval-gated via `Approver` trait). |
| `ship_out` | Rust | Copy files out of the workspace (approval-gated via `Approver` trait). |

### 2.11 Cognitive Map

| Tool | Origin | Description |
|:---|:---|:---|
| `cogmap_brief` | Rust | Get a brief overview of the cognitive map state (enrollment, curriculum, goal, templates). |
| `cogmap_map` | Rust | Get the full concept map (nodes, edges, dimensions). |
| `cogmap_predict` | Rust | Predict the next concept to study based on prerequisites and progress. |
| `cogmap_submit` | Rust | Submit an answer or assessment for a concept. |

### 2.12 Python-Harness Tools (via `home.cli`)

| Tool | Origin | Description |
|:---|:---|:---|
| `home ingest` | Python | Idempotent directory ingestion into a knowledge-base collection. |
| `home search` | Python | Hybrid KB search (dense cosine + full-text, RRF-fused). |
| `home ask` | Python | Run the agent on a task, resumed from Postgres checkpoint. |
| `home status` | Python | Collection stats (files, chunks per collection). |
| `home mem add` | Python | Add a memory entry (kind: fact, preference, episode, session). |
| `home mem search` | Python | Search memories (hybrid semantic + keyword). |
| `home mem update` | Python | Update a memory entry. |
| `home mem forget` | Python | Delete a memory entry. |
| `home mem pinned` | Python | List pinned memories. |
| `home doc` | Python | Convert markdown → styled HTML or `.docx` (zero third-party deps). |
| `home parse` | Python | Parse a document (PDF, DOCX, PPTX, XLSX, EPUB, HTML, text) into readable text. |
| `home flow list` | Python | List workflows in `<workspace>/flows/`. |
| `home flow run` | Python | Stream a workflow execution. |

### 2.13 Direct Engineering and Research

| Tool | Origin | Description |
|:---|:---|:---|
| `detect_toolchains` | Rust + Python | Detect the primary local engineering stacks and declared npm scripts. |
| `plan_language_check` | Rust + Python | Return one fixed validation command without running it. |
| `run_language_check` | Rust + Python | Run one fixed validation command with bounded time and output. |
| `search_arxiv` | Rust + Python | Search read-only arXiv paper metadata. |
| `search_huggingface_models` | Rust + Python | Search read-only Hugging Face model metadata. |

**Total: 47 distinct tool capabilities.**

---

## 3. Thinking Budget System

The `think` tool is a dial, not a toggle. The agent loop reads the current `Policy` and selects a thinking budget per step based on context:

| Mode | Steady | First (plan) | Retry (after failure) | Asked (via `think`) |
|:---|:---:|:---:|:---:|:---:|
| `quick` | 256 | 256 | 1,024 | 16,384 |
| `auto` (default) | 1,024 | 3,072 | 4,096 | 16,384 |
| `think` | 4,096 | 8,192 | 4,096 | 16,384 |
| `deep` | 16,384 | 16,384 | 16,384 | 16,384 |

The budget is sent as `thinking_budget` to the oMLX server. `reasoning_effort` is deliberately never sent (it rewrites the rendered prompt and forces a full re-prefill, breaking the prefix cache).

---

## 4. Language Plugin — Complete Inventory (41 Languages)

Each language has: file extensions, trigger words, a skill reference, checker commands, version info, and tool bindings.

| # | Language | Extensions | Skill | Checkers | Version (pinned) | Source |
|:---|:---|:---|:---|:---|:---|:---|
| 1 | Python | `.py`, `.pyi` | `python` | ruff, mypy, pyright, pytest, flake8, pylint, unittest | Python 3.14 | this machine |
| 2 | JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` | `javascript` | eslint, prettier, vitest, jest, mocha, node --test, biome | Node.js 24 LTS | this machine and web |
| 3 | TypeScript | `.ts`, `.tsx`, `.mts`, `.cts` | `typescript` | tsc, eslint, prettier, vitest, jest, biome | TypeScript 7.0 | web |
| 4 | Rust | `.rs` | `rust` | cargo clippy, cargo test, cargo check, cargo fmt, rustfmt, cargo nextest | Rust 1.99 | this machine |
| 5 | Go | `.go` | `go` | go vet, go test, gofmt, staticcheck, golangci-lint | Go 1.26 | web |
| 6 | Java | `.java` | `java` | mvn, mvnw, gradle, gradlew, javac, checkstyle, spotbugs | Java 25 LTS | knowledge |
| 7 | Kotlin | `.kt`, `.kts` | `kotlin` | gradle, gradlew, ktlint, detekt, kotlinc | Kotlin 2.3 | web |
| 8 | C# | `.cs` | `csharp` | dotnet build, dotnet test, dotnet format | C# 14 on .NET 10 | web |
| 9 | C | `.c`, `.h` | `c` | gcc, clang, cmake, make, cppcheck, clang-tidy, valgrind | C23 | this machine |
| 10 | C++ | `.cc`, `.cpp`, `.cxx`, `.hpp`, `.hh`, `.hxx` | `cpp` | g++, clang++, cmake, make, ctest, clang-tidy, cppcheck | C++23 | this machine |
| 11 | Swift | `.swift` | `swift` | swift build, swift test, swiftlint, swift-format, xcodebuild | Swift 6.4 | this machine |
| 12 | SQL | `.sql` | `sql` | sqlfluff, psql, sqlite3, pg_prove | SQL:2023, PG 18 | this machine |
| 13 | Shell | `.sh`, `.bash`, `.zsh` | `shell` | shellcheck, shfmt, bash -n, sh -n, bats | Bash 5.3 | this machine |
| 14 | Ruby | `.rb` | `ruby` | rubocop, rspec, rake, ruby -c, ruby -wc, brakeman | Ruby 4.0 | this machine |
| 15 | PHP | `.php` | `php` | phpstan, psalm, phpunit, php -l, phpcs, php-cs-fixer, pest | PHP 8.5 | web |
| 16 | Dart | `.dart` | `dart` | dart format, dart analyze, dart test, flutter analyze, flutter test | Dart 3.11 | web |
| 17 | Objective-C | `.m`, `.mm` | `objc` | clang, xcodebuild, clang-format, xctest | Apple clang 21 | this machine |
| 18 | R | `.r`, `.rmd` | `r` | lintr, styler, testthat, devtools::test, r cmd check | R 4.6 | web |
| 19 | Julia | `.jl` | `julia` | pkg.test, juliaformatter, aqua, jet | Julia 1.12 | web |
| 20 | MATLAB | (none) | `matlab` | runtests, checkcode, mlint, matlab -batch, octave | latest MathWorks | host-bound |
| 21 | Octave | `.m` | `matlab` | octave | GNU Octave | VM |
| 22 | Fortran | `.f90`, `.f95`, `.f03`, `.f08`, `.f`, `.for` | `fortran` | gfortran, fpm test, fprettify, flang | Fortran 2023 | web |
| 23 | Lean | `.lean` | `lean` | lake build, lake test, lean | Lean 4.34 | web |
| 24 | Scala | `.scala`, `.sc` | `scala` | sbt, scala-cli, scalafmt, scalafix, mill | Scala 3.8 | web |
| 25 | Elixir | `.ex`, `.exs` | `elixir` | mix format, mix credo, mix dialyzer, mix test, mix compile | Elixir 1.20 | web |
| 26 | Erlang | `.erl`, `.hrl` | `erlang` | rebar3, erlc, dialyzer, escript | OTP 28 | web |
| 27 | Perl | `.pl`, `.pm` | `perl` | perlcritic, perltidy, prove, perl -c, perl -wc | Perl 5.42 | not verified |
| 28 | Groovy | `.groovy`, `.gvy` | `groovy` | codenarc, gradlew, gradle, npm-groovy-lint | Groovy 5.0 | not verified |
| 29 | Haskell | `.hs`, `.lhs` | `haskell` | cabal build, cabal test, stack test, hlint, ormolu, fourmolu, ghc -wall | GHC 9.14 | web |
| 30 | OCaml | `.ml`, `.mli` | `ocaml` | dune build, dune test, dune runtest, ocamlformat | OCaml 5.4 | web |
| 31 | F# | `.fs`, `.fsx`, `.fsi` | `fsharp` | dotnet build, dotnet test, fantomas, dotnet fsi | F# 10 on .NET 10 | knowledge |
| 32 | Clojure | `.clj`, `.cljs`, `.cljc`, `.edn` | `clojure` | clj-kondo, cljfmt, lein test, clojure -x, clojure -m, clj -x, clj -m, kaocha | Clojure 1.12 | web |
| 33 | Zig | `.zig` | `zig` | zig fmt, zig build, zig test | Zig 0.16 | web |
| 34 | Assembly | `.asm`, `.s` | `assembly` | nasm, yasm, clang -c, gcc -c, llvm-mc, objdump, otool | AArch64 + x86-64 | this machine |
| 35 | Lua | `.lua` | `lua` | luacheck, stylua, busted, luau-analyze | Lua 5.4 | not verified |
| 36 | PowerShell | `.ps1`, `.psm1`, `.psd1` | `powershell` | invoke-scriptanalyzer, invoke-pester | PowerShell 7.6 | web |
| 37 | Terraform | `.tf`, `.tfvars`, `.hcl` | `terraform` | terraform fmt, terraform validate, tflint, tofu validate, tofu fmt, checkov, trivy config | Terraform 1.15 | web |
| 38 | GLSL | `.glsl`, `.vert`, `.frag`, `.geom`, `.comp`, `.tesc`, `.tese` | `glsl` | glslangvalidator, glslc, spirv-val | GLSL 4.60 | not verified |
| 39 | HLSL | `.hlsl`, `.hlsli`, `.fx` | `hlsl` | dxc, fxc | Shader Model 6.8 | not verified |
| 40 | Metal | `.metal` | `metal` | xcrun metal, metallib, metal -std | MSL 4.0 | host-bound |
| 41 | Solidity | `.sol` | `solidity` | forge test, forge build, forge fmt, slither, solhint, solc, hardhat test | Solidity 0.8.37 | web |
| 42 | COBOL | `.cob`, `.cbl`, `.cobol`, `.cpy` | `cobol` | cobc, cobol-check | GnuCOBOL 3.2 | web |

### 4.1 Version Verification Status

| Source | Count | Languages |
|:---|:---:|:---|
| **this machine** | 10 | Python, JS, Rust, C, C++, Swift, SQL, Shell, Ruby, ObjC, Assembly |
| **web (verified Sep 2026)** | 17 | TypeScript, Go, Kotlin, C#, PHP, Dart, R, Julia, Fortran, Lean, Scala, Elixir, Erlang, Haskell, OCaml, PowerShell, Terraform, Solidity, COBOL |
| **knowledge (unverified)** | 4 | Java, F#, GLSL, HLSL |
| **not verified online** | 5 | Perl, Groovy, Lua, MATLAB, Metal |
| **host-bound** | 2 | MATLAB (licensed), Metal (Xcode only) |
| **VM-installed** | 1 | Octave |

### 4.2 Tool Bindings

| Tool | Languages |
|:---|:---|
| `run_python` | Python |
| `vm_exec` | All 41 other languages |

---

## 5. Skills — Complete Inventory

### 5.1 Core Skills (in `skills/`)

| Skill | Purpose | Editable |
|:---|:---|:---|
| `persona.md` | Editable agent personality. Starts nearly blank. | Yes (`skill_write`) |
| `engineering.md` | Engineering best practices, adversarial pressure pass, build loop (design-spec → preregister → test-suites → implement → adversarial). | Yes |
| `web-design.md` | Semantic HTML, WCAG 2.2 AA, responsive layout, performance budgets. | Yes |
| `landing-page-checklist.md` | 5-item checklist for shipping a landing page (derived from web-design). | Yes |
| `voice-dickens.md` | Dickens register: comic abundance, periodic sentences, parallel lists, irony. | Yes |
| `voice-lewis.md` | C.S. Lewis register: conversational authority, homely analogy, dry wit. | Yes |
| `voice-tolstoy.md` | Tolstoy register: omniscient, morally attentive, concrete detail, long balanced sentences. | Yes |
| `voice-technical.md` | Technical documentation: define terms, measurable claims, numbered steps, failure modes. | Yes |

### 5.2 Plugin Skills (dynamically loaded)

| Plugin | Skills (loaded by `DocTools::all`) |
|:---|:---|
| `languages` | One skill per language (41 total): `python`, `javascript`, `typescript`, `rust`, `go`, `java`, `kotlin`, `csharp`, `c`, `cpp`, `swift`, `sql`, `shell`, `ruby`, `php`, `dart`, `objc`, `r`, `julia`, `matlab`, `octave`, `fortran`, `lean`, `scala`, `elixir`, `erlang`, `perl`, `groovy`, `haskell`, `ocaml`, `fsharp`, `clojure`, `zig`, `assembly`, `lua`, `powershell`, `terraform`, `glsl`, `hlsl`, `metal`, `solidity`, `cobol` |
| `engineering` | `design-spec`, `preregister`, `test-suites`, `adversarial-pressure`, `code-review`, `debug`, `architecture`, `documentation`, `tech-debt`, `incident-response`, `deploy-checklist` |
| `design` | `plugin-builder` (enforced on build skill/plugin/connector/tool) |

---

## 6. Plugin System

### 6.1 Installed Plugins

| Plugin | Description | Enforced Rules |
|:---|:---|:---|
| `languages` | 41 languages, each with skill, checkers, version info, and tool bindings. | None (auto-loaded) |
| `engineering` | Adversarial pressure pass (enforced), review, debug, architecture, docs, tech debt, incidents, deploys. | `adversarial-pressure` skill enforced when grading/auditing/stress-testing; requires `run_python` or `vm_exec`. |
| `design` | Design-first workflow: spec, sealed claims, test suites with mutation checker, plugin/skill/connector builder. | `plugin-builder` skill enforced when building skills/plugins/connectors/tools; requires `plugin_create`, `connector_write`, `skill_write`. |

### 6.2 Plugin Lifecycle

1. **Create** — `plugin_create` tool creates a new plugin directory with `plugin.json`.
2. **Enable** — `pluginadmin::set_enabled` toggles the switch (outside the plugin files).
3. **Load** — `enforce::load` reads enabled plugins, parses `enforce` rules into `Rule` structs.
4. **Commit** — `pluginadmin::commit` records a local git commit and marks the plugin as committed.
5. **Recommit** — `pluginadmin::recommit` records later edits for any plugin.
6. **Delete** — `pluginadmin::delete` removes the plugin and its switch.

---

## 7. Installation and Setup

### 7.1 Prerequisites

| Component | Requirement |
|:---|:---|
| **OS** | macOS (Tauri, Metal, seatbelt sandbox) |
| **Postgres** | 17 + pgvector extension, running on `127.0.0.1:5433` |
| **oMLX server** | OpenAI-compatible API at `127.0.0.1:8000/v1` |
| **embedder** | Local `model2vec` model2vec directory (default: cached `potion-code-16M`) |
| **Python** | ≥ 3.12, managed by `uv` |
| **Lima** | For microVM (`limactl`) |
| **Node.js** | ≥ 20 (for Tauri build) |
| **Rust** | ≥ 1.75 (for core compilation) |

### 7.2 VM Provisioning

```bash
limactl create --name home vm/home.yaml && limactl start home
limactl shell home -- sudo -n env ATOMA_VM_USER=<user> bash -s < vm/provision-language-toolchains.sh
```

The provision script installs:
- **Build tools**: `build-essential`, `clang`, `cmake`, `make`, `ninja-build`, `valgrind`
- **Compilers**: `g++`, `gfortran`, `clang`, `clang-tidy`, `gcc`
- **Languages**: Go, Java, Kotlin, Ruby, PHP, R, Scala, Elixir, Erlang, Haskell, OCaml, Clojure, Lua, COBOL, .NET 10
- **Interpreters**: Python 3, Node.js 22, Rust (stable + clippy + rustfmt)
- **Shaders**: `glslang-tools`, `spirv-tools`, `octave`
- **Utilities**: `git`, `curl`, `shellcheck`, `shfmt`, `sqlite3`, `pkg-config`

### 7.3 Browser Setup

The browser server (`Chromium + Playwright`) runs in the Lima VM, accessed via loopback port 9333. Authentication is via a token at `~/Library/Application Support/engineering.atoma.modelhome/browser.token`.

### 7.4 Postgres Setup

Managed by launchd job `com.home.postgres`. Data directory: `~/Library/Application Support/ModelHome/pg`.

### 7.5 Python Environment

```bash
uv sync          # installs from pyproject.toml
uv run home      # runs the CLI
```

Dependencies: `langchain`, `langchain-openai`, `langchain-text-splitters`, `langgraph`, `langgraph-checkpoint-postgres`, `psycopg`, `pgvector`, `model2vec`, `pypdf`, `numpy`.

### 7.6 Rust Build

```bash
cargo build      # builds home-core, home-bridge, home-metal
```

Dependencies: `anyhow`, `async-trait`, `futures`, `reqwest`, `serde`, `serde_json`, `sha2`, `tokio`, `tokio-postgres`, `cron`, `chrono`, `url`, `base64`, `metal`.

---

## 8. Gaps and Missing Components

### 8.1 Missing Language Checkers (not installed in VM)

| Language | Missing Checker | Impact |
|:---|:---|:---|
| Dart | `dart`, `flutter` | Listed as `requires-separate-distribution` in inventory |
| PowerShell | `pwsh` | Microsoft package is x86_64-only on Ubuntu arm64 |
| Swift | `swift`, `xcodebuild` | Requires macOS host (not Linux VM) |
| Zig | `zig` | Listed as `requires-separate-distribution` |
| Terraform | `terraform` | Listed as `requires-separate-distribution` |
| Metal | `metal`, `metallib` | Requires macOS host Xcode |
| MATLAB | `matlab` | Requires licensed MathWorks installation |

### 8.2 Missing Version Verification

| Language | Status | Action Needed |
|:---|:---|:---|
| Java | "not verified online; from knowledge" | Verify `java -version` on a fresh install |
| F# | "from knowledge, with .NET 10 confirmed" | Verify `dotnet fsi --version` |
| GLSL | "not verified online; from knowledge" | Verify `glslangValidator --version` |
| HLSL | "not verified online; from knowledge" | Verify `dxc --version` |
| Perl | "not verified online" | Verify `perl -e 'print $]'` |
| Groovy | "not verified online" | Verify `groovy --version` |
| Lua | "not verified online" | Verify `lua -v` |

### 8.3 Missing Tooling

| Gap | Description | Priority |
|:---|:---|:---|
| **No `run_shell` for VM** | `run_shell` only works in the seatbelt sandbox on the Mac. The VM has `vm_exec` but no equivalent allowlisted command runner in the sandbox. | Medium |
| **No language-specific runner** | Each language uses `vm_exec` (generic command runner). There's no `run_<lang>` tool that automatically invokes the correct compiler/interpreter. | Low |
| **No mutation testing** | The engineering skill mentions "mutation check" but no mutation testing tool (e.g., `mutmut` for Python, `schemathesis` for APIs) is configured. | Medium |
| **No container runtime** | `home-bridge` depends on `axum` but there's no Docker/Podman container backend for sandboxing. The `sandbox.rs` has `Backend::Container` but it references `SANDBOX_EXEC` which appears to be a stub. | High |
| **No wander mode scheduler** | `wander()` exists in `standard.rs` but there's no cron entry or auto-start for it. | Low |
| **No browserd.py** | The browser module expects `~/browser/run.sh` and `~/browser/browserd.py` in the VM, but these files are not in the repo. | High |
| **No tests directory** | `tests/` directory exists but is empty. Rust tests exist in `home-core/src/tools.rs` and `home-core/src/sandbox.rs` but no integration tests. | Medium |
| **No CI/CD** | No GitHub Actions, no automated build pipeline. | Medium |
| **No `docs/` content** | Only `vm-language-toolchains.md` and `langeval/` exist. No API docs, no architecture docs. | Medium |

### 8.4 Schema Migration Gaps

| Table | Issue |
|:---|:---|
| `rs_journal` | Has ALTER TABLE for backward compatibility with older databases that lacked `id`, `author`, `reply_to` columns. |
| `rs_messages` | Uses `ON CONFLICT DO NOTHING` (no unique constraint on thread+seq beyond primary key). |

---

## 9. BUILD LIST

Items that should be completed before production readiness:

### P0 — Blockers

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 1 | **Browser server in VM** | infra | `browserd.py` and `run.sh` must exist in the Lima VM at `~/browser/` for any browser tool to work. |
| 2 | **Postgres launchd job** | infra | `com.home.postgres` must be installed and running. Data at `~/Library/Application Support/ModelHome/pg`. |
| 3 | **oMLX server running** | infra | OpenAI-compatible API at `127.0.0.1:8000/v1` must be reachable. |
| 4 | **Embedder model available** | infra | `potion-code-16M` model2vec directory must be cached at `~/.cache/huggingface/hub/`. |

### P1 — High Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 5 | **Container sandbox backend** | infra | `Backend::Container` in `sandbox.rs` references `SANDBOX_EXEC` but the implementation appears incomplete. Without this, `run_shell` and `run_python` are limited to the seatbelt sandbox. |
| 6 | **VM provision script validation** | infra | Run the provision script on a fresh VM image and verify all 41 language checkers report correct versions. |
| 7 | **Version verification sweep** | dev | Verify the 7 languages marked "not verified" or "from knowledge". Update `plugin.json` checked dates. |
| 8 | **Integration test suite** | dev | `tests/` is empty. Add tests that exercise the full agent loop (agent → tools → Postgres → VM). |
| 9 | **Documentation** | docs | Add architecture overview, API reference, and deployment guide to `docs/`. |

### P2 — Medium Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 10 | **CI/CD pipeline** | infra | GitHub Actions for Rust build + test, Python lint, Tauri build. |
| 11 | **Mutation testing** | dev | Add a mutation testing tool (e.g., `mutmut` for Python) and wire it into the engineering skill. |
| 12 | **Wander mode auto-start** | infra | Add a cron entry or launchd job to run wander mode periodically. |
| 13 | **Language-specific runners** | dev | Add `run_python`, `run_rust`, `run_golang` etc. that auto-invoke the correct compiler/interpreter in the VM. |
| 14 | **Schema migration tests** | dev | Tests that verify backward compatibility for `rs_journal` and `rs_messages` schema migrations. |

### P3 — Low Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 15 | **Dart/PowerShell/Zig toolchains** | infra | Add these to the VM provision script once distribution packages are available for Ubuntu arm64. |
| 16 | **Metal/MATLAB fallback** | infra | Surface these as "unavailable" in the language plugin rather than claiming they are installed. |
| 17 | **Performance benchmarks** | dev | Measure thinking budget cache hit rates, embedder throughput, RAG latency. |
| 18 | **Telemetry / usage stats** | infra | Track token usage, tool call frequency, agent step counts. |

---

## 10. Module Map

| Module | File | Responsibility |
|:---|:---|:---|
| `agent` | `agent.rs` | Agent loop: streaming chat, tool dispatch, budget control, context compaction, summarization. |
| `llm` | `llm.rs` | oMLX OpenAI-compatible client: streaming chat, tool call parsing, reasoning extraction. |
| `tools` | `tools.rs` | `Tool` trait, `ReadFile`, `WriteFile`, `ListFiles`, `KbSearch`, `RunShell`, `RunPython`, `confine()`. |
| `extra` | `extra.rs` | `WebFetch`, `WebSearch`, `Ingest`, `VmExec`, `CloneIn`, `ShipOut`, `Schedule`, `Journal`, `JournalRead`, `Scratch`, `Memory`, `DocTools`. |
| `sandbox` | `sandbox.rs` | `Backend` enum (Container, Vm, Seatbelt), Lima integration, profile definitions. |
| `browser` | `browser.rs` | Chromium/Playwright in Lima VM: open, snapshot, click, type, press, scroll, back, tabs, extract. |
| `web` | `web.rs` | Hardened HTTP fetch (SSRF-checked), HTML-to-text, SearXNG search. |
| `think` | `think.rs` | Thinking budget dial: quick/auto/think/deep modes, per-step budget policy. |
| `approval` | `approval.rs` | Human-in-the-loop gate: `Approver` trait, `DenyAll`, `ChannelApprover`. |
| `enforce` | `enforce.rs` | Plugin enforcement rules, language gate (skill + checker verification). |
| `pluginadmin` | `pluginadmin.rs` | Plugin CRUD, skill read/write, catalog, connectors, git commit tracking. |
| `connectors` | `connectors.rs` | `PluginCreate`, `ConnectorWrite` — model-authored tools with approval gate. |
| `cron` | `cron.rs` | Durable cron: schedules in Postgres, 30s poll, unattended runner. |
| `store` | `store.rs` | Postgres thread checkpoints: append, load, replace_head, threads list. |
| `api` | `api.rs` | UI backend: journal, scratch, session close. Pure functions over Postgres. |
| `session` | `session.rs` | Session consolidation: summary → memory, verbatim tail, preamble seeding. |
| `nudge` | `nudge.rs` | User messages during agent mid-turn: queue, drain, frame. |
| `standard` | `standard.rs` | Standard wiring: work agent, unattended runner, wander mode. |
| `cogmap_tools` | `cogmap_tools.rs` | Cognitive map tools: brief, map, predict, submit. Event-sourced state. |
| `vmlog` | `vmlog.rs` | Live VM feed: line publishing, history, code action tracking. |
| `belt` | `belt.rs` | Desktop app's full tool belt: `Belt` struct, `ReadDocument`, `WriteDocument`, `system()` prompt. |
| `home-core` | `lib.rs` | Module declarations (20 submodules). |
| `home-bridge` | `main.rs` | Axum API server bridging Tauri frontend to Rust core. |
| `home-metal` | `lib.rs` | Metal GPU kernels. |
| `home` (Python) | `cli.py` | CLI: ingest, search, ask, mem, doc, parse, flow, status. |
| `home` (Python) | `agent.py` | LangChain agent: `build_agent()`, `run()`. |
| `home` (Python) | `store.py` | Postgres + pgvector store, `HybridRetriever`. |
| `home` (Python) | `ingest.py` | Directory ingestion: PDF/DOCX/PPTX/XLSX/EPUB parsing, chunking. |
| `home` (Python) | `memory.py` | Persistent memory: add, search, update, forget, pinned. |
| `home` (Python) | `embed.py` | Local embedder: `model2vec` wrapper, `LocalEmbeddings`. |
| `home` (Python) | `flows.py` | LangGraph workflow loader and runner. |
| `home` (Python) | `docgen.py` | Markdown to HTML/DOCX conversion (zero third-party deps). |
| `home` (Python) | `config.py` | Configuration: workspace, Postgres, oMLX, embedder paths. |

---

## 11. Summary Statistics

| Metric | Count |
|:---|:---|
| **Distinct tool capabilities** | 42 |
| **Languages in plugin** | 42 |
| **Language checkers total** | ~120 unique checker commands |
| **Rust modules** | 20 |
| **Python modules** | 12 |
| **Skills (core)** | 8 |
| **Skills (plugin)** | ~55 (41 languages + 11 engineering/design) |
| **Plugins** | 3 |
| **Postgres tables** | 8+ (`chunks`, `sources`, `memories`, `rs_messages`, `rs_journal`, `rs_scratch`, `rs_tails`, `rs_crons`) |
| **Rust dependencies** | 15 |
| **Python dependencies** | 12 |
