// expect: ok, output "ok"
// input: "1 300\n"
// model: lis_ends
// model: supermasks
// model: masks_of_size
// model: subset_folds
// model: subset_sums
// model: backtrack
// The small DP helpers against their obvious loops: LIS ends by an O(n^2) scan, supermasks and
// masks of a size by filtering every mask, subset folds and sums by summing each mask's bits,
// and backtrack over a random DAG by a recursive preorder walk. `Step.prune()` is not driven:
// it is lost when `backtrack` runs inside a block or fn (on both engines).
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.dp.{backtrack, lis_ends, masks_of_size, subset_folds, subset_sums, supermasks}

fn walk(kids: List<List<Int>>, s: Int, d: Int, mut out: List<(Int, Int)>):
    out.push((s, d))
    for c in kids[s]: walk(kids, c, d + 1, mut out)

fn popcount(x: Int) -> Int:
    var c = 0
    var y = x
    while y > 0:
        c += y & 1
        y >>= 1
    c

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 4)
    if op == 0:
        let m = rng.int(0, 10)
        let a = [rng.int(-4, 4) for _ in 0..m]
        let strict = rng.int(0, 1) == 0
        let neg = rng.int(0, 1) == 0
        let got = if neg: a.lis_ends(strict: strict, by: x => -x) else: a.lis_ends(strict: strict)
        for i in 0..m:
            var want = 1
            for j in 0..i:
                let (x, y) = if neg: (-a[j], -a[i]) else: (a[j], a[i])
                if (x < y or (not strict and x == y)) and got[j] + 1 > want: want = got[j] + 1
            if got[i] != want:
                bad = "op {k}: lis_ends at {i} = {got[i]} want {want}"
                break
    elif op == 1:
        let bits = rng.int(0, 6)
        let m = rng.int(0, (1 << bits) - 1)
        let kk = rng.int(0, bits + 1)
        if supermasks(m, bits).collect() != [s for s in 0..(1 << bits) if s & m == m]: bad = "op {k}: supermasks {m} {bits}"
        elif masks_of_size(bits, kk).collect() != [s for s in 0..(1 << bits) if popcount(s) == kk]:
            bad = "op {k}: masks_of_size {bits} {kk}"
    elif op == 2:
        let m = rng.int(0, 6)
        let xs = [rng.int(-9, 9) for _ in 0..m]
        var sums: List<Int> = []
        var mins: List<Int> = []
        for s in 0..(1 << m):
            var t = 0
            var lo = 9223372036854775807
            for i in 0..m:
                if s >> i & 1 == 1:
                    t += xs[i]
                    lo = min(lo, xs[i])
            sums.push(t)
            mins.push(lo)
        if subset_folds(xs, +) != sums: bad = "op {k}: subset_folds +"
        elif subset_folds(xs, min) != mins: bad = "op {k}: subset_folds min"
        else:
            var want = sums.clone()
            want.sort()
            if subset_sums(xs) != want: bad = "op {k}: subset_sums"
    else:
        let m = rng.int(1, 9)
        var kids: List<List<Int>> = []
        for s in 0..m:
            var c: List<Int> = []
            for _ in 0..rng.int(0, 2):
                if s + 1 < m: c.push(rng.int(s + 1, m - 1))
            kids.push(c)
        var want: List<(Int, Int)> = []
        walk(kids, 0, 0, mut want)
        var got: List<(Int, Int)> = []
        for st in backtrack(0, s => kids[s]):
            got.push((st.state, st.depth))
        if got != want: bad = "op {k}: backtrack {m}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
