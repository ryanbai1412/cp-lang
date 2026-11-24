// expect: ok, output "ok"
// input: "1 300\n"
// model: DynamicOrderTree
// DynamicOrderTree against a plain array: seeded random point sets, each followed by get,
// count_lt, count_in and kth over a random range. Reads `seed n`; prints the first op that
// differs.
use std.rand.{Rng}
use std.segtree.{DynamicOrderTree}
let seed0 = read<Int>()
let n = read<Int>()
domain W = 0..25
var rng = Rng(seed0)
var a: [W]Int = [W] _ => rng.int(0, 15)
var t = DynamicOrderTree(a)
var bad = ""
for k in 0..n:
    let i = W(rng.int(0, 24))
    let x = rng.int(0, 15)
    t.set(i, x)
    a[i] = x
    let lo = rng.int(0, 25)
    let hi = rng.int(lo, 25)
    let (u, v) = (rng.int(-1, 16), rng.int(-1, 16))
    let j = rng.int(0, hi - lo)
    var ys = [a[W(p)] for p in lo..hi]
    ys.sort()
    let r = W.cuts.nth(lo)..W.cuts.nth(hi)
    if t.get(i) != x: bad = "op {k}: get"
    elif t.count_lt(r, u) != ys.count(y => y < u): bad = "op {k}: count_lt {lo}..{hi} {u}"
    elif t.count_in(r, u, v) != ys.count(y => u <= y and y <= v): bad = "op {k}: count_in {u} {v}"
    elif t.kth(r, j) != (if j < ys.len: Some(ys[j]) else: None): bad = "op {k}: kth {j}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
