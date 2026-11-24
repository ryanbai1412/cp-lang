// Source: APIO 2007 P2 (Backup), https://dmoj.ca/problem/apio07p2
// limits: 1000 ms, 32 MB
// Problem: n <= 1e5 offices at sorted positions <= 1e9 on a street; join k <= n / 2 disjoint
//   pairs by cables. Print the least total cable length.
// Solution: an optimal pairing cables k gaps between neighbours, no two adjacent, so choose k
//   such gaps of least sum. That cost is convex in k, so `aliens` finds it from the penalized
//   problem: with a price λ on each cable, a linear DP over the gaps (take a gap after skipping
//   the one before it) gives the least priced cost and its fewest cables. O(n log C).
// original: corpus/submissions/5532752.cpp
// Bounds: n: 2..=1e5; k: 1..=n / 2
let n: 2..=1e5 = read()
let k: 1..=n / 2 = read()
let at: [n]0..=1e9 = read()
let gaps = [b - a for (a, b) in at.adjacent]

// the least cost of cables priced at λ each, as (cost, cables), fewest cables on ties
fn priced(lam: Int) -> (val: Int, count: Int):
    var before = (0, 0)  // over the gaps up to the one before last
    var best = (0, 0)
    for g in gaps:
        let take = (before.0 + g + lam, before.1 + 1)
        before = best
        best = min(best, take)
    (val: best.0, count: best.1)

print(aliens(k, priced, lambdas: -2_000_000_001..=0))
