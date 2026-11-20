// DP Contest Z (Frog 3): stones at increasing heights, a jump i -> j costs (h[j] - h[i])^2 + c.
// DRAFT 1: O(n^2) DP.
let n: 2..=2e5 = read()
let c: 1..=1e12 = read()
domain I = 1..=n
let h: [I]1..=1e6 = read()
assert all(h[i] < h[(i + 1)!] for i in I if i != I.last)

var dp: [I]Int = 0
for i in I:
    if i != I.first:
        dp[i] = min(dp[j] + (h[i] - h[j]) ** 2 + c for j in ..i)!
print(dp[I.last])
