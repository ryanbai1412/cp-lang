// Source: Baltic OI 2008 Day 2 P3 (Gloves), https://dmoj.ca/problem/btoi08p6
// Problem: n <= 20 colours, a_i left and b_i right gloves of colour i (<= 1e8). Taken blind, how
//   many left L and right R gloves guarantee a same-colour pair? Print L and R minimising L + R.
// Solution: (L, R) fails iff for some colour set S, L <= sum of a over S and R <= sum of b off S.
//   So (L, R) must lie outside all 2^n such boxes. Sorted by the a-sum, a candidate is L = box
//   i's a-sum + 1 and R = 1 + the largest b-sum among later boxes (a suffix max); take the
//   smallest sum with L, R within the drawers, sweeping the boxes from the largest a-sum down.
//   O(2^n n).
// original: corpus/submissions/5325518.cpp
// Bounds: n: 1..=20; a, b: 0..=1e8
let n: 1..=20 = read()
domain C = 0..n
let a: [C]0..=1e8 = read()
let b: [C]0..=1e8 = read()
let (lefts, rights) = (a.sum(), b.sum())

// colour set s fails every (L, R) with L <= its left sum and R <= the right sum off it
let boxes = [(left: s.items.map(c => a[c]).sum(), right: s.complement().items.map(c => b[c]).sum())
    for s in Subset<C>.full.subsets].sorted()

var best: ?(l: Int, r: Int) = None
var later: ?Int = None
for box in boxes.rev:
    if let right = later:
        let (l, r) = (box.left + 1, right + 1)
        if l <= lefts and r <= rights and (if let p = best: l + r <= p.l + p.r else: true):
            best = (l:, r:)
    later = max(later, Some(box.right))
print(best!.l)
print(best!.r)
