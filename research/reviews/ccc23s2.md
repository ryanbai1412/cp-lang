# ccc23s2

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the centre expansion, O(n^2).
- R2: expand every centre, odd and even, and keep the least cost per length: the editorial.
- R6: `h` is indexed by raw `Int` (`h[centre - j]`, L19), reported (E0301).
- R5: `least: [Len]?Int` with `!` at print (L21).

## Owed rewrites

None.

## Language gaps

- Palindromic expansion is index arithmetic (in the report).

## Dream notes

Names the expansion; reads like the editorial.
