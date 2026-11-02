# dotnet: build, test, format — pin-driven SDK work
**Done means these pass:** `dotnet build`, `dotnet test`, `dotnet format --verify-no-changes`.
Status: PROPOSED. Date: 2026-10-30.
## Rules
1. Nullable reference types on (`<Nullable>enable</Nullable>`) and respected: guard with `ArgumentNullException.ThrowIfNull`, prefer `required` members and `record` types for immutable data.
2. Async all the way: never `.Result` or `.Wait()`, no `async void` except event handlers, pass a `CancellationToken`, suffix methods `Async`.
3. Log with `ILogger` message templates (`"Loaded {Count} items"`), never `Console.WriteLine` — logs carry the request id, never secrets.
4. Configuration via the Options pattern (`IOptions<T>`); dependencies through constructors; no service locator.
5. EF Core migrations forward-only: review every generated migration, never edit applied migrations, never concatenate into `FromSqlRaw`.
## Pitfalls
Sync-over-async deadlock (`.Result`/`.Wait()` on an async path); tracked-entity bloat on large read sets (use `AsNoTracking`); a connection/client per request instead of reusing the pool (`IHttpClientFactory` for HTTP).
## Versions
NEEDS-VERIFY — `dotnet` is MISSING on this machine (probed 2026-10-30: `dotnet --version` → command not found). Versions are project-pin-driven: read `global.json` and `TargetFramework` first; write only for that pin.
Rule: first run `dotnet --version` and record the measured output; if the project pins an older SDK, follow the project, and if it pins nothing, add a pin.
Sources: `csharp.md`, .NET and ASP.NET Core docs.

Canonical source: `plugins/web-development/backend/languages/dotnet/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/dotnet.md` (reference; do not author).
