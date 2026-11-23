// expect: ok, output "YES"
use std.rand.{Rng}
domain N = 0..30
var rng = Rng(3)
var a: [N]Int = 0
for i in N: a[i] = rng.int(-50, 50)
var t = BeatsSegTree<N>(a)
var ok = true
for _ in 0..400:
    let l = rng.int(0, 29)
    let r = rng.int(l, 29)
    let op = rng.int(0, 5)
    let x = rng.int(-20, 20)
    let d = rng.int(1, 7)
    if op == 0:
        t.set(N(l), x)
        a[N(l)] = x
    elif op == 1:
        t.assign(N(l)..=N(r), x)
        for i in N(l)..=N(r): a[i] = x
    elif op == 2:
        t.chdiv(N(l)..=N(r), d)
        for i in N(l)..=N(r): a[i] = a[i] / d
    elif op == 3:
        t.chmod(N(l)..=N(r), d)
        for i in N(l)..=N(r): a[i] = a[i] % d
    elif op == 4:
        t.add(N(l)..=N(r), x)
        for i in N(l)..=N(r): a[i] += x
    else:
        t.chmin(N(l)..=N(r), x)
        for i in N(l)..=N(r): a[i] min= x
    var sm = 0
    var mn = a[N(0)]
    var mx = a[N(0)]
    for i in N:
        sm += a[i]
        mn min= a[i]
        mx max= a[i]
    if t.all_sum != sm or t.all_min != Some(mn) or t.all_max != Some(mx): ok = false
    if t.sum(N(l)..=N(r)) != [a[i] for i in N(l)..=N(r)].sum(): ok = false
print(yes(ok))
