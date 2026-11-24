// Source: DMOPC '21 Contest 8 P5 (Tree Building), https://dmoj.ca/problem/dmopc21c8p5
// limits: 2000 ms, 256 MB
// Problem: a node with i connectors costs a[i] (1 <= i <= K, unlimited supply). Buy N nodes
//   and join connectors pairwise into a tree with none left over: the cheapest total, i.e.
//   min sum a[deg] over degree sequences of an N-node tree. 3 <= N <= 1e9, 2 <= K <= 3000.
// Solution: start from two leaves; a node of degree i + 1 adds itself and i - 1 more leaves,
//   so it is an item of weight i (1 <= i < K) and cost a[i+1] + (i-1)a[1], and the answer is
//   2a[1] + the cheapest exact fill of weight N - 2 (unbounded knapsack). Fill weights up to
//   2K + 2 directly; above that an optimal fill splits into two halves within K of x/2, so
//   dp(x) = min dp(i) + dp(x - i) over K + 1 splits, memoized in a Map: O(K^2 log N).
// original: corpus/submissions/4533533.cpp

let (n, k): (Int, Int) = read()
domain K = 1..=k
let a: [K]Int = read()

let base = max(2 * k + 2, 4)
domain Small = 0..=base

var v: List<Int> = [0]
for i in 1..k:
    v.push(a[K(i + 1)] + (i - 1) * a[K.first!])

var sm: [Small]?Int = [Small] _ => None
sm[Small(0)] = 0
for i in 1..=base:
    var best: ?Int = None
    let jmax = min(i, k - 1)
    for j in 1..=jmax:
        if let prev = sm[Small(i - j)]:
            let cand = prev + v[j]
            if best == None or cand < best!:
                best = cand
    sm[Small(i)] = best

var cache = Map<Int, Int>()

fn dp(x: Int) -> Int:
    if x <= base: return sm[Small(x)]!
    if let val = cache.get(x): return val
    let lo = x / 2 - k / 2 - 1
    var best: ?Int = None
    for i in lo..=(lo + k):
        let cand = dp(i) + dp(x - i)
        if best == None or cand < best!:
            best = cand
    cache[x] = best!
    best!

print(dp(n - 2) + 2 * a[K.first!])
