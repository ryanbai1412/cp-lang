// expect: ok, output "YES"
// rule: std.beats.BeatsSegTree.chmin
// Each op set a tree can be used with, no clamp, only `chmin`, only `chmax` or both, with `add`,
// `chdiv`, `chmod`, `assign` and `set` mixed in, answers every query as a brute force does.
use std.beats.{BeatsSegTree}
use std.rand.{Rng}

domain N = 0..24

fn matches(seed: Int, lo: Bool, hi: Bool) -> Bool:
    var rng = Rng(seed)
    var a: [N]Int = 0
    for i in N: a[i] = rng.int(-60, 60)
    var t = BeatsSegTree(a)
    var ok = true
    for _ in 0..500:
        let l = rng.int(0, 23)
        let r = rng.int(l, 23)
        let rg = N(l)..=N(r)
        let x = rng.int(-30, 30)
        let d = rng.int(1, 6)
        let op = rng.int(0, 6)
        if op == 0:
            t.add(rg, x)
            for i in rg: a[i] += x
        elif op == 1:
            t.chdiv(rg, d)
            for i in rg: a[i] = a[i] / d
        elif op == 2:
            t.chmod(rg, d)
            for i in rg: a[i] = a[i] % d
        elif op == 3:
            t.assign(rg, x)
            for i in rg: a[i] = x
        elif op == 4 and lo:
            t.chmin(rg, x)
            for i in rg: a[i] min= x
        elif op == 5 and hi:
            t.chmax(rg, x)
            for i in rg: a[i] max= x
        elif op == 6:
            t.set(N(l), x)
            a[N(l)] = x
        let part = [a[i] for i in rg]
        if t.sum(rg) != part.sum() or t.min_val(rg) != part.min() or t.max_val(rg) != part.max():
            ok = false
        if t.all_sum != [a[i] for i in N].sum(): ok = false
    for i in N:
        if t.get(i) != a[i]: ok = false
    ok

print(
    yes(
        matches(1, false, false) and
            matches(2, true, false) and
            matches(3, false, true) and
            matches(4, true, true)
    )
)
