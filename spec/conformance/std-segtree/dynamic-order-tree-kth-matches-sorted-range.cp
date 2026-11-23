// expect: ok, output "YES"
// `DynamicOrderTree.kth` after random `set`s is the `k`-th of the sorted range, `None` past its end.
use std.segtree.{DynamicOrderTree}
use std.rand.Rng

domain W = 0..40
var rng = Rng(11)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
var b: [W]Int = 0
for i in W: b[i] = r(mut rng, 12) - 6
var t = DynamicOrderTree(b)
var ref = [b[i] for i in W]
var ok = true
for _ in 0..3000:
    let l = r(mut rng, 40)
    let h = l + r(mut rng, 40 - l)
    if r(mut rng, 2) == 0:
        let x = r(mut rng, 12) - 6
        t.set(W(l), x)
        ref[l] = x
    else:
        var s = [ref[i] for i in l..=h]
        s.sort()
        let k = r(mut rng, s.len + 2)
        let want: ?Int = if k < s.len: Some(s[k]) else: None
        if t.kth(W(l)..=W(h), k) != want: ok = false
print(if ok: "YES" else: "NO")
