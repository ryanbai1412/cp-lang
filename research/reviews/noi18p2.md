# noi18p2

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The header proves the no-length-3-decreasing characterisation and the ballot count.

## Checklist failures

- Line 23 spells the case marker `@multitest`; `@!multitest` is the one marker (idiomatic-code/R8).
- Line 15 sizes `Comb` as `2 * 600_000 + 5`, restating n's bound with slack instead of deriving it from the type (idiomatic-code/R4).
- `used: [1..=n + 1]Bool` (line 26) and `(smallest..=n + 1).find(...)!` (line 33) make n + 1 a sentinel for "every value used" (idiomatic-code/R5).
- `ballot`'s `n - low - 2` (line 21) converts between "maximum" and "low" by an offset the doc comment does not derive (idiomatic-code/R1).

## Owed rewrites

- Write the dream.
- Use `@!multitest`.

## Language gaps

- None.
