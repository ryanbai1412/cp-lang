// Source: JOI Open 2016, Skyscraper, https://dmoj.ca/problem/joi16op3
// limits: 1000 ms, 512 MB
// Problem: N <= 100 distinct heights <= 1000. Count the orders of the buildings whose sum of
//   |neighbour height differences| is <= L (L <= 1000), mod 1e9+7.
// Solution: insert buildings from the shortest up into a row of groups. ways[g, c, e] has g
//   groups, e of them pinned to an end of the row, and cost c charged as if every open side
//   already met the current height; each new building pays (h - level) per open side, then starts
//   a group, extends one or joins two, optionally at an end. The answer is one group with both
//   ends: O(N^2 L).
// original: corpus/submissions/4226622.cpp
// Bounds: n: 1..=100; limit, heights: 1..=1000
type Z = Mod<1_000_000_007>
let (n, limit): (1..=100, 1..=1000) = read()
domain Building = 0..n
let heights: [Building]1..=1000 = read()
assert heights.sorted().pairs().all(p => p.0 < p.1)

// ways[g, c, e]: g groups, e of them already fixed at an end of the row, and c paid so far,
// counting every open side of a group as if its future neighbour were as tall as the building
// just placed.
type Ways = [0..=n, 0..=limit, 0..=2]Z
var ways: Ways = 0
ways[0, 0, 0] = 1
var level = heights.min()!
for h in heights.sorted():
    var next: Ways = 0
    for g in 0..n:
        for c in 0..=limit:
            for e in 0..=2:
                let w = ways[g, c, e]
                let sides = 2 * g - e  // open sides, all rising to h
                let paid = c + (h - level) * sides
                if w != 0 and paid <= limit:
                    next[g + 1, paid, e] += w * (g + 1 - e)            // a group of its own
                    if e < 2: next[g + 1, paid, e + 1] += w * (2 - e)  // ... at an end
                    if g >= 1:
                        next[g, paid, e] += w * sides                  // beside one group
                        if e < 2: next[g, paid, e + 1] += w * (2 - e)  // ... at an end
                    if g >= 2: next[g - 1, paid, e] += w * (g - 1)     // between two groups
    ways = next
    level = h
print(if n == 1: Z.one else: sum(ways[1, c, 2] for c in 0..=limit))
