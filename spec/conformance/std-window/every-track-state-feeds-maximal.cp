// expect: ok, output "1 2 3 1 1 2 3 2 1 2 2 2"
// rule: std.seq.maximal
// A `Track` from another module is a `state:` for `maximal` like a `track`: a `SortedBag`, a `Tracked` and an `Undoing` (sequence-patterns §5).
let a = [5, 7, 6, 9]
let spread = a.maximal(state: SortedBag<Int>(), ok: b => b.last! - b.first! <= 2)
let sum = a.maximal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: t => t.state <= 20)
domain V = 0..4
let es: [V](V, V) = [(V(0), V(1)), (V(1), V(2)), (V(2), V(3)), (V(3), V(0))]
let apart = es.maximal(state: undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)), ok: u => u.state.components >= 2)
print([w.len for w in spread], [w.len for w in sum], [w.len for w in apart])
