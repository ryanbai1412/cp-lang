// Source: EGOI 2023 Day 1 P1 (Inflation), https://dmoj.ca/problem/egoi23p1
// Problem: N <= 3e5 prices p_i <= 1e6, then Q <= 1e5 days, each "INFLATION x" (add x to every
//   price) or "SET x y" (every price equal to x becomes y), x, y <= 1e6. After each day print
//   the sum of all prices (needs 64 bits).
// Solution: keep a global offset and a map from price - offset to its count; the sum is the
//   sum of keys + N * offset. SET moves the whole count at key x - offset onto y - offset and
//   adds (y - x) * count to the key sum. O((N + Q) log N).
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
