// expect: ok, output "YES"
// Built with `values:`, `kth`, `count_lt` and `count_in` after random `set`s agree with the sorted slice.
use std.segtree.{DynamicOrderTree}
var ok = true
let vs: List<Int> = [x for x in -5..=5] + [3, -5]
for trial in 0..20:
    domain W = 0..rand(1, 16)
    var b: [W]Int = [W] _ => rand(-5, 5)
    var t = DynamicOrderTree(b, values: vs)
    for _ in 0..60:
        if rand(0, 2) == 0:
            let i = W(rand(0, W.count - 1))
            let x = rand(-5, 5)
            t.set(i, x)
            b[i] = x
        let l = rand(0, W.count - 1)
        let h = rand(l, W.count - 1)
        var s: List<Int> = []
        for i in l..=h: s.push(b[W(i)])
        s.sort()
        for k in 0..s.len + 2:
            let want = s.get(k)
            if t.kth(W(l)..=W(h), k) != want: ok = false
        let x = rand(-7, 7)
        if t.count_lt(W(l)..=W(h), x) != s.count(v => v < x): ok = false
        if t.count_in(W(l)..=W(h), x, x + 2) != s.count(v => v >= x and v <= x + 2): ok = false
print(if ok: "YES" else: "NO")
