# MCP Connectors, Skills, Plugins & Tools — Full Inventory

Complete map of every tool surface in the Model Home harness: built-in tools, plugin skills, MCP servers, connectors, and the skill/plugin ecosystems.

---

## 1. Built-In Agentic Tools (home-core Rust)

The Rust core defines the `Tool` trait (`crates/home-core/src/tools.rs`) and assembles the full belt in `belt.rs`. Every tool is async, serde-JSON-serializable, and workspace-confined.

| Tool | Module | Description |
|:---|:---|:---|
| `read_file` | `tools.rs` | Read a UTF-8 text file from the workspace (path-confined via `confine()`). |
| `write_file` | `tools.rs` | Write (overwrite) a text file in the workspace; creates parent dirs. |
| `list_files` | `tools.rs` | List files under a workspace directory (recursive), path-confined. |
| `kb_search` | `tools.rs` | Search the Postgres knowledge base (hybrid semantic + keyword). Bridges to Python harness. |
| `run_shell` | `tools.rs` | Run an allowlisted program (`ls`, `cat`, `head`, `tail`, `wc`, `grep`, `python3`, `pytest`) in the sandbox. No shell metacharacters. 60 s timeout. |
| `run_python` | `tools.rs` | Run a short Python 3 program in the sandbox. One call instead of write + shell. 100 KB max, 60 s limit. |
| `read_document` | `belt.rs` | Read any supported document (PDF, DOCX, PPTX, XLSX, EPUB, HTML, text) as text. Invokes `home.cli parse`. |
| `write_document` | `belt.rs` | Create a document or artifact: Markdown, HTML, DOCX, or PDF (rendered via VM browser). Files go in `artifacts/`. |
| `web_search` | `extra.rs` | Search via SearXNG (`HOME_SEARCH_URL`). Returns top 8 results. |
| `web_fetch` | `extra.rs` | Hardened HTTP fetch: SSRF-checked, per-hop redirects (max 5), 2 MB size cap, HTML-to-text extraction. |
| `ingest` | `extra.rs` | Index a directory into a knowledge-base collection. Idempotent. Supports `.md`, `.txt`, `.py`, `.rs`, `.ts`, `.html`, `.pdf`, `.docx`, `.pptx`, `.xlsx`, `.epub`. |
| `vm_exec` | `extra.rs` | Run commands inside the Lima microVM (full toolchains, no workspace mount). |
| `clone_in` | `extra.rs` | Copy files into the workspace (approval-gated via `Approver` trait). |
| `ship_out` | `extra.rs` | Copy files out of the workspace (approval-gated via `Approver` trait). |
| `schedule` | `extra.rs` | Create a recurring cron job. Schedules live in Postgres (`rs_crons`). Polls every 30 s. |
| `journal` | `extra.rs` | Write a journal entry. Stored in Postgres (`rs_journal`). Author can be "model" or "user". |
| `journal_read` | `extra.rs` | Read journal entries. |
| `scratch` | `extra.rs` | Ephemeral scratchpad for transient notes. Stored in Postgres (`rs_scratch`). |
| `memory` | `extra.rs` | Persistent facts, preferences, episodes, session summaries. CRUD via `home mem add/search/update/forget/pinned`. Hybrid-searchable. |
| `think` | `think.rs` | Allocate a larger thinking budget before costly decisions. The model's next step gets a much larger budget (see §3). |
| `plugin_create` | `connectors.rs` | Create a new plugin (self-built tool). |
| `connector_write` | `connectors.rs` | Add a tested, approved Python connector (tool) to a plugin. |
| `browser_open` | `browser.rs` | Open a web page in the browser (Chromium + Playwright in the Lima VM). |
| `browser_snapshot` | `browser.rs` | Re-read the current page: URL, title, numbered interactive elements and text. |
| `browser_click` | `browser.rs` | Click an element by its ref. Irreversible clicks require user approval. |
| `browser_type` | `browser.rs` | Type text into an input by ref. Password fields are refused. |
| `browser_press` | `browser.rs` | Press a key (Enter, Escape, Tab, ArrowDown, PageDown, etc.). |
| `browser_scroll` | `browser.rs` | Scroll the page (up, down, top, bottom). |
| `browser_back` | `browser.rs` | Go back one page in history. |
| `browser_tabs` | `browser.rs` | Manage tabs: list, switch, close, or open new. |
| `browser_extract` | `browser.rs` | Return the text of part of the page by CSS selector. |

### 2. Cognitive Map Tools (cogmap_tools.rs)

| Tool | Description |
|:---|:---|
| `cogmap_brief` | Structured teaching brief for the model. Read-only: the model speaks from this brief. |
| `cogmap_map` | Inspectable cognitive map projection. Every non-unknown claim cites evidence and the policy version. |
| `cogmap_predict` | Commit a hash-sealed prediction BEFORE reveal. Post-reveal commits are rejected. |
| `cogmap_submit` | Submit an immutable finalized artifact. Deterministic grading runs only when activation is Active. |

### 3. Dynamic Skill/Workflow Tools (DocTools)

These are dynamically loaded from `skills/`, `workflows/`, and `plugins/` directories at belt construction time.

