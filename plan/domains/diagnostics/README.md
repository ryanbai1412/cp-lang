# diagnostics

Segment: B. Metric. Owns: `crates/cpl-diag`, `spec/diagnostics.md`, the error sites in the front end (with compiler). Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

## Covers
Error codes, messages, recovery, quick fixes, runtime trap messages, the error catalogue.

## Improve
Checker codes match `spec/diagnostics.md`; quick fixes on every syntax error; no cascades on the broken-ports benchmark on v0.3.

## Success criteria
First error on the right line >= 99% and exactly one error per root cause on the broken-ports benchmark; every code has a catalogue page and a conformance case; every syntax error with an obvious fix offers it.

## Ryan's role
Signs off a one-page message style guide once; after that, metric-driven.
