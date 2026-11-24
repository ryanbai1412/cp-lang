// expect: ok, output "ok"
// rule: std.beats.BeatsSegTree.sum
// `sum`, `min_val`, `max_val` and `get` read pending tags without pushing them, so they work on
// a tree passed by value and agree with a brute-force list under random chmin, chmax and add.
use std.beats.{BeatsSegTree}

domain P = 0..13

fn probe(t: BeatsSegTree<P>, r: Range<P>) -> (Int, ?Int, ?Int) = (t.sum(r), t.min_val(r), t.max_val(r))

var ok = true
for round in 0..40:
    var a = [P] _ => rand(-20, 20)
    var t = BeatsSegTree(a)
    for step in 0..60:
        let i = rand(0, 12)
        let j = rand(i, 12)
        let r = P(i)..=P(j)
        let x = rand(-25, 25)
        let op = rand(0, 3)
        if op == 0:
            t.chmin(r, x)
            for p in r: a[p] = min(a[p], x)
        elif op == 1:
            t.chmax(r, x)
            for p in r: a[p] = max(a[p], x)
        elif op == 2:
            t.add(r, x)
            for p in r: a[p] += x
        let q = P(rand(0, 6))..=P(rand(6, 12))
        let vals = [a[p] for p in q]
        let want = (vals.sum(), vals.min(), vals.max())
        if probe(t, q) != want or probe(t, q) != want: ok = false
        let k = P(rand(0, 12))
        if t.get(k) != a[k]: ok = false
print(if ok: "ok" else: "mismatch")