| Tool | Source Directory | Description |
|:---|:---|:---|
| `skill_read` | `skills/`, `plugins/*/skills/` | Read a skill file from an enabled plugin or the `skills/` directory. |
| `skill_write` | `skills/`, `plugins/*/skills/` | Write a skill file (editable persona, custom skills). |
| `workflow_read` | `<workspace>/flows/` | Read a workflow definition. |
| `workflow_write` | `<workspace>/flows/` | Write a workflow definition (compiled LangGraph). |
| `workflow_list` | `<workspace>/flows/` | List all LangGraph workflows. |
| `skill_list` | — | List installed plugins and their skills. |

---

## 4. Python Harness Tools (home/ CLI)

Exposed via `uv run home <cmd>` and callable from Rust tools that spawn the Python process.

| Command | Module | Description |
|:---|:---|:---|
| `home ingest <dir> <coll>` | `ingest.py` | Idempotent directory ingestion into a knowledge-base collection. |
| `home search "<q>"` | `store.py` | Hybrid KB search (dense cosine + full-text, RRF-fused). |
| `home ask "<task>"` | `agent.py` | Run the agent on a task, resumed from Postgres checkpoint. |
| `home status` | `store.py` | Collection stats (files, chunks per collection). |
| `home mem add` | `memory.py` | Add a memory entry (kind: fact, preference, episode, session). |
| `home mem search` | `memory.py` | Search memories (hybrid semantic + keyword). |
| `home mem update` | `memory.py` | Update a memory entry. |
| `home mem forget` | `memory.py` | Delete a memory entry. |
| `home mem pinned` | `memory.py` | List pinned memories. |
| `home doc html/docx` | `docgen.py` | Convert markdown → styled HTML or `.docx` (zero third-party deps). |
| `home parse <path>` | `ingest.py` | Parse a document (PDF, DOCX, PPTX, XLSX, EPUB, HTML, text) into readable text. |
| `home flow list` | `flows.py` | List workflows in `<workspace>/flows/`. |
| `home flow run <name>` | `flows.py` | Stream a workflow execution. |

---

## 5. Plugin System

Three built-in plugins ship with the harness. Each has a `plugin.json` with metadata, skills, and enforcement rules.

### 5.1 `languages` Plugin

**File:** `plugins/languages/plugin.json`

42 languages, each with extensions, trigger words, skill reference, checker commands, version info, and tool bindings.

