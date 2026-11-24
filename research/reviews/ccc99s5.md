# ccc99s5

Score: 3/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: no `// Bounds:` line; `words: [Row]Str` (L19) is unbounded where the statement caps a word at 20 capital letters.
- R2: column-by-column backtracking, as the editorial.
- R4: `used: [Digit]Bool` mirrors `digit` and is kept in step at L35 and L37; the dream drops it (`digit.contains(Some(d))`).
- R7: the pending choice rides along as `give:` because a closure continuation lowers to `std::function` (reported).
- R10: 0.00 s vs 1.11 s.

## Owed rewrites

- Add `// Bounds:` and bound `words`; drop `used`.

## Language gaps

- Scoped assignment or inlined closure for backtracking (in the report).

## Dream notes

`fn fill(col: Int, row: Row, total: Int) -> Bool: ...` elides the search itself, so 3.
