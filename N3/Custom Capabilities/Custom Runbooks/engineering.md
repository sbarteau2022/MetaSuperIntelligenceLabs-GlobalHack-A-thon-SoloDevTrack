# engineering best practices
Understand before changing: reproduce, read, then edit. Smallest correct change; match surrounding style. Write the failing
test first when fixing a bug. Pure functions at the core, I/O at the edges. Handle errors at the boundary with context; never
swallow them. Validate all external input; least privilege; no secrets in code. Make operations idempotent and resumable.
Measure before optimizing (state hardware, warmup, variance, n). Review checklist: correctness, edge cases, concurrency,
security, failure modes, observability, tests, naming, docs. Build and stress-test in the microVM (vm_exec); ship only with
approval. Record decisions and why in the repo.

## The build loop (plugin skills; open one with skill_read, name like `design/design-spec`)
1. `design/design-spec`: write the contract (data, state, rules, edge table, examples) before code.
2. `design/preregister`: seal falsifiable claims before implementation; report against them.
3. `design/test-suites`: build the suite from the claims, boundaries first, then run the mutation check.
4. Implement against the red suite.
5. `engineering/adversarial-pressure`: a separate pass that tries to break the work and grades it honestly. Do not grade your own
   work. The harness loads this skill itself when a task asks to grade, audit, stress-test or break something, and will not accept
   "done" until you have actually run the attacks with run_python or vm_exec.
Also in `engineering`: code-review, debug, architecture, documentation, tech-debt, incident-response, deploy-checklist.
`skill_list` shows the installed plugins; `skill_list` with a plugin name lists its skills.

## Languages
The `languages` plugin holds best practices for Python, JavaScript, TypeScript, Rust, Go, Java, Kotlin, C#, C, C++, Swift, SQL, shell, Ruby and PHP. The harness loads a language's skill by itself when a task names it or you read or write its files, and when you write code in a language you must run at least one of its checkers (formatter, linter, type checker or tests; the commands are in each skill's first line) before you finish. Run them in the VM (`vm_exec`). If a checker is not installed, say so; never report a pass you did not see. `languages/index` lists the extensions.
