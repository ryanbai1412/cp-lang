// Source: Baltic OI 2008 Day 2 P3 (Gloves), https://dmoj.ca/problem/btoi08p6
// limits: 1000 ms, 32 MB
// Problem: n <= 20 colours, a_i left and b_i right gloves of colour i (<= 1e8). Taken blind, how
//   many left L and right R gloves guarantee a same-colour pair? Print L and R minimising L + R.
// Solution: (L, R) fails iff for some colour set S, L <= sum of a over S and R <= sum of b off S.
//   So (L, R) must lie outside all 2^n such boxes. Sorted by the a-sum, a candidate is L = box
//   i's a-sum + 1 and R = 1 + the largest b-sum among later boxes (a suffix max); take the
//   smallest sum with L, R within the drawers, sweeping the boxes from the largest a-sum down.
//   O(2^n n).
// original: corpus/submissions/5325518.cpp
// checker: corpus/problems/btoi08p6/checker.py
// Bounds: n: 1..=20; a, b: 0..=1e8
let n: 1..=20 = read()
domain C = 0..n
let a: [C]0..=1e8 = read()
let b: [C]0..=1e8 = read()
let (lefts, rights) = (a.sum(), b.sum())

// colour set s fails every (L, R) with L <= its left sum and R <= the right sum off it. A box
// packs the two sums, each under 2^32, into one Int ordered by the left sum: 2^20 of them sort
// in place within 32 MB.
let shift = 1 << 32
var boxes = List.filled(1 << n, rights)
for c in C:
    for s in 0..(1 << c.ord):
        boxes[s + (1 << c.ord)] = boxes[s] + a[c] * shift - b[c]
boxes.sort()

var best: ?(l: Int, r: Int) = None
var later: ?Int = None
for box in boxes.rev:
    if let right = later:
        let (l, r) = (box / shift + 1, right + 1)
        if l <= lefts and r <= rights and (if let p = best: l + r <= p.l + p.r else: true):
            best = (l:, r:)
    later = max(later, Some(box % shift))
print(best!.l)
print(best!.r)
