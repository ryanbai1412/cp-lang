// Source: CCC 2018 S4 (Balanced Trees), https://dmoj.ca/problem/ccc18s4
// Problem: a perfectly balanced tree of weight 1 is one node; of weight w >= 2 it is a root with
//   k identical such subtrees (2 <= k <= w), each of weight floor(w / k). Given n <= 1e9, count
//   the perfectly balanced trees of weight n.
// Solution: trees(w) = sum over k = 2..=w of trees(w / k). w / k is constant on O(sqrt w)
//   blocks of k (`floor_quotients`), so add block size * trees(q) per block. Memoised; only
//   values n / d are reached, about O(n^(3/4)) total.
// original: corpus/submissions/4329681.cpp
@memo
fn trees(w: 1..=1e9) -> Int:
    if w == 1: return 1
    sum((b.hi - b.lo) * trees(b.q) for b in floor_quotients(w, from: 2))
let n: 1..=1e9 = read()
print(trees(n))
