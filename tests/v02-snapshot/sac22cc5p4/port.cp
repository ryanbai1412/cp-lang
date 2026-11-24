// Source: SAC '22 Code Challenge 5 P4 (Querying Intervals), https://dmoj.ca/problem/sac22cc5p4
// Problem: N <= 1e5 integer intervals [l_i, r_i] and Q <= 1e5 queries [L, R], endpoints in
//   [-1e9, 1e9]; print Y if every integer of [L, R] lies in some interval, else N.
// Solution: sort the intervals by left end and merge overlapping or adjacent ones (l <= r + 1)
//   into maximal runs. A query is Y iff the last run starting at or before L reaches R, found
//   by binary search. O((N + Q) log N).
let (n, q): (Int, Int) = read()
domain Seg = 0..n
let segs: [Seg](l: Int, r: Int) = read()

var runs: List<(l: Int, r: Int)> = []
for s in sorted(Seg, by: s => segs[s].l):
    let (l, r) = segs[s]
    if runs.len > 0 and l <= runs.last!.r + 1:
        runs.last!.r max= r
    else:
        runs.push((l:, r:))

for _ in 0..q:
    let (l, r): (Int, Int) = read()
    let lo = runs.first_gt(l, by: v => v.l) ?? runs.len  // runs[..lo] start at or before l, runs[hi..] after it
    print(if lo > 0 and r <= runs[lo - 1].r: "Y" else: "N")
