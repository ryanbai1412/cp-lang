# ioi08p6

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-11) gives both cases; `cheapest` with `last_true` (lines 21-39) and the two-pointer over columns with a runs-at-minimum `SegTree` (lines 64-90) follow it.
- `Strip`, `at_low`, `join` and `cover_by` (lines 43-76) are the classic min-and-runs node by hand; in the friction (medium).
- `at_low` and the action rebuild every field (lines 45-51, 74); in the friction (no struct update).
- The span type `(lo: Y, hi: Y, cost: Int)` is written twice (lines 24-25) (idiomatic-code/R4); a `type Span` says it once.
