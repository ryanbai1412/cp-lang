# dmopc20c4p4

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port.

One pass over the throws: `holes` counts the holes made so far, `spare` keeps the throws that can still be pulled back, and each prefix's total is pushed as it goes.

## Checklist failures

- Each throw's `behind` is read inside the loop (`let behind: 0..=Int(i) = read()`), because its bound depends on the index; the dream reads them all at once.
- `total += Int(i) - Int(spare.pop()!)` converts throws to `Int` for a distance and unwraps a pop that `holes < behind` guarantees.
- `holes` and `total` are bare counters mutated in both branches; the two cases of the greedy have no names.

## Gaps that need the language

- The throws read one at a time: research/frictions/dmopc20c4p4.md.
