// expect: ok, output "YES"
// `kth` after random `set`s is the `k`-th smallest of the sorted slice, `None` past its end.
use std.segtree.{DynamicOrderTree}
var ok = true
for trial in 0..20:
    domain W = 0..rand(1, 16)
    var b: [W]Int = [W] _ => rand(-5, 5)
    var t = DynamicOrderTree(b)
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
            let want: ?Int = if k < s.len: Some(s[k]) else: None
            if t.kth(W(l)..=W(h), k) != want: ok = false
print(if ok: "YES" else: "NO")
