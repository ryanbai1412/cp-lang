// expect: ok, output "213 6"
// rule: std.tree.Rooted
// A path fold reads each climbing stretch backwards through its monoid's `rev`: a
// non-commutative op passes `rev:`, a commutative one says `.commutative()`. The `rev`
// default is a private std fn, so a field default resolves in the struct's own module.
// Digits concatenate as (number, its digits reversed, 10^length).
domain V = 0..3
let adj: [V]List<V> = [[1, 2], [0], [0]]
let t = Rooted(adj, root: 0)
let cat = Monoid((0, 0, 1), (a, b) => (a.0 * b.2 + b.0, b.1 * a.2 + a.1, a.2 * b.2), x => (x.1, x.0, x.2))
let digits = SegTree(cat, t.reorder([(1, 1, 10), (2, 2, 10), (3, 3, 10)]))
let sums = SegTree(Monoid(0, (a, b) => a + b).commutative(), t.reorder([1, 2, 3]))
print(t.fold_path(1, 2, digits).0, t.fold_path(1, 2, sums))
