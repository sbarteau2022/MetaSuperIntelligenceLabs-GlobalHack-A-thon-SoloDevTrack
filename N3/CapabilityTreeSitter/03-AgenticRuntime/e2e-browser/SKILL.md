---
name: e2e-browser
description: Design and run reliable, permission-aware browser end-to-end tests with Playwright MCP and reproducible test evidence.
disable-model-invocation: true
---

# Browser E2E workflow

Use this skill manually for browser work. Choose the **isolated** Playwright MCP for test data and the **extension** MCP only when an existing signed-in browser session is essential and the user has approved that scope.

1. Define the journey, assertions, test data, browser matrix, and cleanup before opening a page. Prefer local/staging URLs and disposable accounts.
2. Start the app with its documented command. Confirm health/readiness, not merely that a port is open.
3. Test both the happy path and failures: invalid input, no network, slow response, auth expiry, reload/back navigation, mobile viewport, keyboard-only flow, reduced motion, and duplicate submission.
4. Use stable semantic locators. Never assert private implementation details or sleep for a fixed time when a user-visible condition can be awaited.
5. Capture a screenshot, trace, console errors, failed requests, and exact browser/device for each failure. Redact tokens, passwords, customer data, and authenticated screenshots before attaching them anywhere.
6. Verify no production data was created. State what was tested, the URL/environment, result, and coverage gaps.

Do not bypass login, payment, rate limits, CAPTCHAs, consent, or access controls. Browser extension mode reuses the user profile; treat it as high-sensitivity access.
