// Source: SAC '22 Code Challenge 4 P4, https://dmoj.ca/problem/sac22cc4p4
// limits: 3000 ms, 1024 MB
// Problem: n <= 1e6 integers |a[i]| <= 1e9 and 0 <= B <= T <= 2e9. Print how many subarrays [l, r]
//   have B <= max - min <= T.
// Solution: count(t), the subarrays with max - min <= t, is a two-pointer scan: for each right
//   end, the smallest left end that keeps max - min <= t, with the max and the min of the window
//   in two sliding `Window`s. The answer is count(T) - count(B - 1) (0 when B = 0), O(n).
// original: corpus/submissions/4411200.cpp

let n: 1..=1e6 = read()
let lo_gap: 0..=2e9 = read()
let hi_gap: lo_gap..=2e9 = read()
let a: [n]-1e9..=1e9 = read()

fn within(limit: 0..=2e9) -> Int:
    var top = Window(max)
    var bottom = Window(min)
    var l = 0
    var count = 0
    for r in 0..n:
        top.push(a[r])
        bottom.push(a[r])
        while top.all! - bottom.all! > limit:
            top.pop(a[l])
            bottom.pop(a[l])
            l += 1
        count += r - l + 1
    count

print(within(hi_gap) - (if lo_gap == 0: 0 else: within(lo_gap - 1)))
