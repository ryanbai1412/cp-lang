// expect: ok, output "ok"
// input: "1 300\n"
// model: Fenwick
// Fenwick<V, Int> against a plain array: seeded random adds, each followed by a random range
// fold, prefix and point read. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..37
var rng = Rng(seed0)
var f = Fenwick<V, Int>()
var a = List.filled(37, 0)
var bad = ""
for k in 0..n:
    let (i, x) = (rng.int(0, 36), rng.int(-10, 10))
    f.add(V(i), x)
    a[i] += x
    let lo = rng.int(0, 37)
    let hi = rng.int(lo, 37)
    let want = sum(a[j] for j in lo..hi)
    if f.fold(V.cuts.nth(lo)..V.cuts.nth(hi)) != want:
        bad = "op {k}: fold {lo}..{hi}"
    elif f.prefix(V.cuts.nth(hi)) != sum(a[j] for j in 0..hi):
        bad = "op {k}: prefix {hi}"
    elif f.get(V(i)) != a[i]:
        bad = "op {k}: get {i}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
