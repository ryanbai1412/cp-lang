// expect: ok, output "NO NO 3 3\nYES"
// rule: 08.1.2
// `undoing` passed straight to `minimal` with a positional tuple-pattern `add`: `T` comes from the outer call's element type `(V, V)`, with no annotation on `(u, v)`.
use std.dsu.{Dsu}
domain V = 0..4
let es = [(V(0), V(1)), (V(1), V(2)), (V(2), V(3)), (V(3), V(0))]
let short = es.minimal(undoing(Dsu<V>(rollback: true), (mut d, (u, v)) => d.unite(u, v)), ok: w => w.state.components == 1)
let typed = es.minimal(undoing(Dsu<V>(rollback: true), (mut d, (u, v): (V, V)) => d.unite(u, v)), ok: w => w.state.components == 1)
print(" ".join(short.map(s => if let w = s: w.len.to_str else: "NO")))
print(yes(short.map(s => s.map(w => w.len)) == typed.map(s => s.map(w => w.len))))
