# cobol: exact decimals, scoped statements, checked I/O
**Done means these pass:** it compiles clean with GnuCOBOL (`cobc -x -free -Wall -debug program.cob`), runs and produces the expected output on tests including boundary data, and file statuses are all checked.
## Rules
1. Know the source format: fixed (columns 7-72, indicator in 7, area A from 8, area B from 12) or free (`-free`). State which one; code that is valid in one can be misread in the other.
2. Four divisions in order: `IDENTIFICATION`, `ENVIRONMENT`, `DATA`, `PROCEDURE`. Give every program a `PROGRAM-ID`; end with `STOP RUN` (or `GOBACK` in subprograms).
3. Numbers: use `PIC S9(n)V9(m)` for signed decimals, `COMP-3` (packed) for stored money, never floating point for money; size fields for the largest value; initialise every field (`VALUE`, `INITIALIZE`); always include `ON SIZE ERROR` on arithmetic that can overflow, and `ROUNDED` where rounding is intended.
4. Use explicit scope terminators (`END-IF`, `END-PERFORM`, `END-EVALUATE`, `END-READ`, `END-COMPUTE`) instead of relying on the period. Use `EVALUATE` for multi-way branches; avoid `GO TO` and `ALTER`; structure code into paragraphs or sections invoked with `PERFORM ... THRU` only with a clear range.
5. File I/O: declare `FILE STATUS`; check it after every `OPEN`, `READ`, `WRITE`, `REWRITE`, `CLOSE`; handle `AT END`/`INVALID KEY`; close every file; prefer reading until `AT END` with a switch (88-level condition names) over counting records.
6. Data hygiene: use level-88 condition names for flags, `COPY` books for shared record layouts, meaningful hyphenated names, `REDEFINES` rarely (document it), and `OCCURS ... DEPENDING ON` with a validated counter.
7. Strings: fixed-length and space-padded; use `STRING ... DELIMITED BY SIZE ... INTO ... WITH POINTER ... ON OVERFLOW`, `UNSTRING`, `INSPECT`, and reference modification `(start:length)` with bounds checked; use `FUNCTION TRIM`, `FUNCTION LENGTH`.
8. Validate input data (numeric class tests: `IF field IS NUMERIC`), dates (use `FUNCTION CURRENT-DATE`, integer-of-date for arithmetic), and reject bad records to an error file with a reason; do not let a bad record abort a batch run silently.
9. Batch discipline: return codes (`MOVE n TO RETURN-CODE`), control totals and record counts printed at end, restartability notes; no hard-coded paths (use environment/DD names).
10. Test with edge records: empty file, one record, max-size values, negative amounts, spaces in numeric fields.
## Pitfalls
A stray period ending an `IF` early; unsigned fields silently dropping the sign; truncation on `MOVE` to a smaller field; decimal alignment with `V`; reading past `AT END`; uninitialised `WORKING-STORAGE` assumed to be zero.
## Versions
Pinned: COBOL (ISO/IEC 1989:2023), GnuCOBOL 3.2 (GnuCOBOL 3.2 (2023) is the latest release found; 4.0 is in development; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least COBOL 2002 features.
Rule: first run `cobc --version` and read the project's pin (the build script, the compiler dialect option). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the GnuCOBOL programmer's guide, ISO/IEC 1989:2023 (COBOL), IBM Enterprise COBOL programming guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/cobol.md).

Canonical source: `plugins/web-development/backend/languages/cobol/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/cobol.md` (reference; do not author).
