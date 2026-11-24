# cco01p4io

Score: 3/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R2: the `drops`/`up` DP is two parallel arrays filled by index (L15-20); the dream wants one table built from itself.
- R8: `s.idx` (L17-18) where the corpus spells `.ord`.
- The report's entries have no ideal code, general fix or owner; the self-referential table is now filed.

## Owed rewrites

- `s.idx` to `s.ord`; one `(cost:, up:)` table.

## Language gaps

- `ask` replies cannot be declared in `// Bounds:` (in the report).
- A table defined from itself (added, `cpl run`: E0205).

## Dream notes

`ask("Drop", at)` used as a `Bool` hides the 0/1 reply type; otherwise the editorial.
