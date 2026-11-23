// Part 1. Name a triple's positions i < j < k; its distances are j−i, k−j and the
// largest, k−i. Whichever position holds height k−i, two heights pin the third
// position, so every triple is reached from O(1) candidates per position, except
// "middle holds k−i, crossed" (h[i] = k−j, h[k] = j−i): there i and j share
// h[x] − x, and a √n split between small groups (all pairs) and large groups
// (solve for i, j from each k) bounds it. One predicate checks, `distinct` dedupes.
use std.rand.{Rng}

@grader
fn count_triples(h: [I]1..I.count) -> Int:
    fn ok(x: Int, y: Int, z: Int) -> ?(I, I, I):
        let (i, j, k) = (x, y, z).sorted()
        if i < j < k and [i, j, k].all(x => x in I) and sorted([h[i], h[j], h[k]]) == sorted([j - i, k - j, k - i]): (i, j, k)

    var found: Set<(I, I, I)> = []
    for x in I:
        for s in [-1, 1]:                              // x is an end holding k−i
            let y = x + s * h[x]
            if y in I:
                for m in [x + s * h[y], y - s * h[y]]: found.extend(ok(x, m, y))
        let m = x + h[x]                               // middle holds k−i, h[i] = j−i
        if m in I: found.extend(ok(x, m, x + h[m]))

    for group in I.group_by(x => h[x] - x):            // middle holds k−i, crossed
        if group.len <= isqrt(I.count):
            for (i, j) in group.pairs(): found.extend(ok(i, j, i + h[j]))
        else:
            let g = h[group.first] - group.first
            for k in I if (k - g - h[k]) % 2 == 0:
                found.extend(ok((k - g - h[k]) / 2, (k - g + h[k]) / 2, k))
    return found.len

// Part 2. Take an even point set S and, for each pair x < y in S, put a peak at
// the midpoint (x + y) / 2 with height (y − x) / 2. Then any x < y < z in S gives
// peaks at (x+y)/2 < (x+z)/2 < (y+z)/2 with heights (y−x)/2, (z−x)/2, (z−y)/2,
// which are exactly their pairwise distances: a mythical triple. Midpoints of
// different pairs can collide (the smaller height wins), so try many random S of
// about √(6M) points, C(|S|, 3) ≈ M^1.5 triples, and keep the best range.
@grader
fn construct_range(m: 3..=2e5, _k: Int) -> List<Int>:
    fn attempt(seed: Int) -> List<Int>:
        var rng = Rng(seed)
        let size = isqrt(6 * m)
        let evens = (-m / 2 .. 3 * m / 2).filter(x => x % 2 == 0)
        let s = rng.sample(rng.int(2 * size / 3, 4 * size / 3), from: evens).sorted()
        var h: [0..m]?Int = None
        for (x, y) in s.pairs() if (x + y) / 2 in 0..m and (y - x) / 2 in 1..m:
            h[(x + y) / 2] min= (y - x) / 2              // min= skips none
        h.map(v => v ?? rng.int(1, 2))
    (0..1e6 / m).map(attempt).max(by: count_triples)!
