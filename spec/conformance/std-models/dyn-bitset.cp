// expect: ok, output "ok"
// input: "1 300\n"
// model: DynBitset
// DynBitset against a List<Bool> of its width: seeded random point edits, set algebra with a
// random other set of the same width, shifts, resizes and subset sums, with every query
// compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bitset.{DynBitset}

fn bits(b: DynBitset) -> List<Bool> = [b.contains(i) for i in 0..b.cap]

fn shift(a: List<Bool>, k: Int) -> List<Bool> = [i - k >= 0 and i - k < a.len and a[i - k] for i in 0..a.len]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var b = DynBitset(70)
var a = List.filled(70, false)
var bad = ""
for k in 0..n:
    let m = a.len
    let op = rng.int(0, 17)
    let i = if m == 0: 0 else: rng.int(0, m - 1)
    let kind = rng.int(0, 2)
    var o = [rng.int(0, 3) < kind + 1 for _ in 0..m]
    var ob = if kind == 2 and rng.int(0, 1) == 0: DynBitset.full(m) else: DynBitset(m)
    if ob.len == m: o = List.filled(m, true)
    else:
        for j in 0..m:
            if o[j]: ob.set(j)
    let s = rng.int(0, m + 3)
    if op == 0 and m > 0:
        if b.add(i) != (not a[i]): bad = "op {k}: add {i}"
        a[i] = true
    elif op == 1 and m > 0:
        if b.remove(i) != a[i]: bad = "op {k}: remove {i}"
        a[i] = false
    elif op == 2 and m > 0:
        b.toggle(i)
        a[i] = not a[i]
    elif op == 3 and m > 0:
        let on = kind != 0
        if kind == 2: b.set(i) else: b.set(i, on)
        a[i] = on
    elif op == 4 and kind == 0:
        b.clear()
        a = List.filled(m, false)
    elif op == 5:
        let c = rng.int(0, 140)
        b.resize(c)
        a = [j < m and a[j] for j in 0..c]
    elif op == 6:
        b.union(ob)
        a = [a[j] or o[j] for j in 0..m]
    elif op == 7:
        b.intersect(ob)
        a = [a[j] and o[j] for j in 0..m]
    elif op == 8:
        b.subtract(ob)
        a = [a[j] and not o[j] for j in 0..m]
    elif op == 9:
        b.symmetric(ob)
        a = [a[j] != o[j] for j in 0..m]
    elif op == 10:
        if b.meets(ob) != any(a[j] and o[j] for j in 0..m): bad = "op {k}: meets"
    elif op == 11:
        b.shift_up(s)
        a = shift(a, s)
    elif op == 12:
        b.shift_down(s)
        a = shift(a, -s)
    elif op == 13:
        b.shift_or(s)
        let t = shift(a, s)
        a = [a[j] or t[j] for j in 0..m]
    elif op == 14:
        let c = rng.int(0, 90)
        let ws = [rng.int(0, 30) for _ in 0..rng.int(0, 5)]
        b = DynBitset.subset_sums(c, ws)
        a = [j == 0 for j in 0..c]
        for w in ws:
            let t = shift(a, w)
            a = [a[j] or t[j] for j in 0..c]
    elif op == 15:
        let c = rng.int(0, 90)
        b = if kind == 0: DynBitset.full(c) else: DynBitset.new(c)
        a = List.filled(c, kind == 0)
    if bad != "": break
    let w = a.len
    let ones = [j for j in 0..w if a[j]]
    let q = rng.int(0, w)
    let lo = rng.int(0, w)
    let hi = rng.int(lo, w)
    let nk = rng.int(0, ones.len + 1)
    let want_next = [j for j in ones if j >= q]
    if b.cap != w or bits(b) != a: bad = "op {k}: bits"
    elif b.len != ones.len or b.is_empty != (ones.len == 0): bad = "op {k}: len"
    elif b.each().collect() != ones: bad = "op {k}: each"
    elif b.each_rev().collect() != [ones[ones.len - 1 - j] for j in 0..ones.len]: bad = "op {k}: each_rev"
    elif b.first() != ones.first or b.last() != ones.last: bad = "op {k}: first/last"
    elif b.next(q) != want_next.first: bad = "op {k}: next {q}"
    elif q < w and b.prev(q) != [j for j in ones if j <= q].last: bad = "op {k}: prev {q}"
    elif b.count_below(q) != [j for j in ones if j < q].len: bad = "op {k}: count_below {q}"
    elif b.count_range(lo..hi) != [j for j in ones if j >= lo and j < hi].len: bad = "op {k}: count_range {lo}..{hi}"
    elif b.nth(nk) != (if nk < ones.len: Some(ones[nk]) else: None): bad = "op {k}: nth {nk}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
