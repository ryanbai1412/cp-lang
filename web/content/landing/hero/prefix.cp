let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
var pre: [V.cuts]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]
let q: 1..=2e5 = read()
for _ in 0..q:
    let (l, r): (V, V) = read()
    print(pre[r.r] - pre[l.l])
