use std.tree.{LazyLinkCut}

let n: 2..=200_000 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
let par: [2..=n]V = read()
let add_sum = Action<Int, Int>(0, (f, g) => f + g, (d, x, n) => x + d * n, Monoid<Int>(0, (x, y) => x + y))
var lct = LazyLinkCut<V, Int, Int>.new(add_sum, Some(a))
for v in 2..=n: lct.link(V(v), par[v])
let q: 0..=1_000_000 = read()
var hash = 0
for _ in 0..q:
    let t: 0..=2 = read()
    if t == 0:
        let (u, v, x): (V, V, -10_000..=10_000) = read()
        lct.apply_path(u, v, x)
    elif t == 1:
        let (v, old, p): (V, V, V) = read()
        lct.cut(v, old)
        lct.link(v, p)
    else:
        let (u, v): (V, V) = read()
        hash = (hash * 31 + lct.fold_path(u, v) % 1_000_000_007) % 1_000_000_007
print(hash)
