// expect: ok, output "10 10\n2 4 4 3 2\n2 2 | 7 | 1 2\n2 2"
// rule: R04.2.4 R04.2.13 R04.2.19
// A domain's own members are `Domain`'s ordinary fns: `V.range` is its whole range, `V.wrap`
// steps Euclidean mod its count, and `v.after`/`v.before` are the pieces beside `v`. A bound
// reaches the same fns.
domain V = 1..=4
domain W = 0..5
let a = [V] v => Int(v)
print(a[V.range].sum(), a[a.keys].sum())
print(Int(W.wrap(7)), Int(W.wrap(-1)), Int(V.wrap(-1)), Int(V.wrap(6)), Int(W.wrap(-8)))
let v = W(2)
print(v.after.len, v.before.len, "|", a[V(2).after].sum(), "|", W(1).before.len, V(3).before.len)
fn ends<D: Domain>(d: D) -> (Int, Int) = (d.after.len, D.wrap(d.ord + 1).before.len)
let (x, y) = ends(V(2))
print(x, y)
