# lint

Segment: C. Design, then build. Owns: `crates/cpl-lint`. Worklog: [worklog.md](worklog.md).

## Covers
The linter, rebuilt per `LINTER-DESIGN.md`.

## Improve
`@lint(...)` as a general mechanism user libraries can use; each lint comes from an idiom in the skill catalog, with before/after/near-miss tests and a verified autofix.

## Success criteria
0 false positives on the idiomatic corpus (corpus-judges); every lint has tests and an autofix verified by recompiling; every catalog idiom with a detectable anti-pattern has a lint.

## Ryan's role
Reviews the lint list and severity policy (what is an error vs a warning vs off), then demos.