| # | Language | Extensions | Skill | Checkers | Version Source |
|:---|:---|:---|:---|:---|:---|
| 1 | Python | `.py`, `.pyi` | `python` | ruff, mypy, pyright, pytest, flake8, pylint, unittest | this machine |
| 2 | JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx` | `javascript` | eslint, prettier, vitest, jest, mocha, node --test, biome | this machine and web |
| 3 | TypeScript | `.ts`, `.tsx`, `.mts`, `.cts` | `typescript` | tsc, eslint, prettier, vitest, jest, biome | web |
| 4 | Rust | `.rs` | `rust` | cargo clippy, cargo test, cargo check, cargo fmt, rustfmt, cargo nextest | this machine |
| 5 | Go | `.go` | `go` | go vet, go test, gofmt, staticcheck, golangci-lint | web |
| 6 | Java | `.java` | `java` | mvn, mvnw, gradle, gradlew, javac, checkstyle, spotbugs | knowledge |
| 7 | Kotlin | `.kt`, `.kts` | `kotlin` | gradle, gradlew, ktlint, detekt, kotlinc | web |
| 8 | C# | `.cs` | `csharp` | dotnet build, dotnet test, dotnet format | web |
| 9 | C | `.c`, `.h` | `c` | gcc, clang, cmake, make, cppcheck, clang-tidy, valgrind | this machine |
| 10 | C++ | `.cc`, `.cpp`, `.cxx`, `.hpp`, `.hh`, `.hxx` | `cpp` | g++, clang++, cmake, make, ctest, clang-tidy, cppcheck | this machine |
| 11 | Swift | `.swift` | `swift` | swift build, swift test, swiftlint, swift-format, xcodebuild | this machine |
| 12 | SQL | `.sql` | `sql` | sqlfluff, psql, sqlite3, pg_prove | this machine |
| 13 | Shell | `.sh`, `.bash`, `.zsh` | `shell` | shellcheck, shfmt, bash -n, sh -n, bats | this machine |
| 14 | Ruby | `.rb` | `ruby` | rubocop, rspec, rake, ruby -c, ruby -wc, brakeman | this machine |
| 15 | PHP | `.php` | `php` | phpstan, psalm, phpunit, php -l, phpcs, php-cs-fixer, pest | web |
| 16 | Dart | `.dart` | `dart` | dart format, dart analyze, dart test, flutter analyze, flutter test | web |
| 17 | Objective-C | `.m`, `.mm` | `objc` | clang, xcodebuild, clang-format, xctest | this machine |
| 18 | R | `.r`, `.rmd` | `r` | lintr, styler, testthat, devtools::test, r cmd check | web |
| 19 | Julia | `.jl` | `julia` | pkg.test, juliaformatter, aqua, jet | web |
| 20 | MATLAB | (none) | `matlab` | runtests, checkcode, mlint, matlab -batch, octave | host-bound |
| 21 | Octave | `.m` | `matlab` | octave | VM |
| 22 | Fortran | `.f90`, `.f95`, `.f03`, `.f08`, `.f`, `.for` | `fortran` | gfortran, fpm test, fprettify, flang | web |
| 23 | Lean | `.lean` | `lean` | lake build, lake test, lean | web |
| 24 | Scala | `.scala`, `.sc` | `scala` | sbt, scala-cli, scalafmt, scalafix, mill | web |
| 25 | Elixir | `.ex`, `.exs` | `elixir` | mix format, mix credo, mix dialyzer, mix test, mix compile | web |
| 26 | Erlang | `.erl`, `.hrl` | `erlang` | rebar3, erlc, dialyzer, escript | web |
| 27 | Perl | `.pl`, `.pm` | `perl` | perlcritic, perltidy, prove, perl -c, perl -wc | not verified |
| 28 | Groovy | `.groovy`, `.gvy` | `groovy` | codenarc, gradlew, gradle, npm-groovy-lint | not verified |
| 29 | Haskell | `.hs`, `.lhs` | `haskell` | cabal build, cabal test, stack test, hlint, ormolu, fourmolu, ghc -wall | web |
| 30 | OCaml | `.ml`, `.mli` | `ocaml` | dune build, dune test, dune runtest, ocamlformat | web |
| 31 | F# | `.fs`, `.fsx`, `.fsi` | `fsharp` | dotnet build, dotnet test, fantomas, dotnet fsi | knowledge |
| 32 | Clojure | `.clj`, `.cljs`, `.cljc`, `.edn` | `clojure` | clj-kondo, cljfmt, lein test, clojure -x, clojure -m, kaocha | web |
| 33 | Zig | `.zig` | `zig` | zig fmt, zig build, zig test | web |
| 34 | Assembly | `.asm`, `.s` | `assembly` | nasm, yasm, clang -c, gcc -c, llvm-mc, objdump, otool | this machine |
| 35 | Lua | `.lua` | `lua` | luacheck, stylua, busted, luau-analyze | not verified |
| 36 | PowerShell | `.ps1`, `.psm1`, `.psd1` | `powershell` | invoke-scriptanalyzer, invoke-pester | web |
| 37 | Terraform | `.tf`, `.tfvars`, `.hcl` | `terraform` | terraform fmt, terraform validate, tflint, tofu validate, tofu fmt, checkov, trivy config | web |
| 38 | GLSL | `.glsl`, `.vert`, `.frag`, `.geom`, `.comp`, `.tesc`, `.tese` | `glsl` | glslangvalidator, glslc, spirv-val | not verified |
| 39 | HLSL | `.hlsl`, `.hlsli`, `.fx` | `hlsl` | dxc, fxc | not verified |
| 40 | Metal | `.metal` | `metal` | xcrun metal, metallib, metal -std | host-bound |
| 41 | Solidity | `.sol` | `solidity` | forge test, forge build, forge fmt, slither, solhint, solc, hardhat test | web |
| 42 | COBOL | `.cob`, `.cbl`, `.cobol`, `.cpy` | `cobol` | cobc, cobol-check | web |

**Version Verification Status:**

| Source | Count | Languages |
|:---|:---:|:---|
| this machine | 10 | Python, JS, Rust, C, C++, Swift, SQL, Shell, Ruby, ObjC, Assembly |
| web (verified Sep 2026) | 17 | TypeScript, Go, Kotlin, C#, PHP, Dart, R, Julia, Fortran, Lean, Scala, Elixir, Erlang, Haskell, OCaml, PowerShell, Terraform, Solidity, COBOL |
| knowledge (unverified) | 4 | Java, F#, GLSL, HLSL |
| not verified online | 5 | Perl, Groovy, Lua, MATLAB, Metal |
| host-bound | 2 | MATLAB (licensed), Metal (Xcode only) |
| VM-installed | 1 | Octave |

### 5.2 `engineering` Plugin

**File:** `plugins/engineering/plugin.json`

| Skill | File | Description |
|:---|:---|:---|
| `adversarial-pressure` | `adversarial-pressure.md` | Try to break it, then grade it honestly. Enforced when grading/auditing/stress-testing. |
| `code-review` | `code-review.md` | Review a change for what will actually break. |
| `debug` | `debug.md` | Find the cause, not a patch that hides it. |
| `architecture` | `architecture.md` | Decide the structure, and write down why. |
| `documentation` | `documentation.md` | Write what a stranger needs to use and change it. |
| `tech-debt` | `tech-debt.md` | Find it, price it, pay the right part. |
| `incident-response` | `incident-response.md` | Stop the damage, find the cause, learn once. |
| `deploy-checklist` | `deploy-checklist.md` | Ship without surprises. |

**Enforced Rules:**

| Skill | When (trigger phrases) | Required Tools |
|:---|:---|:---|
| `adversarial-pressure` | grade, adversarial, pressure test, stress test, break it, try to break, red team, find bugs, audit | `run_python`, `vm_exec` |

### 5.3 `design` Plugin

**File:** `plugins/design/plugin.json`

| Skill | File | Description |
|:---|:---|:---|
| `design-spec` | `design-spec.md` | Write the contract before the code. |
| `preregister` | `preregister.md` | Say what would prove you wrong, before you build. |
| `test-suites` | `test-suites.md` | Build a suite that can actually fail. |
| `plugin-builder` | `plugin-builder.md` | Build your own skills, plugins and tools, with rigour. |

**Enforced Rules:**

| Skill | When (trigger phrases) | Required Tools |
|:---|:---|:---|
| `plugin-builder` | build a skill, create a skill, make a skill, new skill, build a plugin, create a plugin, make a plugin, new plugin, build a connector, create a connector, make a connector, new connector, build a tool, create a tool, make a tool, new tool, mcp | `plugin_create`, `connector_write`, `skill_write` |

---

## 6. Core Skills (in `skills/`)

These are plain markdown files in the `skills/` directory, loaded dynamically by `DocTools`.

| Skill | Purpose | Editable |
|:---|:---|:---|
| `persona.md` | Editable agent personality. Starts nearly blank. | Yes (`skill_write`) |
| `engineering.md` | Engineering best practices, adversarial pressure pass, build loop. | Yes |
| `web-design.md` | Semantic HTML, WCAG 2.2 AA, responsive layout, performance budgets. | Yes |
| `landing-page-checklist.md` | 5-item checklist for shipping a landing page (derived from web-design). | Yes |
| `voice-dickens.md` | Dickens register: comic abundance, periodic sentences, parallel lists, irony. | Yes |
| `voice-lewis.md` | C.S. Lewis register: conversational authority, homely analogy, dry wit. | Yes |
| `voice-tolstoy.md` | Tolstoy register: omniscient, morally attentive, concrete detail, long balanced sentences. | Yes |
| `voice-technical.md` | Technical documentation: define terms, measurable claims, numbered steps, failure modes. | Yes |

---

## 7. Model Context Protocol (MCP) Surface

### 7.1 oMLX Server MCP Configuration

The oMLX model server (local, at `127.0.0.1:8000/v1`) has MCP built-in:

| Setting | Value | Description |
|:---|:---|:---|
| `mcp.config_path` | `null` | No external MCP config file loaded. |
| `mcp.expose_tools` | `true` | MCP tools are exposed to clients. |

The server's `model_settings.json` shows `mcp.expose_tools: true` — MCP tools are available from the model server but no external MCP servers are configured.

### 7.2 Browser Server (MCP-like RPC)

The Lima VM browser server (`browserd.py`) exposes an authenticated JSON-RPC on `127.0.0.1:9333`:

| RPC Action | Description |
|:---|:---|
| `open` | Open a URL in the browser. |
| `snapshot` | Read the current page (URL, title, interactive elements, text). |
| `shot` | Take a screenshot (JPEG, base64-encoded). |
| `click` | Click an element by ref. |
| `type` | Type text into an input by ref. |
| `press` | Press a key. |
| `scroll` | Scroll the page. |
| `back` | Go back in history. |
| `tabs` | Manage tabs (list, switch, close, new). |
| `extract` | Return text by CSS selector. |
| `describe` | Describe an element (tag, label, action, href). |
| `pdf` | Render HTML to PDF (A4 format). |
| `mouse` | Simulate mouse click (for user driving the browser). |
| `wheel` | Simulate mouse wheel (for user driving the browser). |
| `keys` | Type text (for user driving the browser). |
| `key` | Press a key (for user driving the browser). |
| `status` | Return browser status (up, url, title, tab count). |

### 7.3 Skills-Installed MCP Servers (External)

The following installed skills expose MCP tools:

| Skill | MCP Tools | Description |
|:---|:---|:---|
| `agents-sdk` | `McpAgent` (build MCP servers), MCP client API | Cloudflare Workers Agents SDK — build MCP servers, stateful agents, durable workflows. |
| `bailian-cli` | `bl mcp list`, `bl mcp tools`, `bl mcp call` | Aliyun Bailian CLI — MCP marketplace for models and tools. |
| `bailian-web-search` | MCP routing | Routes web search to MCP when Token Plan search fails. |

---

## 8. Connector System

### 8.1 `plugin_create` Tool

Creates a new plugin directory with `plugin.json`. The model can build its own plugins.

| Field | Description |
|:---|:---|
| Authorship | Marked as model-authored in the plugin metadata. |
| Enforce | Auto-cleaned of stale enforcement rules. |
| Git | Recorded as a local git commit. |

### 8.2 `connector_write` Tool

Adds a tested, approved Python connector (tool) to a plugin.

| Step | Description |
|:---|:---|
| 1. Write code | A pure function of `ARGS` (dict of arguments). Prints the result. On bad input, `raise SystemExit(1)`. |
| 2. Call `connector_write` | Runs the code's examples; if any fail, nothing is registered. |
| 3. Approval | If examples pass, the user is asked to approve the code. |
| 4. Registration | After approval, the tool is registered as `<plugin>__<name>`. |
| 5. Void | Editing the code later voids the approval. |

### 8.3 Connector Schema

| Field | Type | Description |
|:---|:---|:---|
| `fq` | string | Fully qualified name (`plugin__toolname`). |
| `description` | string | Tool description (shown in tool list). |
| `properties` | object | JSON Schema for arguments. |
| `required` | array | Required argument names. |
| `code` | string | Python source (SHA-256 hashed for integrity). |
| `active` | bool | Whether the connector is enabled. |

---

## 9. Language Verification Toolchain

### 9.1 `versions.py` — Single Source of Truth

**File:** `scripts/langs/versions.py`

Maintains the canonical table of language pins, verified versions, and check dates. Two commands:

| Command | Description |
|:---|:---|
| `versions.py apply` | Write each skill's "## Versions" section and `plugin.json`'s per-language "version" metadata from the table. |
| `versions.py check` | Compare the pins with what is installed on this machine; exit 1 if anything needs a look. |

### 9.2 `langeval.py` — Language Evaluation

**File:** `scripts/langeval/langeval.py`

Evaluates language skills by running code blocks in a sandboxed environment and judging correctness.

| Function | Description |
|:---|:---|
| `tool_ok` | Check if a tool is available. |
| `ask` | Interactive question prompt. |
| `code_block` | Extract and run code blocks. |
| `sandboxed` | Run code in a sandbox. |
| `judge_prog` | Judge program correctness. |
| `build_and_run` | Build and run a program. |
| `tasks_for` | Generate evaluation tasks for a language. |
| `run` | Run the full evaluation. |
| `report` | Generate the evaluation report. |

---

## 10. Infrastructure Services

### 10.1 oMLX Model Server

| Setting | Value |
|:---|:---|
| Launchd job | `com.arena.omlx` |
| Binary | `/opt/homebrew/bin/omlx serve` |
| Model dir | `~/models-mtp-test` |
| Host:Port | `127.0.0.1:8000` |
| Model | `Qwen3.8-27B-OptiQ-4bit-mtp` |
| Max context | 131,072 tokens |
| Max tokens | 16,384 |
| Thinking budget | 256 (default, adjustable) |
| GPU offload | ANE (Apple Neural Engine) enabled |
| SSD cache | 128 GB max |
| Memory guard | 41 GB custom ceiling |

### 10.2 Lima MicroVM

| Setting | Value |
|:---|:---|
| Instance name | `home` |
| Config | `vm/home.yaml` |
| VM Type | vz (Apple Virtualization.framework) |
| Architecture | aarch64 |
| CPUs | 6 |
| Memory | 16 GiB |
| Disk | 80 GiB |
| Image | Ubuntu 24.04 server cloud image (ARM) |
| Mounts | None (nothing from the Mac is visible) |
| Browser | Chromium + Playwright at `~/browser/` |
| Provision | `vm/provision-language-toolchains.sh` |

### 10.3 Postgres

| Setting | Value |
|:---|:---|
| Version | 17 + pgvector extension |
| Port | 5433 |
| DSN | `postgresql://home@127.0.0.1:5433/home` |
| Data dir | `~/Library/Application Support/ModelHome/pg` |
| Launchd job | `com.home.postgres` |

