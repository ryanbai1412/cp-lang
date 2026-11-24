// Source: NOI 2004 P5 (Little H's Little Hut), https://dmoj.ca/problem/noi04p5
// limits: 600 ms, 64 MB
// Problem: a 100-wide hut whose north and south walls have slopes k1, k2 (one decimal, <= 50).
//   Split [0, 100] at integer points into m north and n south patches (2 <= m <= n <= 100),
//   every north split point also a south one; a patch of width w along a wall of slope k has
//   area k w^2. Print the least total area (one decimal is exact).
// Solution: each north patch of width w is cut into some j south patches, best as evenly as
//   possible: k1 w^2 + k2 (sum of squares of the even split). DP over (north patches placed,
//   width covered, south patches used), adding one north patch (width, j) per step, in tenths
//   so it stays integer: O(m * 100^2 * n^2) in the worst case. In the port, a, b are k1, k2
//   and n, m are the statement's m, n.
// original: corpus/submissions/3744270.cpp
// The two prices have one decimal, so everything is counted in tenths.
// Each price is read as two Ints, `whole` then `digit`: the prototype has no
// F64, a Str cannot be indexed, and its Int reader swallows the one byte after
// a number, here the decimal point. So the input must spell out one decimal.
// Bounds: n, m: 1..=100
fn read_tenths() -> Int:
    let whole: 0..=1e6 = read()
    let frac: 0..=9 = read()
    whole * 10 + frac

let a = read_tenths()
let b = read_tenths()
let n: 1..=100 = read()
let m: 1..=100 = read()

// least sum of squares when `len` is cut into `parts` positive integers:
// the parts differ by at most one
fn even_split(len: Int, parts: Int) -> Int:
    let (q, r) = (len / parts, len % parts)
    r * (q + 1) * (q + 1) + (parts - r) * q * q

// cost[i, k, j]: i main pieces cover the first k units and hold j parts
var cost: [0..=n, 0..=100, 0..=m]?Int = None
cost[0, 0, 0] = 0
for i in 0..n:
    for k in 0..100:
        for j in 0..m:
            if let base = cost[i, k, j]:
                for len in 1..=100 - k:
                    for parts in 1..=min(len, m - j):
                        cost[
                            i + 1,
                            k + len,
                            j + parts,
                        ] min= base + a * len * len + b * even_split(len, parts)
let best = cost[n, 100, m]!
// no string interpolation in the prototype
print(best / 10, end: "")
print(".", end: "")
print(best % 10)
