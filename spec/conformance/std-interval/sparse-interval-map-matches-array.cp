// expect: ok, output "1\n3 1 5 10 4..5 5..6 0 1 0 5..10 3..5 0..3\n1 0\n3\n1 2 1\nYES"
use std.interval.{SparseIntervalMap}
use std.rand.{Rng}
print(SparseIntervalMap(0, 1_000_000_000, 0).runs)
var m = SparseIntervalMap(0, 10, 0)
m.assign(3..5, 1)
print(m.runs, m.get(4), m.run_at(7).range.l, m.run_at(7).range.r, m.range(4..6).map(p => p.range).collect(), m.each().map(p => p.val).collect(), m.each_rev().map(p => p.range).collect())
var old: List<Int> = []
m.assign(4..6, 2, on_old: (_, v) => old.push(v))
print(old)
var n = SparseIntervalMap(0, 10, 0)
n.assign(3..5, 1)
n.assign(5..7, 1)
print(n.runs)
var q = SparseIntervalMap(0, 10, 0)
q.assign(3..5, 1)
q.update(0..4, v => v + 1)
print(q.get(0), q.get(3), q.get(4))
var rng = Rng(4)
var z = SparseIntervalMap(-20, 20, 0)
var arr = List.filled(40, 0)
var ok = true
for _ in 0..300:
    let a = rng.int(-20, 20)
    let b = rng.int(a, 20)
    if rng.int(0, 1) == 0:
        let v = rng.int(0, 2)
        var seen = 0
        z.assign(a..b, v, on_old: (rr, _) => seen += rr.len)
        if seen != b - a: ok = false
        for i in a..b: arr[i + 20] = v
    else:
        z.update(a..b, x => (x + 1) % 3)
        for i in a..b: arr[i + 20] = (arr[i + 20] + 1) % 3
    var runs = 1
    for i in 1..40:
        if arr[i] != arr[i - 1]: runs += 1
    if z.runs != runs: ok = false
    for i in -20..20:
        if z.get(i) != arr[i + 20]: ok = false
print(yes(ok))
