# noi06p4

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (the friction's `## Dream`). The header's maximum-weight closure as a min cut, built in five lines with `None` for the uncapped arcs (lines 23-29), and the answer is total payment minus the max flow.

## Checklist failures

- idiomatic-code/R5: `cost: [Station]Int` and `pay: Int` (lines 14-15) leave the statement's 0..=100 bounds out of the reads and the header's Bounds line.
- The flow nodes are numbered by hand with `station`/`group` (lines 18-21): reported (no constructible sum domain, E0302 quoted).

## Owed rewrites

- Bound `cost` and `pay` as the statement does.

## Language gaps

- Disjoint-union domains: in the friction.

## Dream notes

`domain Node = Station + User + End` removes the numbering exactly; the rest of the dream is the port.
