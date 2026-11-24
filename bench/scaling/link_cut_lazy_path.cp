// n: 2048
use std.rand.{Rng}
use std.tree.{LazyLinkCut}
let n: Int = read()
domain V = 0..n
var rng = Rng(3)
var par: [V]Int = [V] v => if v.ord == 0: -1 else: rng.int(0, v.ord - 1)
let vals: [V]Int = [V] v => v.ord % 10
let act = Action<Int, Int>(apply: (d, x, k) => x + d * k)
var s = 0
var lz = LazyLinkCut(act, a: Some(vals))
for v in V:
    if v.ord > 0:
        lz.link(v, V(par[v]))
for i in 0..32 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    s += lz.get(a) // timed: tree.LazyLinkCut.get
    lz.set(a, i % 10) // timed: tree.LazyLinkCut.set
    lz.apply_path(a, b, 1) // timed: tree.LazyLinkCut.apply_path
    s += lz.fold_path(a, b) // timed: tree.LazyLinkCut.fold_path
    s += lz.dist(a, b) ?? 0 // timed: tree.LazyLinkCut.dist
    s += (lz.jump(a, b, 2) ?? a).ord // timed: tree.LazyLinkCut.jump
    s %= 1000000007
print(s)
