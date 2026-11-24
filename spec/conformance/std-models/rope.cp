// expect: ok, output "ok"
// input: "1 300\n"
// model: Rope
// Two Ropes against plain lists: one whose tags overwrite (values are (stamp, value) pairs, so
// query_path's first piece stamped at least the current stamp is the current value) and one
// whose tags add; seeded random apply, reverse and set, with get, query_path and to_array
// checked after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.lazy.{Rope}
let seed0 = read<Int>()
let n = read<Int>()
domain R = 0..12
var rng = Rng(seed0)
var a: List<(Int, Int)> = [(0, rng.int(0, 9)) for _ in 0..12]
var o = Rope([R] i => a[i.ord])
var b: List<Int> = [5 for _ in 0..12]
var s = Rope<R, Int>(fill: 5, merge: (t, x) => t + x)
var bad = ""
for k in 1..=n:
    let op = rng.int(0, 3)
    let lo = rng.int(0, 12)
    let hi = rng.int(lo, 12)
    let r = R.cuts.nth(lo)..R.cuts.nth(hi)
    let x = rng.int(-9, 9)
    let i = rng.int(0, 11)
    if op == 0:
        o.apply(r, (k, x))
        s.apply(r, x)
        for j in lo..hi:
            a[j] = (k, x)
            b[j] += x
    elif op == 1:
        o.reverse(r)
        s.reverse(r)
        a = a[..lo] + a[lo..hi].rev + a[hi..]
        b = b[..lo] + b[lo..hi].rev + b[hi..]
    elif op == 2:
        o.set(R(i), (k, x))
        s.set(R(i), x)
        a[i] = (k, x)
        b[i] = x
    let (st, _) = a[i]
    let seen = o.query_path(R(i), (mut p) => if p.0 >= st: Some(p) else: None)
    if o.get(R(i)) != a[i]: bad = "op {k}: get {i}"
    elif seen != Some(a[i]): bad = "op {k}: query_path {i}"
    elif s.get(R(i)) != b[i]: bad = "op {k}: add get {i}"
    elif rng.int(0, 7) == 0 and ([v for v in o.to_array()] != a or [v for v in s.to_array()] != b): bad = "op {k}: to_array"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
