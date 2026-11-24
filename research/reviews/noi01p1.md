# noi01p1

Score: 4/5
Dream: 3/5

Port 4/5, dream 3/5 (the friction's `## Dream`). Kinds are `Mod<3>` and std `PotentialDsu<Animal, Kind>` (line 14) answers whether a statement is consistent, with labels past n rejected by the `Animal.at` let-else (line 18): the editorial in a dozen lines.

## Checklist failures

- idiomatic-code/R7: `holds` (line 17) is named as a test but records the statement through `known.unite`, and line 24 calls it inside an `if` condition.

## Owed rewrites

- Name the operation for what it does (`record(claim, x, y) -> Bool`, "false when it contradicts what is known"), so line 24 reads as recording, not testing.

## Language gaps

None beyond the friction.

## Dream notes

`print(count(not known.unite(...) for ... in read(k)))` hides the DSU's mutation inside a counting generator (idiomatic-code/R7), which caps the dream at 3. The port's loop is the honest shape; the dream should be that loop with a typed `?Animal` read.
