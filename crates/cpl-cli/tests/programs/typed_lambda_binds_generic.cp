// A lambda with every parameter annotated binds a bare type parameter `M` like any other argument.
domain V = 0..4
let ws: [V]Int = [3, 1, 4, 1]
fn fold_with<M>(op: M, a: [V]V) -> V:
    var acc = a[V(0)]
    for (_, x) in a.items: acc = op(acc, x)
    acc
print(fold_with((i: V, j: V) => if ws[i] <= ws[j]: i else: j, [V] v => v))
print(fold_with((i: V, j: V) => if ws[i] >= ws[j]: i else: j, [V] v => v))
