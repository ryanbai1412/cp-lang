// Source: EGOI 2022 Day 1 (Lego Wall), https://dmoj.ca/problem/egoi22p2
// Problem: count the walls of width w and height h (w * h <= 5e5) built from 1x1 and 2x1
//   bricks with no holes that are connected (bricks touching one above the other link),
//   mod 1e9+7.
// Solution: a wall is disconnected exactly when some inner vertical line cuts every row.
//   For w <= 1e4: a row of width i has fib(i) tilings, so total[i] = fib(i)^h, and splitting
//   off the connected part before the first full cut gives solid[i] = total[i] - sum over
//   j < i of solid[j] * total[i - j]: O(w^2). Otherwise h <= 50: walk the cuts from the
//   right keeping, per number j of rows a brick ends at, the ways to finish; a row a brick
//   crosses must end one at the next cut, so ways[j] = sum C(j, k) next[h - j + k], and an
//   inner cut with all h rows ending counts nothing: O(w h^2).
// Bounds: w, h: 1..=5e5
// original: corpus/submissions/5199888.cpp

type Z = Mod<1_000_000_007>

let w: 1..=5e5 = read()
let h: 1..=5e5 = read()

if w <= 1e4:
    var tilings: [0..=w]Z = Z(1)
    for i in 2..=w: tilings[i] = tilings[i - 1] + tilings[i - 2]
    let total: [0..=w]Z = [0..=w] i => tilings[i] ** h
    var solid: [0..=w]Z = Z(0)
    for i in 1..=w: solid[i] = total[i] - sum(solid[j] * total[i - j] for j in 1..i)
    print(solid[w])
else:
    let comb = Comb<Z>(h)  // ways[j]: j rows end a brick at this cut
    var ways: [0..=h]Z = Z(0)
    ways[h] = Z(1)
    for cut in (0..w).rev:
        ways = [0..=h] j => sum(comb.choose(j, k) * ways[h - j + k] for k in 0..=j)
        if cut > 0: ways[h] = Z(0)
    print(ways[h])
