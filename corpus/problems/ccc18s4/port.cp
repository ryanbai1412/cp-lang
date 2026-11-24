// Source: CCC 2018 S4 (Balanced Trees), https://dmoj.ca/problem/ccc18s4
// limits: 1000 ms, 256 MB
// Problem: a perfectly balanced tree of weight 1 is one node; of weight w >= 2 it is a root with
//   k identical such subtrees (2 <= k <= w), each of weight floor(w / k). Given n <= 1e9, count
//   the perfectly balanced trees of weight n.
// Solution: trees(w) = sum over k = 2..=w of trees(w / k). With r = isqrt(w), the k <= w / r
//   each give their own quotient w / k >= r, and every k beyond has a quotient q < r, taken by the
//   w / q - w / (q + 1) such k. Memoised; only values n / d are reached, about O(n^(3/4)) total.
// original: corpus/submissions/4329681.cpp
// Bounds: n: 1..=1e9
@memo
fn trees(w: 1..=1e9) -> Int:
    if w == 1: return 1
    let r = w.isqrt
    sum(trees(w / k) for k in 2..=w / r) + sum((w / q - w / (q + 1)) * trees(q) for q in 1..r)
let n: 1..=1e9 = read()
print(trees(n))
