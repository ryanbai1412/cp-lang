// expect: ok, output "ok"
// rule: std.beats.BeatsSegTree.chmin
// A tree keeps second extremes only from its first `chmin` or `chmax` on, and a node whose
// minimum equals its maximum hands that value down: random add, chdiv, chmod and assign rounds
// before and after the first chmin/chmax agree with a brute-force list on every query.
use std.beats.{BeatsSegTree}
use std.rand.{Rng}

domain P = 0..21

var rng = Rng(7)
var ok = true
for round in 0..30:
    var a = [P] _ => rng.int(-40, 40)
    var t = BeatsSegTree(a)
    let start = rng.int(0, 80)
    for step in 0..120:
        let i = rng.int(0, 20)
        let j = rng.int(i, 20)
        let r = P(i)..=P(j)
        let x = rng.int(-30, 30)
        let op = if step < start: rng.int(0, 3) else: rng.int(0, 5)
        if op == 0:
            t.add(r, x)
            for p in r: a[p] += x
        elif op == 1:
            let d = rng.int(1, 6)
            t.chdiv(r, d)
            for p in r: a[p] = a[p] / d
        elif op == 2:
            let m = rng.int(1, 9)
            t.chmod(r, m)
            for p in r: a[p] = a[p] % m
        elif op == 3:
            t.assign(r, x)
            for p in r: a[p] = x
        elif op == 4:
            t.chmin(r, x)
            for p in r: a[p] min= x
        else:
            t.chmax(r, x)
            for p in r: a[p] max= x
        let q = P(rng.int(0, 10))..=P(rng.int(10, 20))
        let vals = [a[p] for p in q]
        if (
            t.sum(q),
            t.min_val(q),
            t.max_val(q),
        ) != (vals.sum(), vals.min(), vals.max()): ok = false
        let k = P(rng.int(0, 20))
        if t.get(k) != a[k]: ok = false
    if t.all_sum != a.sum() or t.all_min != a.min() or t.all_max != a.max(): ok = false
print(if ok: "ok" else: "mismatch")
