// Source: AtCoder Educational DP Contest M (Candies), https://atcoder.jp/contests/dp/tasks/dp_m
// limits: 2000 ms, 1024 MB
// Problem: count the ways to share exactly K <= 1e5 candies among N <= 100 children, child i
//   getting 0..a_i of them (a_i <= K), modulo 1e9+7.
// Solution: ways[g] = ways to give out g candies to the children so far; the next child with
//   cap a makes next[g] = sum of ways[g - a ..= g], a sliding window over the previous row.
//   O(N K).
// original: corpus/submissions/4131778.cpp
// Bounds: n: 1..=100; k: 0..=1e5; a: 0..=k
type Z = Mod<1_000_000_007>
let n: 1..=100 = read()
let k: 0..=1e5 = read()
domain Given = 0..=k
let a: [n]0..=k = read()

// ways[g]: ways to give out g candies among the children so far. The next
// child takes up to `most` of them, so it sums a window of the row before.
var ways: [Given]Z = 0
ways[Given.first!] = 1
for most in a:
    var next: [Given]Z = 0
    var window: Z = 0  // ways[g - most ..= g]
    for g in Given:
        window += ways[g]
        if let gone = Given.at(g - (most + 1)): window -= ways[gone]
        next[g] = window
    ways = next
print(ways[Given.last!])
