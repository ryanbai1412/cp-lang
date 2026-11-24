// Source: Codeforces 2229C2 (We Be Flipping, hard), https://codeforces.com/contest/2229/problem/C2
// Problem: n <= 2e5 nonzero integers |a_i| <= 1e9 (sum of n <= 2e5, multitest). An operation picks
//   i with a_i > 0 and negates a_1..a_i. Print at most n operations that maximise the final sum.
// Solution: the largest operated index p starts positive and ends negative, nothing right of it
//   changes, and every element left of it can be made positive. So pick the positive pivot p
//   maximising (sum of 2|a_j| over negative a_j, j < p) - 2 a_p, if that gain is positive. Sweep
//   i from p - 1 down, operating on i whenever a_i's current sign is positive, so every earlier
//   element is negative before the final operation on p flips them all. O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
var best_delta = 0
var best_idx: ?Int = None
var cur_delta = 0
for (i, v) in a.items:
    if v >= 0:
        if cur_delta - 2 * v > best_delta:
            best_delta = cur_delta - 2 * v
            best_idx = i
    else:
        cur_delta -= 2 * v

var ops: List<Int> = []
if let idx = best_idx:
    var flip = false
    for i in (0..idx).rev if (a[i] > 0) != flip:
        ops.push(i + 1)
        flip = not flip
    ops.push(idx + 1)
print(ops.len)
print(ops)
