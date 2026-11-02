# index: how the languages plugin works, and the extension map
When you read or write a file of a language below, the harness loads that language's skill into the conversation (once). When you write code files of a language, you may not finish until you have run at least one of its checkers, and said what passed. If a checker is not installed, say so; do not claim a pass you did not see.
Run checkers in the VM with `vm_exec` (build and test there), or with `run_shell` if that is what you have. Check commands per language are in each skill's first line.
| language | files | skill |
|---|---|---|
| Python | .py .pyi | languages/python |
| JavaScript | .js .mjs .cjs .jsx | languages/javascript |
| TypeScript | .ts .tsx .mts .cts | languages/typescript |
| Rust | .rs | languages/rust |
| Go | .go | languages/go |
| Java | .java | languages/java |
| Kotlin | .kt .kts | languages/kotlin |
| C# | .cs | languages/csharp |
| C | .c .h | languages/c |
| C++ | .cc .cpp .cxx .hpp .hh .hxx | languages/cpp |
| Swift | .swift | languages/swift |
| SQL | .sql | languages/sql |
| Shell | .sh .bash .zsh | languages/shell |
| Ruby | .rb | languages/ruby |
| PHP | .php | languages/php |
| Dart | .dart | languages/dart |
| Objective-C | .m .mm | languages/objc |
| R | .r .rmd | languages/r |
| Julia | .jl | languages/julia |
| MATLAB / Octave | (by name only; .m belongs to Objective-C) | languages/matlab |
| Fortran | .f90 .f95 .f03 .f08 .f .for | languages/fortran |
| Lean | .lean | languages/lean |
| Scala | .scala .sc | languages/scala |
| Elixir | .ex .exs | languages/elixir |
| Erlang | .erl .hrl | languages/erlang |
| Perl | .pl .pm | languages/perl |
| Groovy | .groovy .gvy | languages/groovy |
| Haskell | .hs .lhs | languages/haskell |
| OCaml | .ml .mli | languages/ocaml |
| F# | .fs .fsx .fsi | languages/fsharp |
| Clojure | .clj .cljs .cljc .edn | languages/clojure |
| Zig | .zig | languages/zig |
| Assembly | .asm .s | languages/assembly |
| Lua | .lua | languages/lua |
| PowerShell | .ps1 .psm1 .psd1 | languages/powershell |
| Terraform / HCL | .tf .tfvars .hcl | languages/terraform |
| GLSL | .glsl .vert .frag .geom .comp | languages/glsl |
| HLSL | .hlsl .hlsli .fx | languages/hlsl |
| Metal | .metal | languages/metal |
| Solidity | .sol | languages/solidity |
| COBOL | .cob .cbl .cobol .cpy | languages/cobol |
For anything not listed, use `engineering/code-review` and the target language's own formatter, linter and test runner. Cross-language process lives in the `design` and `engineering` plugins (spec, tests, adversarial pass).
