# noi02p1

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (`dream.cp`). A `PotentialDsu<Ship>` keeps each ship's place in its column, commands are a tagged enum read by tag, and a query is `line.diff(i, j).map(d => abs(d) - 1) ?? -1` (line 25), as in the dream.

## Checklist failures

- idiomatic-code/R4: the column's front ship is kept beside the DSU in `head` (line 14) and repaired after every merge (line 24), because `PotentialDsu` roots by size. Reported in the friction.

## Owed rewrites

- `repeat t: match read<Cmd>()` (lines 19-20) could stream as the dream's `for cmd in read<[t]Cmd>()`; equivalent, cosmetic.

## Language gaps

- Union with a chosen root or a stable potential origin: in the friction.

## Dream notes

The dream keeps the same `front` table and patches it after `unite`; the friction's own ideal (the front is the root) is the stronger dream and should replace those two lines once `PotentialDsu` grows a chosen-root union.
