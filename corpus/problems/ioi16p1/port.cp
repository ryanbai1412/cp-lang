// Source: IOI 2016 P1 (Detecting Molecules), https://dmoj.ca/problem/ioi16p1
// Problem: n <= 2e5 weights w_i < 2^31 and a range [l, u] with u - l >= max w - min w. Find any
//   subset whose total lies in [l, u] and give its indices, or report none. Grader task, any
//   valid subset is accepted.
// Solution: sort by weight. If some k items work, so does a window of k consecutive sorted items:
//   moving the window by one changes its sum by at most max w - min w <= u - l, from the k
//   lightest to the k heaviest. Two pointers: grow the window, drop the lightest while the sum
//   exceeds u, stop at the first sum >= l. O(n log n).
// original: corpus/submissions/4686581.cpp (grader function find_subset)
// checker: corpus/problems/ioi16p1/checker.py
// stdin here: n l u, then the n weights; stdout: the subset's size, then its 0-based indices.
let n: 1..=200_000 = read()
let l: 1..=2_147_483_647 = read()
let u: l..=2_147_483_647 = read()
domain I = 0..n
let w: [I]1..=u = read()

// a window of the lightest-first order: drop its lightest while it weighs over u
let order = I.sort(by: i => (w[i], i))
var (lo, total) = (0, 0)
var picked: List<I> = []
for (hi, i) in order.items:
    total += w[i]
    while total > u:
        total -= w[order[lo]]
        lo += 1
    if total >= l:
        picked = order.take(hi + 1).drop(lo).collect()
        break
print(picked.len)
print(picked)
