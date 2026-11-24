# cco09p4

Score: 4/5
Dream: 4/5

The editorial's nuts-and-bolts quicksort over two domains: a random pivot bottle splits the caps by `group`, its cap splits the other bottles by `partition`, and the two sides recurse. The interactor checks the 5e5 query budget.

## Checklist failures

None.

## Owed rewrites

- `[b for b in bottles if b != pivot]` (line 24) is fine; nothing else is owed.

## Language gaps

- `group` returns a map, so the sides need `by_fit.get(..) ?? []` and the fitting cap is `by_fit[0][0]`: research/frictions/cco09p4.md (`partition3`).

## Dream notes

- The dream is the port with `partition3` and `without`; it has no header and drops the reads, but nothing in it reads like the C++.
