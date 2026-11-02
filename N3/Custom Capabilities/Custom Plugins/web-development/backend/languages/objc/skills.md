# objc: ARC discipline, nullability and no retain cycles
**Done means these pass:** build with `clang -fobjc-arc -Wall -Wextra -Wpedantic -Werror` (or `xcodebuild test`) with the Address and Undefined Behaviour sanitizers, `clang-format`, and the unit tests.
## Rules
1. ARC always (`-fobjc-arc`). Do not call `retain`, `release` or `autorelease`. Bridge CF types with `__bridge`, `CFBridgingRetain`/`Release` and pair them.
2. Annotate every header with `NS_ASSUME_NONNULL_BEGIN/END` and `nullable` where needed. Use lightweight generics (`NSArray<NSString *> *`).
3. Properties: `strong` for owned objects, `weak` for delegates and back-references, `copy` for `NSString`, `NSArray`, `NSDictionary` and blocks, `assign` only for scalars; mark `nonatomic` unless you need atomic semantics.
4. Blocks capturing `self` that outlive the call: `__weak typeof(self) weakSelf = self;` then `__strong` inside the block and nil-check it. Delegates and notification observers are removed in `dealloc`/`invalidate`.
5. Errors: `NSError **` out-parameters with a boolean or nil return; check the return value, not the error. Exceptions are for programmer errors.
6. Use designated initialisers (`NS_DESIGNATED_INITIALIZER`), call `[super init]`, and check `self`. Prefer modern literals (`@[]`, `@{}`, `@1`), `NS_ENUM`, and `instancetype`.
7. Threading: UI on the main queue only; use GCD or `NSOperationQueue`, never mutate shared collections off one queue; use `dispatch_once` for singletons.
8. Interop with Swift: nullability, `NS_SWIFT_NAME`, `NS_ERROR_ENUM`. Prefer new code in Swift where the project allows.
9. Tests with XCTest; run under ASan, UBSan and Thread Sanitizer.
10. Under `-Werror`, unused parameters fail the build (`int main(void)` or `(void)argc;`) and nullability gaps warn: annotate pointers in headers and avoid writing `_Nonnull` inside array-literal element types. Compile after every edit with the full flag set.
11. Format strings and string arguments: `NSLog(@"%@", s)` takes an `NSString *`; `printf("%s", s.UTF8String)` takes a C string. Never pass an `NSString *` to a C `char *` parameter, and write results with `fputs(s.UTF8String, stdout)` when the output must match exactly (no `NSLog` prefix, it prints to stderr).
## Pitfalls
Retain cycles through blocks, timers and delegates; messaging `nil` silently returns zero; mutable subclasses stored in `strong` properties; `NSNotFound` is not `-1` semantically; main-thread UI violations.
## Versions
Pinned: Apple clang 21 with Xcode 27.0 (the toolchain installed here; Objective-C itself is stable; checked 2026-10-07, source: this machine). This skill assumes at least ARC-era Objective-C 2.0.
Rule: first run `clang --version` and read the project's pin (MACOSX_DEPLOYMENT_TARGET, xcodebuild -version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Apple's Programming with Objective-C, ARC transition guide, Cocoa coding guidelines.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/objc.md).

Canonical source: `plugins/web-development/backend/languages/objc/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/objc.md` (reference; do not author).
