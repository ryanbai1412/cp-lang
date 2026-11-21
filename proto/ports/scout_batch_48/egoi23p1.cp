// EGOI '23 P1 (Inflation): prices as a multiset keyed by (price - global
// inflation); SET x y merges bucket x into bucket y.
// original: corpus/submissions/5678049.cpp
let n: 1..=3e5 = read()
let p: [0..n]Int = read()
var cnt = HashMap<Int, Int>()
for x in p: cnt[x] += 1
var total = p.sum()
let q: 1..=1e5 = read()
var inflation = 0
for _ in 0..q:
    let kind: Str = read()
    if kind == "INFLATION":
        let x: Int = read()
        inflation += x
    else:
        let (u, v): (Int, Int) = read()
        if u != v:
            if let moved = cnt.get(u - inflation):
                cnt.remove(u - inflation)
                cnt[v - inflation] += moved
                total += (v - u) * moved
    print(total + inflation * n)
