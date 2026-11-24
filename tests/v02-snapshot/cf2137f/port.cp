// Source: Codeforces 2137F, Prefix Maximum Invariance,
//   https://codeforces.com/contest/2137/problem/F
// Problem: multitest, sum n <= 2e5, a_i, b_i in 1..=2n. f(x, y) is the most positions where z
//   can equal y, over z with the same prefix maxima as x. Print the sum of
//   f(a[l..=r], b[l..=r]) over all subarrays.
// Solution: in a subarray starting at L, position i matches iff a_i == b_i, or some a_j with
//   L <= j < i is >= max(a_i, b_i). A strictly decreasing monotone stack and a binary search
//   give the last such j; i adds (lo + 1) * (n - i) for its valid L and R. O(n log n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=2 * n = read()
let b: [0..n]1..=2 * n = read()

var stk: List<(val: Int, idx: Int)> = []
var ans = 0
for i in 0..n:
    let lo: ?Int = if a[i] == b[i]:
        i
    else:
        let need = max(a[i], b[i])
        if let j = (0..stk.len).last_true(m => stk[m].val >= need): stk[j].idx else: None
    if let l = lo: ans += (l + 1) * (n - i)
    while let top = stk.last and top.val <= a[i]: stk.pop()
    stk.push((a[i], i))

print(ans)
