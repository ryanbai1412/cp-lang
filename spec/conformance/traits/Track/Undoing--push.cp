// expect: ok, output "1"
domain V = 0..3
fn add<S: Track<(V, V)>>(mut s: S, e: (V, V)) = s.push(e)
var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
add(mut u, (V(0), V(1)))
add(mut u, (V(1), V(2)))
print(u.state.components)
