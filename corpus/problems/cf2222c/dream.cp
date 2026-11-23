// Codeforces 2222 C — Odd-Length Equal-Median Partition
//
// Every odd-length segment has strictly more than half of its elements `<= m`
// and strictly more than half `>= m`, where `m` is its median. If all segments
// share the same median `m`, summing those inequalities over all segments
// forces strictly more than half of the whole array to be `<= m` and `>= m`, so
// `m` must equal the global median `med = a.sorted()[n / 2]`.
//
// Because `n` is odd, the single-segment partition `[0..n)` always has median
// `med`, guaranteeing `dp[n]!` is present (`>= 1`). Complexity: O(n^2) (`n <= 5_000`).

@!multitest
let n: 1..=5_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let med = a.sorted()[n / 2]
let pref_lt = prefix([Int(x < med) for x in a])
let pref_le = prefix([Int(x <= med) for x in a])

var dp: [0..=n]?Int = None
dp[0] = 0
for i in 0..n if let cur = dp[i]:
    for j in i + 1..=n by 2:
        if pref_lt.sum(i..j) <= (j - i) / 2 < pref_le.sum(i..j):
            dp[j] max= cur + 1
print(dp[n]!)
