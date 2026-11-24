// expect: ok, output "YES"
// rule: std.segtree.SegTree.fold
// copies: 1
// A fold over a replacing action (`assign`, `replaces:`) stops at the first tag that sets its
// span (`assign_add` too); a non-replacing action, a commuting one, and `None` (assign's id) tags agree with an array.
use std.algebra.{Action}
var ok = true
for trial in 0..30:
    domain V = 0..rand(1, 24)
    var b: [V]Int = [V] _ => rand(-9, 9)
    var c = b
    let plain = Action<Int, ?Int>(None, (nf, of) => nf ?? of, (f, v, span) => if let x = f: x * span else: v, over: +)
    var set = SegTree(+, b, act: assign)
    var keep = SegTree(+, b, act: plain)
    var add_t = SegTree(+, b, act: add)
    var mx = SegTree(max, b, act: assign)
    var aa = SegTree(+, b, act: assign_add)
    for _ in 0..80:
        let l = rand(0, V.count - 1)
        let h = rand(l, V.count - 1)
        let r = V(l)..=V(h)
        let k = rand(0, 3)
        if k == 0:
            let x = rand(-9, 9)
            set.apply(r, Some(x))
            keep.apply(r, Some(x))
            mx.apply(r, Some(x))
            aa.apply(r, (set: Some(x), add: 0))
            add_t.apply(r, x)
            for i in l..=h:
                b[V(i)] = x
                c[V(i)] += x
        elif k == 1:
            set.apply(r, None)
            keep.apply(r, None)
            mx.apply(r, None)
            aa.apply(r, (set: None, add: 0))
        else:
            var (s, m, d) = (0, Int.MIN, 0)
            for i in l..=h:
                s += b[V(i)]
                m = max(m, b[V(i)])
                d += c[V(i)]
            if set.fold(r) != s or keep.fold(r) != s or add_t.fold(r) != d or mx.fold(r) != m or aa.fold(r) != s: ok = false
print(if ok: "YES" else: "NO")
