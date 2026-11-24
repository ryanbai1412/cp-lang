// expect: ok, output "ok"
// input: "1 300\n"
// model: SparseTable
// model: DisjointSparseTable
// model: LinearRmq
// model: idempotent
// SparseTable (min), DisjointSparseTable (affine maps under composition, which do not commute)
// and LinearRmq (min and max, 150 positions so queries cross 64-wide blocks) against a plain list,
// rebuilt after each random point change; each op also checks a random range fold, try_fold, get,
// all, argbest and idempotent against loops. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.sparse.{SparseTable, DisjointSparseTable, LinearRmq, idempotent}
domain V = 0..150
let modp = 1_000_003

fn then(f: (Int, Int), g: (Int, Int)) -> (Int, Int) = (f.0 * g.0 % modp, (f.1 * g.0 + g.1) % modp)
fn aff(x: Int) -> (Int, Int) = (x % 7 + 1, x)

fn brute_min(a: List<Int>, lo: Int, hi: Int) -> Int:
    var m = a[lo]
    for i in lo..hi: m = min(m, a[i])
    m

fn brute_arg(a: List<Int>, lo: Int, hi: Int, big: Bool) -> Int:
    var b = lo
    for i in lo..hi:
        if (big and a[i] > a[b]) or (not big and a[i] < a[b]): b = i
    b

fn brute_aff(a: List<Int>, lo: Int, hi: Int) -> (Int, Int):
    var acc = (1, 0)
    for i in lo..hi: acc = then(acc, aff(a[i]))
    acc

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var a = [rng.int(0, 20) for _ in 0..150]
var bad = ""
for k in 0..n:
    let i = rng.int(0, 149)
    a[i] = rng.int(0, 20)
    let arr: [V]Int = [V] v => a[v.ord]
    let st = SparseTable<V, Int>(min, arr)
    let dst = DisjointSparseTable<V, (Int, Int)>(then, [V] v => aff(a[v.ord]))
    let lm = LinearRmq(arr)
    let lx = LinearRmq(arr, before: (x, y) => x > y)
    let lo = rng.int(0, 150)
    let hi = rng.int(lo, 150)
    let r = V.cuts.nth(lo)..V.cuts.nth(hi)
    if lo == hi:
        if st.try_fold(r) != None or dst.try_fold(r) != None or lm.try_best(r) != None:
            bad = "op {k}: empty {lo}..{hi} not None"
    elif st.fold(r) != brute_min(a, lo, hi) or st.try_fold(r) != Some(brute_min(a, lo, hi)):
        bad = "op {k}: SparseTable fold {lo}..{hi}"
    elif dst.fold(r) != brute_aff(a, lo, hi) or dst.try_fold(r) != Some(brute_aff(a, lo, hi)):
        bad = "op {k}: DisjointSparseTable fold {lo}..{hi}"
    elif lm.argbest(r).ord != brute_arg(a, lo, hi, false) or lm.best(r) != brute_min(a, lo, hi):
        bad = "op {k}: LinearRmq argbest {lo}..{hi} = {lm.argbest(r)}"
    elif lx.argbest(r).ord != brute_arg(a, lo, hi, true) or lx.try_best(r) != Some(a[brute_arg(a, lo, hi, true)]):
        bad = "op {k}: LinearRmq max argbest {lo}..{hi} = {lx.argbest(r)}"
    if bad == "" and (st.get(V(i)) != a[i] or dst.get(V(i)) != aff(a[i])):
        bad = "op {k}: get {i}"
    if bad == "" and (st.all != brute_min(a, 0, 150) or dst.all != brute_aff(a, 0, 150)):
        bad = "op {k}: all"
    let xs = [a[j] for j in lo..hi]
    let zero = xs.all(x => x == 0)
    if bad == "" and (not idempotent<Int>(min, xs) or idempotent<Int>(+, xs) != zero):
        bad = "op {k}: idempotent on {xs}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
