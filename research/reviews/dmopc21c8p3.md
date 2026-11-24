# dmopc21c8p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

`most[i]` is the largest inversion-like count reachable with i values, capped at k + 1; values then pick offsets greedily, and a `Fenwick` of open positions places each value at its offset.

## Checklist failures

- `elif free == 5 and left == 4` is a special case with no comment.
- `most` is indexed by `0..=n` while values use `Val`, so `most[free - 1]` mixes counts and domain ordinals.
- `open.first_gt(offset[v])!.left!` unwraps twice to find the k-th open position; an order-statistic `nth` would say it.
- `offset` and `left` are mutated through three branches of one loop.