### 10.4 Browser Server (VM)

| Setting | Value |
|:---|:---|
| Script | `vm/browser/browserd.py` |
| Setup | `vm/browser/setup.sh` |
| Port | 9333 |
| Framework | Playwright (Chromium) + aiohttp |
| Auth | Bearer token at `~/Library/Application Support/engineering.atoma.modelhome/browser.token` |
| Viewport | 1280 × 800 |
| Network policy | HTTP(S) only; private/local addresses blocked |

---

## 11. Cognitive Map Modules

Five Rust crates in `cognitive-map/` (excluded from the main Cargo workspace):

| Crate | Path | Description |
|:---|:---|:---|
| `cogmap-slice1` | `slice1/` | Session state reducer (event-sourced state machine). |
| `cogmap-slice2` | `slice2/` | Concept mapping and dimension analysis. |
| `cogmap-slice3` | `slice3/` | Curriculum, goal, and task template management. |
| `cogmap-slice4` | `slice4/` | Opt-in/kill-switch activation machine. |
| `cogmap-integration` | `integration/` | SQL mapping, activation persistence. |

**Manifest:** `cognitive-map/manifest.json` — 100 modules across courses, each with concepts, dimensions, rubric criteria, and prerequisites.

---

## 12. Gap Analysis

### 12.1 Missing Infrastructure

