# yac7p4 review

Score: 3/5
Dream: 4/5

Cold review: rethought. The old port hand-rolled a recursive Euler tour, binary lifting, a
`kth_anc` that recomputed the lifted vertex it already had, and a min segment tree over preorder
(with an INF sentinel) just to find the top of the occupied subtree. Now std `Rooted` gives
`is_anc`, `parent` and `jump_while`, and the top is a tracked `V`: a deleted top hands over to its
one neighbour and an added parent becomes the top, so no segment tree is needed. The highest free
ancestor of a below the top is one `jump_while(a, v => not occupied[v] and tree.is_anc(top, v))`.
The step towards a is a `?(V, V)` (None when a is occupied) instead of three loose optionals,
leaves are a `Set<V>` kept exact by `settle`, and reads are typed (`[0..n - 1](V, V)`).

## Cold review (HEAD)

Port 3/5 (no dream.cp), dream 4/5. The idea reads clearly: one `step` (lines 36-41) that says where the occupied subtree grows, using `tree.is_anc` and `tree.jump_while`, then a sweep of the leaves. It runs at 0.26x the accepted C++'s time and 0.5x its memory.
- Line 43 copies `leaves` into a list just to delete from it while walking it (Owner: stdlib, `remove_where`, in the friction).
- `near: [V]Set<V>` plus `settle` (lines 23-30, 44-58) keeps a live adjacency by hand, and `size`/`top` are updated beside it. That's three facts about one subtree kept in step manually.
- Dream: the friction's dream is the port with the drain fixed. That's honest; the bookkeeping above stays.
