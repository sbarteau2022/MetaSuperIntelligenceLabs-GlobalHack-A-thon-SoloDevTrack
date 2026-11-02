# ts-to-rust MCP server v1

One stdio MCP server, two tools.

- `ts_check`: runs TypeScript 7.1 (`tsrs`, fallback `tsc` 7.1) with
  `--noEmit --strict` and returns the verdict and diagnostics.
- `ts_to_rust`: runs `ts_check` first, refuses on failure, then
  transpiles the supported subset to Rust. Unsupported constructs are
  listed in the result, never silently rewritten.

## Leverage, stated plainly

TS 7.1 is the gatekeeper and typechecker here, via its real compiler
binary. TS 7.1's JS API is a native sync channel, not the classic
`createSourceFile` API, so v1 does not claim to parse through that API.
The emitter is a subset parser in this file. A future version can swap
the emitter's front end for the TS 7.1 AST API without changing the
two-tool contract.

## Supported subset

Interfaces, type aliases (string-literal unions, object-literal unions,
Option unions), TS enums, classes (fields, methods, constructor),
typed functions, const/let, if/else, blocks, return, expression
statements. Type map: number -> f64, string -> String, boolean -> bool,
T[] -> Vec<T>, T | null | undefined -> Option<T>.

## Run

```sh
/opt/homebrew/bin/node server.js
```

Protocol is newline-delimited JSON-RPC (MCP stdio). `TSRS_BIN` env
overrides the tsrs path.
