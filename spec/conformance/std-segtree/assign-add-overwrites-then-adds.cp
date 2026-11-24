// expect: ok, output "ok"
// rule: std.algebra.assign_add
// `assign_add` tags are `(set: ?T, add: T)`: overwrite when `set` is there, then add. Random
// range updates on `+`, `min` and `max` trees agree with a plain list updated cell by cell.
use std.rand.{Rng}
domain V = 0..12
var r = Rng.new(7)
var a: [V]Int = [V] v => r.int(0, 19)
var sums = SegTree(+, a, act: assign_add)
var mins = SegTree(min, a, act: assign_add)
var maxs = SegTree(max, a, act: assign_add)
for _ in 0..300:
    let i = V(r.int(0, 11))
    let j = V(r.int(0, 11))
    let (lo, hi) = (min(i, j), max(i, j))
    let s: ?Int = if r.int(0, 1) == 0: None else: Some(r.int(0, 29) - 10)
    let d = r.int(0, 8) - 4
    for v in lo..=hi: a[v] = (s ?? a[v]) + d
    sums.apply(lo..=hi, (set: s, add: d))
    mins.apply(lo..=hi, (set: s, add: d))
    maxs.apply(lo..=hi, (set: s, add: d))
    let x = V(r.int(0, 11))
    let y = V(r.int(0, 11))
    let q = min(x, y)..=max(x, y)
    assert sums.fold(q) == sum(a[v] for v in q)
    assert mins.fold(q) == min(a[v] for v in q)!
    assert maxs.fold(q) == max(a[v] for v in q)!
print("ok")
