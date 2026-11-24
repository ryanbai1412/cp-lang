# ccc22s2

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header names the `Map<Str, Group>` and the two counts.
- R2: map names to groups, count broken rules: the editorial.
- R3: the map is filled by a read-and-insert loop over `[3]Str` rows (L18-20) where `Map((name, k) for k in Group for name in groups[k])` works (`cpl run` in the report).
- R10: 0.12 s, 89 MB vs 0.08 s, 11 MB (1.5x time, 8x memory) at `--max`, with no performance friction; now filed.

## Owed rewrites

- Read `groups: [Group][3]Str` and build `group_of` with one `Map(...)` generator, then re-measure.

## Language gaps

- `Map<Str, _>` over 3e5 names: 1.5x time, 8x memory (added, measured).

## Dream notes

Builds the map as one expression; reads like the editorial.
