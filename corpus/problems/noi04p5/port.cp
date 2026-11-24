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
// Bounds: n, m: 2..=100
// The slopes have one decimal, so everything is counted in tenths.
let (k1, k2): (F64, F64) = read()
let (a, b) = (round(k1 * 10), round(k2 * 10))
let n: 2..=100 = read()
let m: 2..=100 = read()

// least sum of squares when `len` is cut into `parts` positive integers:
// the parts differ by at most one
fn even_split(len: Int, parts: Int) -> Int:
    let (q, r) = (len / parts, len % parts)
    r * (q + 1) * (q + 1) + (parts - r) * q * q

// south[len, parts]: the south patches under one main piece, cut evenly
let south: [1..=100, 1..=100]Int = [1..=100, 1..=100] (len, parts) => b * even_split(len, parts)

// cost[i, k, j]: i main pieces cover the first k units and hold j parts
var cost: [0..=n, 0..=100, 0..=m]?Int = None
cost[0, 0, 0] = 0
// every later main piece needs at least one unit and one part
for i in 0..n:
    let after = n - i - 1
    for k in i..=100 - after - 1:
        for j in i..=m - after - 1:
            if let base = cost[i, k, j]:
                for len in 1..=100 - k - after:
                    let main = a * len * len
                    for parts in 1..=min(len, m - j - after):
                        cost[
                            i + 1,
                            k + len,
                            j + parts,
                        ] min= base + main + south[len, parts]
let best = cost[n, 100, m]!
print("{best / 10}.{best % 10}")
