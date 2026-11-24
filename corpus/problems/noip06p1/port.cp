// Source: NOIP 2006 Senior P1 (Energy Necklace), Luogu P1063 (no longer on DMOJ)
// limits: 1000 ms, 128 MB
// Problem: N beads (4 <= N <= 100) in a ring; bead i has head mark[i] <= 1000 and tail equal to
//   the next bead's head. Merging adjacent beads (m, r) and (r, n) releases m * r * n and
//   leaves (m, n). Merge the ring down to one bead; print the largest total energy (<= 2.1e9).
// Solution: interval DP on the ring: best(i, len) is the most energy from merging the len beads
//   starting at bead i (indices wrap). The last merge joins the first k beads to the other
//   len - k and releases mark[i] * mark[i + k] * mark[i + len]. Memoised over n^2 states with
//   O(n) splits each, O(n^3); the answer is the max of best(i, n) over every start i.
// original: corpus/submissions/5165686.cpp
let n: 4..=100 = read()
domain Bead = 0..n
let mark: [Bead]1..=1000 = read()

// best(i, len): the most energy from merging the len beads from bead i on,
// going round the ring; the last merge joins the first k to the rest
@memo
fn best(i: Bead, len: 1..=n) -> Int =
    [best(i, k) + best(
            Bead.wrap(i.ord + k),
            len - k,
        ) + mark[i] * mark[Bead.wrap(i.ord + k)] * mark[Bead.wrap(i.ord + len)] for k in 1..len].max() ?? 0
print([best(i, n) for i in Bead].max()!)
