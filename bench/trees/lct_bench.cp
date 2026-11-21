use std.tree.{LinkCut}

let n: 2..=200_000 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
let par: [2..=n]V = read()
var lct = LinkCut<V, Int>.new(+, [V] v => a[v])
for v in 2..=n: lct.link(V(v), par[v])
let q: 0..=1_000_000 = read()
var hash = 0
for _ in 0..q:
    let t: 0..=2 = read()
    if t == 0:
        let (v, x): (V, -1e9..=1e9) = read()
        lct.set(v, x)
    elif t == 1:
        let (v, old, p): (V, V, V) = read()
        lct.cut(v, old)
        lct.link(v, p)
    else:
        let (u, v): (V, V) = read()
        hash = (hash * 31 + lct.fold_path(u, v)) % 1_000_000_007
print(hash)
