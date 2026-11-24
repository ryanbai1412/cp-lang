// expect: R0401
// rule: std.tree.Rooted
// No value is "its own reverse": a path fold over a monoid made without `rev:` stops at its
// first climbing stretch, at the `fold_path` call that reached it (R09.4.5), naming `rev:`, `.commutative()` and `impl Reversible`.
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
let t = Rooted(adj, root: 0)
let digits = Monoid(0, (a, b) => a * 10 + b)
let seg = SegTree(digits, t.reorder([1, 2, 3]))
print(t.fold_path(1, 2, seg))  // <- here
