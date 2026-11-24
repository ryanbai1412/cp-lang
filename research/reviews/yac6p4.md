# yac6p4

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port.

The program is four lines under a table: read the sides, XOR their Grundy values with `map(...).fold(^)`, and print the least side in `1..=m` with `(1..=m).find(...) ?? -1`. Reads carry the statement bounds (n <= 1e6, sides and m <= 100). There is no sentinel and no index arithmetic. The port matches the dream except for a named `sides` read, which the `// Bounds:` header needs.

## Checklist failures

- The `grundy` table is 101 magic numbers, "precomputed offline by the original author", with no program that derives them. A reader can't check them, and a variant with sides over 100 has nowhere to start. The Sprague-Grundy recurrence over cut and nail moves could be computed at start-up, or kept beside the table as the generator that produced it.
- `let grundy: List<Int> = [...]` carries an annotation the literal already implies (the dream drops it).

## Gaps that need the language

- The table's one-number-per-line layout: research/frictions/yac6p4.md (Owner: fmt).
