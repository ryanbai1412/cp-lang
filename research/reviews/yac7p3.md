# yac7p3

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the answer (a
component with cycle length L allows exactly L groups, so the total is the sum of cycle lengths),
the single repointing that gives a cycle its deepest tail, and the labelling, with the cost.
`Functional` carries `on_cycle`, `depth`, `root_of`, `cycles()` and `cycle_len`, so no walk is
hand-rolled; `deep` and `moved` are a `max(by:)` and a `find`.

## Checklist failures
- idiomatic-code/R2: a cycle slot is kept in two parallel arrays, `idx` and `base` (L21-22,
  L26-27), where the dream keeps one `spot: [V](base: Int, i: Int)`. Today's language allows the
  dream's record, so this is a port-side respelling; the friction's entry is about the wider
  per-cycle tree walk.

## Owed rewrites
- Replace `idx`/`base` with the dream's `spot` record.

## Language gaps
- `Functional` has no per-cycle walk of the hanging trees (`tree_from(c)` with depths), so each
  vertex looks its root up (in the friction).

## Dream notes
The dream keeps every fact once and names the functional-graph structure; it still labels via
`spot[g.root_of(v)]` rather than the per-cycle walk the friction proposes. 4/5.
