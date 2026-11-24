# ucrpc21h

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the round trip
(2 dist(v) from one Dijkstra), the four-load split capped at 300, the subset-sum over three loads
with the third as a bitset per `(a, b)` cell, and the cost. The port is the dream almost token
for token: `dijkstra(Graph.undirected(roads), ...)`, `fn busiest() -> ?Int` with `None` for the
impossible cases, `domain Load`, and `Bitset<Load>` with `shifted`/`union`.

## Checklist failures
- idiomatic-code/R8: the item loop is `for v in V:` then `if d[v] > 0:` (L27-28) where the dream
  writes the canonical filter `for x in d if x > 0:`. Unreported, but a two-line respelling.
- idiomatic-code/R2 (dream and port): the answer walks `(Load, Load).flat_map(((a, b)) => ...)`
  instead of the spec's `can.items.flat_map(((a, b), cs) => ...)`; the two-parameter closure
  is E0302, in the friction with a repro.

## Owed rewrites
- L27-28: `for x in d if x > 0:` as in the dream.

## Language gaps
- A destructuring two-parameter closure into `flat_map` (E0302; in the friction).

## Dream notes
The dream states the invariant in its header, names Dijkstra and the bitset, keeps every bound
once and returns `?Int` for impossibility. It inherits the `flat_map` workaround above, so 4/5.
