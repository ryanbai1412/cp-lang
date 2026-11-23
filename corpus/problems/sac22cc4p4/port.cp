// Source: SAC '22 Code Challenge 4 P4, https://dmoj.ca/problem/sac22cc4p4
// Problem: n <= 1e6 integers |a[i]| <= 1e9 and 0 <= B <= T <= 2e9. Print how many subarrays [l, r]
//   have B <= max - min <= T.
// Solution: count(t), the subarrays with max - min <= t, is a two-pointer scan: for each right
//   end, the smallest left end that keeps max - min <= t, with the max and the min of the window
//   in two monotone deques (index arrays). The answer is count(T) - count(B - 1), O(n).
// original: corpus/submissions/4411200.cpp
// note: a runnable subset, n <= 2e5 and B, T <= 1e9, below the statement's 1e6 and 2e9.

let n: 1..=200_000 = read()
let ql: 0..=1_000_000_000 = read()
let qr: 0..=1_000_000_000 = read()
let a: [0..n]Int = read()

fn count_at_most(limit: Int) -> Int:
    if limit < 0:
        return 0
    var mn_idx: [0..n]Int = 0
    var mx_idx: [0..n]Int = 0
    var mn_l = 0
    var mn_r = 0
    var mx_l = 0
    var mx_r = 0
    var l = 0
    var ans = 0
    for r in 0..n:
        while mn_r > mn_l and a[mn_idx[mn_r - 1]] >= a[r]:
            mn_r -= 1
        mn_idx[mn_r] = r
        mn_r += 1
        while mx_r > mx_l and a[mx_idx[mx_r - 1]] <= a[r]:
            mx_r -= 1
        mx_idx[mx_r] = r
        mx_r += 1
        while a[mx_idx[mx_l]] - a[mn_idx[mn_l]] > limit:
            l += 1
            while mn_l < mn_r and mn_idx[mn_l] < l:
                mn_l += 1
            while mx_l < mx_r and mx_idx[mx_l] < l:
                mx_l += 1
        ans += r - l + 1
    ans

print(count_at_most(qr) - count_at_most(ql - 1))
