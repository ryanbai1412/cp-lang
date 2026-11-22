# spec

Segment: D. Taste and co-design. Owns: `spec/core`, `spec/lib`, `spec/tools`, `spec/conformance`. Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

## Covers
The normative spec (core chapters, library guides, tool contracts) and the conformance suite. Writes tests only, never crates. Settles spec ambiguities (via `decisions.md`).

## Improve

- Rule ids + accept/reject cases for every chapter (today mostly ch. 1-3).
- Feature x consumer matrix (domains x generics x closures x multitest x ...): most recent bugs were combinations.
- Trait coverage: every trait method exercised on every claimer (measures it; the generator is stdlib's).
- An independent-rebuild check: a fresh agent with only `spec/` answers sample questions the same way.

## Success criteria
Every normative rule has >=1 accept and (where it forbids) >=1 reject case; 0 empty matrix cells; every spec example is a case; trait coverage 100%. Measured by a coverage report next to the pass rate.

## Ryan's role
Co-designer: settles language-character calls from the queue. Conformance writing itself needs no Ryan time.