| Gap | Impact | Priority |
|:---|:---|:---|
| **Browser server not deployed** | `browserd.py` exists in repo but has not been deployed to the Lima VM. Without it, all browser tools fail. | P0 |
| **No container sandbox backend** | `Backend::Container` in `sandbox.rs` is a stub. `run_shell` and `run_python` are limited to the seatbelt sandbox only. | P1 |
| **No tests** | `tests/` directory is empty. No integration tests for the agent loop, tools, or Postgres checkpointing. | P1 |
| **No CI/CD** | No GitHub Actions, no automated build pipeline for Rust, Python, or Tauri. | P2 |
| **No wander mode scheduler** | `wander()` exists in `standard.rs` but no cron entry or auto-start. | P3 |

### 12.2 Missing Language Checkers

| Language | Missing Checker | Reason |
|:---|:---|:---|
| Dart | `dart`, `flutter` | Requires separate distribution. |
| PowerShell | `pwsh` | Microsoft package is x86_64-only on Ubuntu arm64. |
| Swift | `swift`, `xcodebuild` | Requires macOS host (not Linux VM). |
| Zig | `zig` | Requires separate distribution. |
| Terraform | `terraform` | Requires separate distribution. |
| Metal | `metal`, `metallib` | Requires macOS host Xcode. |
| MATLAB | `matlab` | Requires licensed MathWorks installation. |

