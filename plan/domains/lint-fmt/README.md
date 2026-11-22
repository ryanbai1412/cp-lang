# lint-fmt

Segment: C. Design, then build. Owns: `crates/cpl-lint`, `crates/cpl-fmt`. Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

The source-to-source tools: both work on the lossless syntax tree, offer verified autofixes, and are gated on the idiomatic corpus.

## Covers
**Lint.** The linter, rebuilt per `LINTER-DESIGN.md`.

**Fmt.** The formatter.

## Improve
**Lint.** `@lint(...)` as a general mechanism user libraries can use; each lint comes from an idiom in the skill catalog, with before/after/near-miss tests and a verified autofix.

**Fmt.** Every v0.3 construct formats; comments and blank lines survive; one style, decided once.

## Success criteria
**Lint.** 0 false positives on the idiomatic corpus (corpus-judges); every lint has tests and an autofix verified by recompiling; every catalog idiom with a detectable anti-pattern has a lint.

**Fmt.** `cpl fmt` is idempotent and meaning-preserving (same output from interpreter and C++) on the whole corpus and every spec case; `cpl fmt --check` clean on the corpus and docs snippets.

## Ryan's role
**Lint.** Reviews the lint list and severity policy (what is an error vs a warning vs off), then demos.

**Fmt.** Picks the few style calls the spec leaves open, once.
