// expect: ok, output "ok"
// input: "1 300\n"
// model: sort
// model: sorted
// model: reverse
// model: dedup
// model: deduped
// model: distinct
// model: partition
// model: partition_in_place
// model: stable_partition_in_place
// model: select
// model: radix_sort
// model: counting_sort
// model: rotate_left
// model: rotate_right
// model: rotated_left
// model: rotated_right
// model: resized
// model: swap_at
// model: swap
// model: adjacent_diff
// model: record_highs
// model: inversions
// The std reorderings of a List against a stable insertion sort and index loops: each op
// draws a short random list of (value, position) pairs and checks every sort (stable ones on
// ties), partition, selection, rotation, reversal, dedup and count. Reads `seed n`; prints the
// first op that differs.
use std.rand.{Rng}
domain K = 0..5
domain W = 0..9

fn isort(a: List<(Int, Int)>) -> List<(Int, Int)>:
    var b = a.clone()
    for i in 1..b.len:
        var j = i
        while j > 0 and b[j - 1].0 > b[j].0:
            let x = b[j]
            b[j] = b[j - 1]
            b[j - 1] = x
            j -= 1
    b

fn rot(a: List<(Int, Int)>, lo: Int, hi: Int, r: Int) -> List<(Int, Int)>:
    let m = hi - lo
    [if lo <= j < hi: a[lo + ((j - lo + r) % m + m) % m] else: a[j] for j in 0..a.len]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let len = rng.int(0, 8)
    let a = [(rng.int(-3, 3), i) for i in 0..len]
    let lo = rng.int(0, len)
    let hi = rng.int(lo, len)
    let r = rng.int(-9, 9)
    let sz = rng.int(0, 10)
    let srt = isort(a)
    let keep = (q: (Int, Int)) => q.0 > 0
    var why = ""
    var s = a.clone()
    s.sort(by: .0)
    var rs = a.clone()
    rs.radix_sort(by: .0)
    var cs = a.clone()
    cs.counting_sort(by: q => K(q.0 + 3 - (q.0 + 3) / 5 * 5))
    var cb = a.clone()
    for i in 1..cb.len:
        var j = i
        while j > 0 and (cb[j - 1].0 + 3) % 5 > (cb[j].0 + 3) % 5:
            let x = cb[j]
            cb[j] = cb[j - 1]
            cb[j - 1] = x
            j -= 1
    var rv = a.clone()
    rv.reverse(range: Some(lo..hi))
    var full = a.clone()
    full.reverse()
    var rl = a.clone()
    rl.rotate_left(r, range: Some(lo..hi))
    var rr = a.clone()
    rr.rotate_right(r, range: Some(lo..hi))
    var fl = a.clone()
    fl.rotate_left(r)
    var pi = a.clone()
    let mid = pi.partition_in_place(keep)
    var sp = a.clone()
    let smid = sp.stable_partition_in_place(keep)
    let yes = [q for q in a if keep(q)]
    let no = [q for q in a if not keep(q)]
    let v = [q.0 for q in a]
    var dd = v.clone()
    dd.dedup()
    let adj = [v[j] for j in 0..len if j == 0 or v[j] != v[j - 1]]
    let dist = [v[j] for j in 0..len if all(v[i] != v[j] for i in 0..j)]
    var highs: List<Int> = []
    for j in 0..len:
        if all(v[i] < v[j] for i in 0..j): highs.push(j)
    var inv = 0
    for i in 0..len:
        for j in i + 1..len:
            if v[i] > v[j]: inv += 1
    var sw: [W]Int = [W] w => w.ord * 10 + r
    sw.swap_at(W(lo), W(hi))
    var swb = [j * 10 + r for j in 0..9]
    let x = swb[lo]
    swb[lo] = swb[hi]
    swb[hi] = x
    if s != srt: why = "sort"
    elif a.sorted(by: .0) != srt or v.sorted() != [q.0 for q in srt]: why = "sorted"
    elif a.sorted(by: q => desc(q.0)) != isort([(-q.0, q.1) for q in a]).map(q => (-q.0, q.1)).collect(): why = "sorted desc"
    elif rs != srt: why = "radix_sort"
    elif cs != cb: why = "counting_sort"
    elif rv != [if lo <= j < hi: a[hi - 1 - (j - lo)] else: a[j] for j in 0..len]: why = "reverse range"
    elif full != [a[len - 1 - j] for j in 0..len]: why = "reverse"
    elif lo < hi and rl != rot(a, lo, hi, r): why = "rotate_left range"
    elif lo < hi and rr != rot(a, lo, hi, -r): why = "rotate_right range"
    elif len > 0 and fl != rot(a, 0, len, r): why = "rotate_left"
    elif len > 0 and (a.rotated_left(r) != rot(a, 0, len, r) or a.rotated_right(r) != rot(a, 0, len, -r)): why = "rotated"
    elif len == 0 and (a.rotated_left(r) != [] or a.rotated_right(r) != []): why = "rotated empty"
    elif a.resized(sz, (9, 9)) != [if j < len: a[j] else: (9, 9) for j in 0..sz]: why = "resized"
    elif [sw[w] for w in W] != swb: why = "swap_at"
    elif a.any(q => (u: q.0, v: q.1).swap != (u: q.1, v: q.0)): why = "swap"
    elif mid != yes.len or pi.take(mid).collect().sorted(by: .1) != yes or pi.drop(mid).collect().sorted(by: .1) != no: why = "partition_in_place"
    elif smid != yes.len or sp != yes + no: why = "stable_partition_in_place"
    elif a.partition(keep) != (yes, no): why = "partition"
    elif dd != adj or v.deduped() != adj: why = "dedup"
    elif v.distinct() != dist: why = "distinct"
    elif a.distinct(by: q => q.0 * q.0).map(q => q.1).collect() != [j for j in 0..len if all(v[i] * v[i] != v[j] * v[j] for i in 0..j)]: why = "distinct by"
    elif v.adjacent_diff() != [if j == 0: v[0] else: v[j] - v[j - 1] for j in 0..len]: why = "adjacent_diff"
    elif v.record_highs().collect() != highs: why = "record_highs"
    elif v.inversions() != inv: why = "inversions"
    if why == "" and len > 0:
        let j = (lo + hi) % len
        var se = a.clone()
        se.select(j, by: .0)
        if se.nth(j).0 != srt[j].0 or any(se[i].0 > srt[j].0 for i in 0..j) or any(se[i].0 < srt[j].0 for i in j + 1..len):
            why = "select {j}"
        elif se.sorted(by: .1) != a: why = "select keeps elements"
    if why != "":
        bad = "op {k}: {why} on {a} ({lo}..{hi}, r {r}, n {sz})"
        break
print(if bad == "": "ok" else: bad)
