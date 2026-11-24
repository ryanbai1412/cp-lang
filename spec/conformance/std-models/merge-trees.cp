// expect: ok, output "ok"
// input: "1 300\n"
// model: MergeTrees
// A MergeTrees pool against plain arrays: seeded random set, scale and the max-convolution merge
// `new[x] = a[x] * B(<= x) + b[x] * A(< x)` (its result replaces both inputs), with get, sum and
// total checked after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.segtree.{MergeTree, MergeTrees}
type Z = Mod<998244353>
let seed0 = read<Int>()
let n = read<Int>()
domain D = 0..13
var rng = Rng(seed0)
var p = MergeTrees<D, Z>()
var ts: List<MergeTree<D>> = [MergeTree<D>() for _ in 0..4]
var arr: List<[D]Z> = [[D] _ => Z(0) for _ in 0..4]
var bad = ""
for k in 0..n:
    let op = rng.int(0, 3)
    let i = rng.int(0, 3)
    let x = D(rng.int(0, 12))
    let v = Z(rng.int(0, 1000))
    if op <= 1:
        var t = ts[i]
        p.set(mut t, x, v)
        ts[i] = t
        arr[i][x] = v
    elif op == 2:
        p.scale(ts[i], v)
        for y in D: arr[i][y] = arr[i][y] * v
    else:
        let j = (i + rng.int(1, 3)) % 4
        let (a, b) = (arr[i], arr[j])
        ts[i] = p.merge(ts[i], ts[j], (va, vb, pa, pb) => va * (pb + vb) + vb * pa)
        arr[i] = [D] y => a[y] * sum(b[z] for z in D if z <= y) + b[y] * sum(a[z] for z in D if z < y)
        ts[j] = MergeTree<D>()
        arr[j] = [D] _ => Z(0)
    let lo = rng.int(0, 13)
    let hi = rng.int(lo, 13)
    if p.get(ts[i], x) != arr[i][x]: bad = "op {k}: get"
    elif p.sum(ts[i], D.cuts.nth(lo)..D.cuts.nth(hi)) != sum(arr[i][D(y)] for y in lo..hi): bad = "op {k}: sum {lo}..{hi}"
    elif p.total(ts[i]) != sum(arr[i][y] for y in D): bad = "op {k}: total"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
