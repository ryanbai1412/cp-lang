// EGOI '23 P1 (Inflation): prices as a multiset keyed by (price - global
// inflation); SET x y merges bucket x into bucket y.
// Runnable on proto/cpc today.
// original: corpus/submissions/5678049.cpp
let n: Int = read()
let p: [0..n]Int = read()
var cnt = Map<Int, Int>()
var total = 0
for x in p:
    cnt[x] = (cnt.get(x) ?? 0) + 1
    total += x
let q: Int = read()
var inflation = 0
for _ in 0..q:
    let kind: Str = read()
    if kind == "INFLATION":
        let x: Int = read()
        inflation += x
    else:
        let u: Int = read()
        let v: Int = read()
        let moved = cnt.get(u - inflation) ?? 0
        if u != v and moved > 0:
            cnt[v - inflation] = (cnt.get(v - inflation) ?? 0) + moved
            cnt.remove(u - inflation)
            total += (v - u) * moved
    print(total + inflation * n)
