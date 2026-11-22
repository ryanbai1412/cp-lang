// SAC '22 Code Challenge 5 #4: n closed intervals of integers and q
// questions. A question l r asks whether every integer in l..=r lies in
// some interval: print Y or N.
//
// The original compresses every endpoint and its two neighbours, marks the
// covered points with a difference array and binary-searches the uncovered
// ones. Merging the intervals into maximal runs needs no compression: a
// question is a yes exactly when one run holds both of its ends.
let (n, q): (Int, Int) = read()
domain Seg = 0..n
let segs: [Seg](l: Int, r: Int) = read()

var runs: List<(l: Int, r: Int)> = []
for s in sorted(Seg, by: s => segs[s].l):
    let (l, r) = segs[s]
    if runs.len > 0 and l <= runs.last!.r + 1:
        runs.last!.r = max(runs.last!.r, r)
    else:
        runs.push((l:, r:))

for _ in 0..q:
    let (l, r): (Int, Int) = read()
    let lo = runs.first_gt(l, by: v => v.l) ?? runs.len  // runs[..lo] start at or before l, runs[hi..] after it
    print(if lo > 0 and r <= runs[lo - 1].r: "Y" else: "N")
