// cpl 0.3.0
// Problem: 0/1 knapsack (AtCoder DP D).
// Input: n cap, then n lines "w v". Output: the maximum total value with
// total weight <= cap.
// Guesses: H-ANON-RANGE-ARRAY (`[0..=cap]Int` with runtime cap, filled by
// a scalar), H-RANGE-REV (`(lo..=hi).rev` on an Int range).

let n = read<1..=100>()
let cap = read<1..=1e5>()
let items = read<[n](w: 1..=1e5, v: 1..=1e9)>()
var dp: [0..=cap]Int = 0
for it in items:
    for c in (it.w..=cap).rev:
        dp[c] max= dp[c - it.w] + it.v
print(dp[cap])
