// Source: Codeforces 2207E1 (N-MEX, constructive version),
//   https://codeforces.com/contest/2207/problem/E1
// limits: 2000 ms, 256 MB
// Problem: given a_1..a_n (n <= 2e5, a_i <= 1e9), build b_1..b_n with b_i <= 1e9 such that the
//   (n-i+1)-th smallest non-negative integer missing from b_1..b_i is a_i, or print NO.
//   Multitest, sum of n <= 2e5.
// Solution: b exists iff a is non-increasing and n - i <= a_i <= n. Then build b from the back:
//   with bound n before a, go from the last bound to the first, push onto a stack every value
//   not yet pushed up to the current bound, and pop the top as the next b (the first pop is
//   discarded). The bounds keep the stack non-empty. O(n).
// Bounds: n: 1..=200_000; a: 0..=10 ** 9

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()

if (0..n - 1).any(i => a[i] < a[i + 1]) or (0..n).any(i => a[i] < n - i - 1 or a[i] > n):
    print("NO")
    return

let bounds = [n] + a
var b: [0..n]Int = 0
var stk: List<Int> = []
var next_val = 0
for i in (0..=n).rev:
    let cur = bounds[i]
    while next_val <= cur:
        stk.push(next_val)
        next_val += 1
    let x = stk.pop()!
    if i < n: b[i] = x

print("YES")
print(b)
