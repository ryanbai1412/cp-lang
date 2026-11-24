// Source: DMOPC '21 Contest 8 P5 (Tree Building), https://dmoj.ca/problem/dmopc21c8p5
// limits: 2000 ms, 256 MB
// Problem: a node with i connectors costs a[i] (1 <= i <= K, unlimited supply). Buy N nodes
//   and join connectors pairwise into a tree with none left over: the cheapest total, i.e.
//   min sum a[deg] over degree sequences of an N-node tree. 3 <= N <= 1e9, 2 <= K <= 3000.
// Solution: start from two leaves; a node of degree i + 1 adds itself and i - 1 more leaves,
//   so it is an item of weight i (1 <= i < K) and cost a[i+1] + (i-1)a[1], and the answer is
//   2a[1] + the cheapest exact fill of weight N - 2 (unbounded knapsack). Fill weights up to
//   2K + 2 directly; above that an optimal fill splits into two halves within K of x/2, so
//   dp(x) = min dp(i) + dp(x - i) over K + 1 splits. The weights needed at each depth form a window of about 2K
//   around N / 2^d, filled bottom-up as one list per depth: O(K^2 log N).
// original: corpus/submissions/4533533.cpp
// Bounds: n: 3..=1e9; k: 2..=3000

let n: 3..=1e9 = read()
let k: 2..=3000 = read()
domain Deg = 1..=k
let a: [Deg]Int = read()

// item i (1 <= i < k): a node of degree i + 1 adds weight i at cost a[i + 1] + (i - 1) a[1]
fn cost(i: Int) -> Int = a[Deg(i + 1)] + (i - 1) * a[Deg.first!]

let base = 2 * k + 2
var small: [0..=base]Int = 0
for w in 1..=base:
    small[w] = min(small[w - i] + cost(i) for i in 1..=min(w, k - 1))!

fn half(x: Int) -> Int = x / 2 - k / 2 - 1  // splits of x take i in half(x)..=half(x) + k

// the weights each level needs: level 0 is n - 2; the next holds both halves of every split
var levels: List<(lo: Int, hi: Int)> = [(n - 2, n - 2)]
while levels.last!.hi > base:
    let (lo, hi) = levels.last!
    levels.push((max(half(lo), 0), hi - half(hi)))

var below: List<Int> = []
var below_lo = 0
fn fill(x: Int) -> Int = if x <= base: small[x] else: below[x - below_lo]
for (lo, hi) in levels.rev if hi > base:
    below = [
        if x <= base: small[x] else: min(fill(i) + fill(x - i) for i in half(x)..=half(x) + k)!
        for x in lo..=hi
    ]
    below_lo = lo

print(fill(n - 2) + 2 * a[Deg.first!])
