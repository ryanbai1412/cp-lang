// expect: ok, output "YES"
// rule: std.segtree.SegTree.apply
// A tag put on a node that holds none is stored as it is, and a pushed node keeps its stale tag
// unread: random non-commuting affine tags on a summing tree and on a dual tree (no `op:`),
// composed by `compose` or in place by `compose_in`, agree with an array on every fold and get.
use std.algebra.{Action, Monoid}
type Aff = (a: Int, b: Int)
fn comp(g: Aff, f: Aff) -> Aff = (g.a * f.a, g.a * f.b + g.b)
fn onto(mut f: Aff, g: Aff):
    f.b = g.a * f.b + g.b
    f.a = g.a * f.a
var ok = true
for trial in 0..30:
    domain V = 0..rand(1, 24)
    var b: [V]Int = [V] _ => rand(-9, 9)
    let plain = Action<Int, Aff>((1, 0), comp, (f, v, span) => f.a * v + f.b * span, over: +)
    let inplace = Action<Int, Aff>(
        (1, 0),
        comp,
        (f, v, span) => f.a * v + f.b * span,
        over: +,
        compose_in: (mut o, g) => onto(mut o, g),
    )
    let point = Monoid<Int>(0, (x, _) => x)
    var whole = SegTree(+, b, act: plain)
    var sum_in = SegTree(+, b, act: inplace)
    var dual = SegTree<V, Int, Aff>(
        a: b,
        act: Action<Int, Aff>((1, 0), comp, (f, v, _) => f.a * v + f.b, over: point),
    )
    var dual_in = SegTree<V, Int, Aff>(
        a: b,
        act: Action<Int, Aff>(
            (1, 0),
            comp,
            (f, v, _) => f.a * v + f.b,
            over: point,
            compose_in: (mut o, g) => onto(mut o, g),
        ),
    )
    for _ in 0..80:
        let l = rand(0, V.count - 1)
        let h = rand(l, V.count - 1)
        let r = V(l)..=V(h)
        if rand(0, 2) > 0:
            let f = (a: if rand(0, 1) == 0: -1 else: 1, b: rand(-5, 5))
            whole.apply(r, f)
            sum_in.apply(r, f)
            dual.apply(r, f)
            dual_in.apply(r, f)
            for i in r: b[i] = f.a * b[i] + f.b
        else:
            let s = sum(b[i] for i in r)
            if whole.fold(r) != s or sum_in.fold(r) != s: ok = false
            for i in r:
                if dual.get(i) != b[i] or dual_in.get(i) != b[i] or whole.get(i) != b[i]: ok = false
print(if ok: "YES" else: "NO")
