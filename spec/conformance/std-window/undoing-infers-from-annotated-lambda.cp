// expect: ok, output "YES"
// rule: 08.1.2
// `undoing`'s `E` comes from the lambda's annotated parameter `e: (V, V)`, with no `Undoing<..>` written; today `E` defaults to `Int`.
domain V = 0..3
var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
u.push((V(0), V(1)))
print(yes(u.state.same(V(0), V(1))))
