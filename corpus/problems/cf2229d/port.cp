// Codeforces 2229 D — 2xN Threshold Binary Search
//
// `a0` and `a1` hold a `2 x n` grid of values in `1..=2 * n`. For a threshold
// `v`, map each column `i` to its net sign `s = Int(a0[i] >= v) + Int(a1[i] >= v) - 1`
// in `{-1, 0, +1}` and drop zeros. Consecutive `-1` columns merge into a single
// `-1` run, so `v` is achievable iff `+1` columns outnumber `-1` runs.
// Raising `v` only decreases column signs (shrinking `+1`s and creating/widening
// `-1` runs), so `is_good(v)` is monotone non-increasing on `1..=2 * n`.

@!multitest
let n: 1..=200_000 = read()
let a0: [0..n]1..=2 * n = read()
let a1: [0..n]1..=2 * n = read()

fn is_good(v: Int) -> Bool:
    let signs = [Int(x >= v) + Int(y >= v) - 1 for (x, y) in zip(a0, a1)]
    let nz = [s for s in signs if s != 0]
    let neg_runs = nz.items.count(((i, s)) => s == -1 and (i == 0 or nz[i - 1] != -1))
    nz.count(1) > neg_runs

print((1..=2 * n).last_true(is_good)!)
