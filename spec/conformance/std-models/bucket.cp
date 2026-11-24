// expect: ok, output "ok"
// input: "1 300\n"
// model: BucketCount
// BucketCount over a random range lo..lo+w (lo may be negative) against a List<Int> of counts:
// seeded random add and remove with default and explicit deltas and clear, with count (also
// outside the range), count_lt, kth, len and is_empty compared after each op. Reads `seed n`;
// prints the first op that differs.
use std.rand.{Rng}
use std.bag.{BucketCount}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let lo = rng.int(-30, 30)
let w = rng.int(1, 40)
var b = BucketCount(lo..lo + w)
var c = List.filled(w, 0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let x = rng.int(lo, lo + w - 1)
    let d = rng.int(1, 3)
    if op <= 1:
        b.add(x)
        c[x - lo] += 1
    elif op == 2:
        b.add(x, d)
        c[x - lo] += d
    elif op == 3:
        if c[x - lo] > 0:
            b.remove(x)
            c[x - lo] -= 1
    elif op == 4:
        let t = min(d, c[x - lo])
        if rng.int(0, 1) == 0: b.remove(x, t) else: b.add(x, -t)
        c[x - lo] -= t
    elif rng.int(0, 5) == 0:
        b.clear()
        c = List.filled(w, 0)
    let total = c.sum()
    let q = rng.int(lo - 3, lo + w + 3)
    let want_q = if lo <= q and q < lo + w: c[q - lo] else: 0
    var lt = 0
    for j in 0..w:
        if lo + j < q: lt += c[j]
    var kth_ok = true
    var at = 0
    for j in 0..w:
        for _ in 0..c[j]:
            if b.kth(at) != lo + j: kth_ok = false
            at += 1
    if b.len != total or b.is_empty != (total == 0):
        bad = "op {k}: len {b.len} want {total}"
    elif b.count(q) != want_q:
        bad = "op {k}: count {q} gave {b.count(q)} want {want_q}"
    elif b.count_lt(q) != lt:
        bad = "op {k}: count_lt {q} gave {b.count_lt(q)} want {lt}"
    elif not kth_ok:
        bad = "op {k}: kth"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
