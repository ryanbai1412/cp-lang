// Source: Codeforces 2229D (Me When Median Problem), https://codeforces.com/contest/2229/problem/D
// limits: 2000 ms, 256 MB
// Problem: arrays a, b of n <= 1e5 values in 1..=2n. n - 1 times pick adjacent columns i, i + 1,
//   sort a_i, a_(i+1), b_i, b_(i+1) as s1..s4 and merge them into one column (s2, s3). Print the
//   largest final min(a_1, b_1). Multitest, sum of n <= 1e5.
// Solution: binary search the threshold v (feasibility only falls as v grows). A column scores +1
//   if both values are >= v, -1 if both are below, 0 otherwise; drop the zeros and merge runs of
//   -1 into one. v is reachable iff the +1 columns outnumber the -1 runs. O(n log n).
// Bounds: n: 1..=1e5; a0, a1: 1..=2 * n

@!multitest
let n: 1..=1e5 = read()
let a0: [0..n]1..=2 * n = read()
let a1: [0..n]1..=2 * n = read()

fn is_good(v: Int) -> Bool:
    var (ones, neg_runs, last) = (0, 0, 0)  // last: the previous nonzero sign
    for (x, y) in zip(a0, a1):
        let s = Int(x >= v) + Int(y >= v) - 1
        if s == 1: ones += 1
        elif s == -1 and last != -1: neg_runs += 1
        if s != 0: last = s
    ones > neg_runs
print((1..=2 * n).last_true(is_good)!)
