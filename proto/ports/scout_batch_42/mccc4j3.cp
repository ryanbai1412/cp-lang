// Mock CCC '22 Contest 2 J3 — Prefix-Sum Equilibrium Split (CP Lang v2 spec)
// original: corpus/submissions/4264907.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Find the first 1-based prefix index i whose prefix sum equals half the array total,
// or print "Andrew is sad." when no split exists.

let n: 1..=1e6 = read()
domain V = 1..=n
let a: [V]Int = read()

let pre = prefix(a)
// FRICTION MC10: `Range<I>` lists `..i`, `..=i`, and `i..`, but omits the unbounded
// full range `..` (`pre.sum(..)` / `st.fold(..)`), forcing `V.first..=V.last`.
let total = pre.sum(V.all)
let ans = first(i for i in V if 2 * pre.sum(..=i) == total)
print(ans, none: "Andrew is sad.")
