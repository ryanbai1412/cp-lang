# diagnostics

Segment: B. Metric. Owns: `crates/cpl-diag`, `crates/cpl-fmt`, `spec/diagnostics.md`. Worklog: [worklog.md](worklog.md).

## Covers
Error codes, messages, recovery, quick fixes, runtime trap messages, the error catalogue, and the formatter (merged from fmt).

## Improve
Checker codes match `spec/diagnostics.md`; quick fixes on every syntax error; no cascades on the broken-ports benchmark on v0.3; every v0.3 construct formats, comments and blank lines survive.

## Success criteria
First error on the right line >= 99% and exactly one error per root cause on the broken-ports benchmark; every code has a catalogue page and a conformance case; every syntax error with an obvious fix offers it; `cpl fmt` idempotent and meaning-preserving on the corpus and every spec case, `--check` clean on corpus and docs.

## Ryan's role
Signs off a one-page message style guide and the open format-style calls once; after that, metric-driven.