### 12.3 Missing Version Verification

| Language | Status | Action |
|:---|:---|:---|
| Java | "not verified online; from knowledge" | Verify `java -version` on a fresh install. |
| F# | "from knowledge, with .NET 10 confirmed" | Verify `dotnet fsi --version`. |
| GLSL | "not verified online; from knowledge" | Verify `glslangValidator --version`. |
| HLSL | "not verified online; from knowledge" | Verify `dxc --version`. |
| Perl | "not verified online" | Verify `perl -e 'print $]'`. |
| Groovy | "not verified online" | Verify `groovy --version`. |
| Lua | "not verified online" | Verify `lua -v`. |

### 12.4 Missing Tooling

| Gap | Description | Priority |
|:---|:---|:---|
| **No `run_shell` for VM** | `run_shell` only works in the seatbelt sandbox on the Mac. The VM has `vm_exec` but no equivalent allowlisted command runner. | Medium |
| **No language-specific runner** | Each language uses `vm_exec` (generic command runner). No `run_<lang>` tool that auto-invokes the correct compiler/interpreter. | Low |
| **No mutation testing** | The engineering skill mentions "mutation check" but no mutation testing tool is configured. | Medium |
| **No `docs/` content** | Only `vm-language-toolchains.md` and `langeval/` exist. No API docs, no architecture docs. | Medium |
| **No search backend** | `web_search` requires `HOME_SEARCH_URL` set to a SearXNG instance. Without it, search falls back to a message. | Medium |

---

## 13. Install Plan — GitHub-Sourced Skills

### 13.1 Priority Skills to Install

| Priority | Skill | Source | Purpose |
|:---|:---|:---|:---|
| P0 | `cloudflare` | `npx skills add cloudflare/skills/cloudflare` | Workers, Pages, KV, D1, R2, AI, Tunnel, WAF. |
| P0 | `wrangler` | `npx skills add cloudflare/skills/wrangler` | Cloudflare Workers CLI for deploy/dev. |
| P0 | `workers-best-practices` | `npx skills add cloudflare/skills/workers-best-practices` | Writing Workers code against production best practices. |
| P1 | `durable-objects` | `npx skills add cloudflare/skills/durable-objects` | Stateful coordination primitives. |
| P1 | `cloudflare-email-service` | `npx skills add cloudflare/skills/cloudflare-email-service` | Transactional email sending + routing. |
| P1 | `turnstile-spin` | `npx skills add cloudflare/skills/turnstile-spin` | CAPTCHA replacement for bot verification. |
| P1 | `modern-web-guidance` | `npx skills add cloudflare/skills/modern-web-guidance` | HTML/CSS/JS best practices (MANDATORY for frontend work). |
| P1 | `web-perf` | `npx skills add cloudflare/skills/web-perf` | Chrome DevTools performance auditing. |
| P2 | `agents-sdk` | `npx skills add cloudflare/skills/agents-sdk` | Stateful agents, MCP client/server, durable workflows. |
| P2 | `sandbox-stable` | `npx skills add cloudflare/skills/sandbox-stable` | Cloudflare Sandbox apps (commands, sessions, files, ports, tunnels). |
| P2 | `sandbox-next` | `npx skills add cloudflare/skills/sandbox-next` | Cloudflare Sandbox @next (Sandbox SDK 1.0 preview). |
| P2 | `sandbox-migrate-to-next` | `npx skills add cloudflare/skills/sandbox-migrate-to-next` | Porting stable Sandbox to 1.0. |
| P2 | `cloudflare-one` | `npx skills add cloudflare/skills/cloudflare-one` | Zero Trust and SASE (Access, Gateway, WARP, Tunnel). |
| P2 | `cloudflare-one-migrations` | `npx skills add cloudflare/skills/cloudflare-one-migrations` | Zscaler/Palo Alto migration planning. |
| P3 | `chrome-extensions` | `npx skills add cloudflare/skills/chrome-extensions` | Manifest V3 extension development + publishing. |

### 13.2 Community Skills (Already Installed)

The following skills were installed by a previous agent session and exist in `~/.agents/skills/`:

