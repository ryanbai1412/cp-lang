# ccc22s2

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header names the `Map<Str, Group>` and the two counts.
- R2: map names to groups, count broken rules: the editorial.
- R4: the report's first section describes `group_of[groups[k].a] = k` / `.b` / `.c`; the port now reads `[3]Str` rows (L19-20), so the report is stale.
- R10: 0.12 s, 89 MB vs 0.08 s, 11 MB (1.5x time, 8x memory) at `--max`, with no performance friction; now filed.

## Owed rewrites

- Update the report's first section to the `[3]Str` read the port uses.

## Language gaps

- `Map<Str, _>` over 3e5 names: 1.5x time, 8x memory (added, measured).

## Dream notes

Builds the map as one expression; reads like the editorial.
