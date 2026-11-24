# yac7p4 review

Score: pending
Dream: pending

Cold review: rethought. The old port hand-rolled a recursive Euler tour, binary lifting, a
`kth_anc` that recomputed the lifted vertex it already had, and a min segment tree over preorder
(with an INF sentinel) just to find the top of the occupied subtree. Now std `Rooted` gives
`is_anc`, `parent` and `jump_while`, and the top is a tracked `V`: a deleted top hands over to its
one neighbour and an added parent becomes the top, so no segment tree is needed. The highest free
ancestor of a below the top is one `jump_while(a, v => not occupied[v] and tree.is_anc(top, v))`.
The step towards a is a `?(V, V)` (None when a is occupied) instead of three loose optionals,
leaves are a `Set<V>` kept exact by `settle`, and reads are typed (`[0..n - 1](V, V)`).