| Category | Skills |
|:---|:---|
| **Cloudflare** | `cloudflare`, `wrangler`, `workers-best-practices`, `durable-objects`, `cloudflare-email-service`, `cloudflare-one`, `cloudflare-one-migrations`, `turnstile-spin`, `modern-web-guidance`, `web-perf`, `agents-sdk`, `sandbox-stable`, `sandbox-next`, `sandbox-migrate-to-next` |
| **Bailian (Aliyun)** | `bailian-cli`, `bailian-docs-llm-wiki`, `bailian-finetune`, `bailian-gen`, `bailian-managed-agent`, `bailian-model-recommend`, `bailian-protocol`, `bailian-train-deploy`, `bailian-web-search` |
| **Other** | `chrome-extensions`, `find-skills`, `goal`, `create-skill` |

### 13.3 Skills to Bulk-Install from GitHub

| Source Repo | Skills | Install Command |
|:---|:---|:---|
| `vercel-labs/agent-skills` | 135 community skills (artifacts-builder, brainstorming, brand-extract, figma-*, gsap-*, fal-*, venice-*, etc.) | `npx skills add vercel-labs/agent-skills` |
| `anthropics/skills` | Anthropic-curated skills | `npx skills add anthropics/skills` |
| `github/agent-skills` | GitHub-curated skills | `npx skills add github/agent-skills` |

---

## 14. Module Map

### 14.1 Rust Modules (home-core)

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
| `cron` | `cron.rs` | Durable cron: schedules in Postgres, 30 s poll, unattended runner. |
| `store` | `store.rs` | Postgres thread checkpoints: append, load, replace_head, threads list. |
| `api` | `api.rs` | UI backend: journal, scratch, session close. Pure functions over Postgres. |
| `session` | `session.rs` | Session consolidation: summary → memory, verbatim tail, preamble seeding. |
| `nudge` | `nudge.rs` | User messages during agent mid-turn: queue, drain, frame. |
| `standard` | `standard.rs` | Standard wiring: work agent, unattended runner, wander mode. |
| `cogmap_tools` | `cogmap_tools.rs` | Cognitive map tools: brief, map, predict, submit. Event-sourced state. |
| `vmlog` | `vmlog.rs` | Live VM feed: line publishing, history, code action tracking. |
| `belt` | `belt.rs` | Desktop app's full tool belt: `Belt` struct, `ReadDocument`, `WriteDocument`, `system()` prompt. |
| `home-core` | `lib.rs` | Module declarations (20 submodules). |

### 14.2 Python Modules

| Module | File | Responsibility |
|:---|:---|:---|
| `cli` | `cli.py` | CLI: ingest, search, ask, mem, doc, parse, flow, status. |
| `agent` | `agent.py` | LangChain agent: `build_agent()`, `run()`. |
| `store` | `store.py` | Postgres + pgvector store, `HybridRetriever`. |
| `ingest` | `ingest.py` | Directory ingestion: PDF/DOCX/PPTX/XLSX/EPUB parsing, chunking. |
| `memory` | `memory.py` | Persistent memory: add, search, update, forget, pinned. |
| `embed` | `embed.py` | Local embedder: `model2vec` wrapper, `LocalEmbeddings`. |
| `flows` | `flows.py` | LangGraph workflow loader and runner. |
| `docgen` | `docgen.py` | Markdown to HTML/DOCX conversion (zero third-party deps). |
| `config` | `config.py` | Configuration: workspace, Postgres, oMLX, embedder paths. |

### 14.3 Other Modules

| Module | File | Responsibility |
|:---|:---|:---|
| `home-bridge` | `src-tauri/src/main.rs` | Axum API server bridging Tauri frontend to Rust core. |
| `home-metal` | `crates/home-metal/lib.rs` | Metal GPU kernels. |
| `browserd` | `vm/browser/browserd.py` | Browser server (Chromium + Playwright) in the Lima VM. |
| `versions` | `scripts/langs/versions.py` | Single source of truth for language version pins. |
| `langeval` | `scripts/langeval/langeval.py` | Language skill evaluation framework. |

---

## 15. Summary Statistics

| Metric | Count |
|:---|:---|
| **Built-in agentic tools** | 32 |
| **Cognitive map tools** | 4 |
| **Dynamic skill/workflow tools** | 6 |
| **Python CLI commands** | 13 |
| **Browser RPC actions** | 16 |
| **Languages in plugin** | 42 |
| **Language checkers total** | ~120 unique checker commands |
| **Rust modules** | 20 |
| **Python modules** | 9 |
| **Core skills** | 8 |
| **Plugin skills** | 55 (41 languages + 11 engineering/design) |
| **Built-in plugins** | 3 |
| **Installed skills (external)** | 26 core + 135 community |
| **MCP-exposing skills** | 3 (agents-sdk, bailian-cli, bailian-web-search) |
| **Connector tools** | 2 (plugin_create, connector_write) |
| **Postgres tables** | 8+ |
| **Cognitive map modules** | 100 |
| **Infrastructure services** | 4 (oMLX, Lima VM, Postgres, Browser) |

---

## 16. Build List

