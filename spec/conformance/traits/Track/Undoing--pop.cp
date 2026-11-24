// expect: ok, output "2"
domain V = 0..3
fn evict<S: Track<(V, V)>>(mut s: S, e: (V, V)) = s.pop(e)
var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
u.push((V(0), V(1)))
u.push((V(1), V(2)))
evict(mut u, (V(0), V(1)))
print(u.state.components)
