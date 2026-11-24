// Source: IOI 2025 Day 1 (Triple Peaks), https://github.com/ioi/task-archive/tree/main/2025/day1/triples
// limits: 2000 ms, 2048 MB
// Problem: peaks 0..N have heights in 1..=N - 1; (i, j, k), i < j < k, is mythical when its
//   heights are its pairwise distances j - i, k - j, k - i in some order. Part I counts the
//   mythical triples of H; Part II, given M and K, builds 3 <= N <= M heights with at least K.
// Solution: Part I: the position holding k - i is unique. An end holding it, or the middle with
//   h[i] = j - i, pins the triple from one position in O(1); the middle with h[i] = k - j
//   pairs positions with equal h[x] - x, all pairs in groups of at most sqrt(n) and a solve
//   per k in the larger ones: O(n sqrt n). Part II: peaks at the midpoints of pairs from a
//   random even point set make every 3 of its points a mythical triple; keep the best of
//   8e5 / M tries, by Part I's count.
// Stdin: the sample grader's: 1, N, H for count_triples (prints T), or 2, M K for
//   construct_range (prints N and H). The judge form is `@grader fn count_triples(h:
//   List<1..2e5>) -> Int` and `@grader fn construct_range(m: 3..=2e5, k: Int) -> List<Int>`;
//   the corpus runs a port on stdin with no grader.cpp, so this one reads the grader's input.
// original: corpus/problems/ioi25_triples/orig.cpp (the official model solution plus the sample grader)
// Bounds: part: 1..=2; n: 3..=2e5; h: [n]1..n; m: 3..=2e5; k: 1..=12_000_000
use std.rand.{Rng}

// Part 1. Name a triple's positions i < j < k; its distances are j−i, k−j and the
// largest, k−i, which exactly one position holds. If an end holds it, two heights
// pin the third position, and if the middle does with h[i] = j−i, so does h[i]: O(1)
// candidates per position. Left is "middle holds k−i, crossed" (h[i] = k−j,
// h[k] = j−i): there i and j share h[x] − x, and a √n split between small groups (all
// pairs) and large groups (solve for i, j from each k) bounds it. One predicate
// checks a candidate i < j < k and that `top` holds k−i, so each triple counts once.
fn sort3(a: Int, b: Int, c: Int) -> (Int, Int, Int):
    let (lo, hi) = (min(a, b, c), max(a, b, c))
    (lo, a + b + c - lo - hi, hi)

fn count_triples(h: List<Int>) -> Int:
    let at = 0..h.len
    var found = 0
    fn try(i: Int, j: Int, k: Int, top: Int):
        if i < j < k and i in at and k in at and h[top] == k - i and sort3(
            h[i],
            h[j],
            h[k],
        ) == sort3(
            j - i,
            k - j,
            k - i,
        ):
            found += 1

    for x in at:  // x is an end holding k−i
        let y = x + h[x]
        if y in at:
            try(x, x + h[y], y, top: x)
            if 2 * h[y] != h[x]: try(x, y - h[y], y, top: x)
            try(x, y, x + h[y], top: y)  // middle holds k−i, h[i] = j−i
        let w = x - h[x]
        if w in at:
            try(w, x - h[w], x, top: x)
            if 2 * h[w] != h[x]: try(w, w + h[w], x, top: x)

    // Middle holds k−i, crossed; with j−i = k−j it is the case above.
    fn crossed(i: Int, j: Int, k: Int) = if i in at and h[i] != j - i: try(i, j, k, top: j)
    for (g, group) in at.group(by: x => h[x] - x):
        if group.len <= isqrt(h.len):
            for a in 0..group.len:
                for b in a + 1..group.len: crossed(group[a], group[b], group[a] + h[group[b]])
        else:
            for k in at if (k - g - h[k]) % 2 == 0:
                crossed((k - g - h[k]) / 2, (k - g + h[k]) / 2, k)
    found

// Part 2. Take an even point set S and, for each pair x < y in S, put a peak at
// the midpoint (x + y) / 2 with height (y − x) / 2. Then any x < y < z in S gives
// peaks at (x+y)/2 < (x+z)/2 < (y+z)/2 with heights (y−x)/2, (z−x)/2, (z−y)/2,
// which are exactly their pairwise distances: a mythical triple. Midpoints of
// different pairs can collide (the smaller height wins), so try 8e5 / M random S of
// about √(6M) points, C(|S|, 3) ≈ M^1.5 triples, and keep the best range. For M = 500
// no early seed reaches K = 2000, so take one found by trying 2e6 (as the reference does).
fn construct_range(m: 3..=2e5, _k: Int) -> List<Int>:
    fn attempt(seed: Int) -> List<Int>:
        var rng = Rng.new(seed)
        let size = isqrt(6 * m)
        let evens = (-m / 2..3 * m / 2).filter(x => x % 2 == 0)
        let s = rng.sample(rng.int(2 * size / 3, 4 * size / 3), from: evens).sorted()
        var h: [0..m]Int = m  // m: no pair has put a peak here
        for (x, y) in s.pairs() if (x + y) / 2 in 0..m and (y - x) / 2 in 1..m:
            h[(x + y) / 2] min= (y - x) / 2
        for i in 0..m if h[i] == m: h[i] = rng.int(1, 2)
        return List(h)
    if m == 500: return attempt(1606132)
    var (best, most) = (List<Int>(), -1)
    for seed in 0..8e5 / m:
        let h = attempt(seed)
        let count = count_triples(h)
        if count > most: (best, most) = (h, count)
    return best

let part: 1..=2 = read()
if part == 1:
    let n: 3..=2e5 = read()
    let h: [n]1..n = read()
    print(count_triples(h.map(x => Int(x))))
else:
    let (m, k) = read<(3..=2e5, 1..=12_000_000)>()
    let h = construct_range(m, k)
    print(h.len)
    print(h)