### P0 — Blockers

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 1 | **Browser server deployed to VM** | infra | `browserd.py` and `setup.sh` must be deployed to `~/browser/` in the Lima VM. |
| 2 | **Postgres running** | infra | `com.home.postgres` must be installed and running on port 5433. |
| 3 | **oMLX server running** | infra | OpenAI-compatible API at `127.0.0.1:8000/v1` must be reachable. |
| 4 | **Embedder model cached** | infra | `potion-code-16M` model2vec directory at `~/.cache/huggingface/hub/`. |
| 5 | **Search backend configured** | infra | Set `HOME_SEARCH_URL` to a SearXNG instance for `web_search` to work. |

### P1 — High Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 6 | **Container sandbox backend** | infra | Implement `Backend::Container` in `sandbox.rs`. |
| 7 | **VM provision script validation** | infra | Run the provision script on a fresh VM image and verify all 41 language checkers. |
| 8 | **Version verification sweep** | dev | Verify the 7 languages marked "not verified" or "from knowledge". |
| 9 | **Integration test suite** | dev | `tests/` is empty. Add tests for the full agent loop. |
| 10 | **Documentation** | docs | Add architecture overview, API reference, and deployment guide to `docs/`. |

### P2 — Medium Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 11 | **CI/CD pipeline** | infra | GitHub Actions for Rust build + test, Python lint, Tauri build. |
| 12 | **Mutation testing** | dev | Add a mutation testing tool and wire it into the engineering skill. |
| 13 | **Wander mode auto-start** | infra | Add a cron entry or launchd job for wander mode. |
| 14 | **Language-specific runners** | dev | Add `run_python`, `run_rust`, `run_golang` etc. |
| 15 | **Schema migration tests** | dev | Tests for backward compatibility of `rs_journal` and `rs_messages`. |

### P3 — Low Priority

| # | Item | Owner | Notes |
|:---|:---|:---|:---|
| 16 | **Dart/PowerShell/Zig toolchains** | infra | Add to the VM provision script once arm64 packages are available. |
| 17 | **Metal/MATLAB fallback** | infra | Surface as "unavailable" rather than claiming installed. |
| 18 | **Performance benchmarks** | dev | Measure thinking budget cache hit rates, embedder throughput, RAG latency. |
| 19 | **Telemetry / usage stats** | infra | Track token usage, tool call frequency, agent step counts. |
---

## Audit certification (Coder A — Agent Harness partition, 2026-10-30)

- Date: 2026-10-30
- Status: PASS-WITH-NOTES (compliant copy; original body above preserved byte-for-byte, nothing rewritten)
- Scope: skills/MCP_SKILLS_PLUGINS.md, 616 lines (REPORTED: wc -l 2026-10-30; VERIFIED: read from /Users/stewartbarteau/Desktop/skills/MCP_SKILLS_PLUGINS.md)
- Placement: ~/Desktop/N3/skills/MCP_SKILLS_PLUGINS.md (REPORTED: contract maps this partition to N3/skills/; no pre-existing N3 copy — ls confirmed absent 2026-10-30 — so the compliant copy takes the canonical name)
- Claim-label convention for this copy: every factual claim in the preserved body above is REPORTED (coder audit observation, not independently re-verified) unless the body itself marks it otherwise. Nothing in the body is asserted as VERIFIED by this audit except where this appendix says VERIFIED with evidence.

Per-gate verdicts (contract audit gate, 8 points):
1. Date/Status: body has none (VERIFIED by read 2026-10-30); supplied by this appendix. NOTE.
2. Claim labels: body has 16 VERIFIED/REPORTED/INFERRED/PROPOSED keyword hits (REPORTED: grep -c 2026-10-30) but tool/plugin/MCP/infra inventory claims are unlabeled; blanket-REPORTED convention above applies. NOTE.
3. Perf numbers: no tok/s or latency claims in body (VERIFIED by read); infra tables (section 10: 131072-token context, 128 GB SSD cache, 41 GB memory guard) state settings, not measured performance. N/A.
4. Corrections section: absent in body (VERIFIED); added below; future changes append dated entries only.
5. Paths/versions: exact module paths, RPC actions, plugin files, and version sources present in sections 1-11 (VERIFIED by read); NOT-CHECKED statement below.
6. Secrets: grep 2026-10-30 for sk-/ghp_/private-key patterns: no hits (VERIFIED, exit 1).
7. Bulk actions: body contains no bulk-delete/rewrite instructions (VERIFIED); section 13.3 "Bulk-Install" lists per-repo install commands with per-item disposition (INFERRED: title only, rows are per-item); the section 16 build list is per-item.
8. Web-development scope: N/A (INFERRED: harness inventory doc, no frontend code); placement per contract mapping (REPORTED).

## Corrections

- 2026-10-30: Audit appendix added (Date/Status/claim-label convention/per-gate verdicts/Corrections/NOT-CHECKED). Zero body bytes altered. (REPORTED)

## NOT CHECKED

- Live infra values (oMLX settings, Lima VM spec, Postgres DSN/port, browser token path, SearXNG URL) not re-verified on this machine.
- Language version sources (section 5.1 table) and "not verified / from knowledge" rows taken as REPORTED from the body; no fresh version checks run.
- External references (plugin.json files, scripts/langs/versions.py, scripts/langeval, skills in ~/.agents/skills) not resolved.
- GitHub-sourced install plan (section 13) not executed or verified; commands carried as REPORTED.
