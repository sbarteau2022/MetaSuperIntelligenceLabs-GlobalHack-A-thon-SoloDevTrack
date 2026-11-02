# dart: sound null safety and an analyzer that stays quiet
**Done means these pass:** `dart format --output=none --set-exit-if-changed .`, `dart analyze --fatal-infos`, `dart test` (Flutter: `flutter analyze`, `flutter test`).
## Rules
1. Sound null safety: avoid `!` and `late` unless the initialisation is guaranteed; prefer `?.`, `??` and early returns. Use `final` and `const` everywhere possible.
2. Model data with immutable classes (`final` fields, `const` constructors), `enum`s with members, and sealed classes with exhaustive `switch` expressions (Dart 3 patterns and records).
3. Async: `async`/`await` with `try`/`catch` at the boundary; never leave a `Future` unawaited (lint `unawaited_futures`, or wrap in `unawaited(...)` deliberately). Use `Stream` for sequences; cancel `StreamSubscription`s and close `StreamController`s.
4. Do not catch `Error` (it signals bugs); catch specific `Exception` types; rethrow with `rethrow` to keep the stack.
5. Use the recommended lints (`package:lints` or `flutter_lints`) and commit `analysis_options.yaml`. Prefer `package:` imports over relative imports across libraries; keep public API in `lib/`, internals in `lib/src/`.
6. Flutter: keep `build` pure and cheap; split widgets into small `const` widgets; keep state at the lowest owner that needs it; dispose controllers, focus nodes and subscriptions in `dispose`; never use a `BuildContext` after an `await` without checking `mounted`.
7. Equality: override `==` and `hashCode` together (or use a package/record), never one alone.
8. Tests with `package:test`; `testWidgets` for widgets; fake clocks and HTTP; one behaviour per test.
## Pitfalls
`late` fields that throw at first read; shared mutable default lists; `==` on lists compares identity; `setState` after dispose; heavy work in `build`.
## Versions
Pinned: Dart 3.11 (3.11 confirmed Mar 2026; 3.12-3.13 builds were seen and may be current; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Dart 3.0.
Rule: first run `dart --version` and read the project's pin (pubspec.yaml environment.sdk). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Effective Dart, the Dart language tour, Flutter docs.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/dart.md).

Canonical source: `plugins/web-development/frontend/languages/dart/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/dart.md` (reference; do not author).
