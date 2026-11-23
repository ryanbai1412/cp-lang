// expect: ok, output "3"
domain V = 0..3
let u: Undoing<Dsu<V>, (V, V)> = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
print(u.state.components)
