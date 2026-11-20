//! Knuth's optimization for interval DPs: optimal ways to split `0..n` at interior points
//! when the best split point is monotone in both ends. O(n^2) instead of O(n^3).

/// `dp[i][j]` for `0 <= i < j <= n`: the least total cost of cutting the interval of
/// boundaries `i..j` down to unit pieces, where cutting an interval `i..j` anywhere costs
/// `cost(i, j)` and unit pieces cost nothing. `cost` must satisfy the quadrangle
/// inequality and be monotone on inclusion (any cost that is a sum of non-negative
/// weights over the interval does). Returns the whole table; the answer is `dp[0][n]`.
pub fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>:
    var dp = [[0 for _ in 0..=n] for _ in 0..=n]
    // opt[i][j]: the best first cut of i..j; it lies between opt[i][j-1] and opt[i+1][j]
    var opt = [[0 for _ in 0..=n] for _ in 0..=n]
    for i in 0..n: opt[i][i + 1] = i
    for len in 2..=n:
        for i in 0..=n - len:
            let j = i + len
            let from = max(opt[i][j - 1], i + 1)
            let to = min(opt[i + 1][j], j - 1)
            var best: ?(Int, Int) = none
            for k in from..=max(from, to):
                let v = dp[i][k] + dp[k][j]
                if best == none or v < best!.0: best = (v, k)
            dp[i][j] = best!.0 + cost(i, j)
            opt[i][j] = best!.1
    dp

@test
fn optimal_merge_matches_the_cubic_dp():
    for _ in 0..30:
        let n = rand(1, 10)
        let a = [rand(0, 20) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre[pre.len - 1] + x)
        let cost = (i, j) => pre[j] - pre[i]
        let fast = knuth(n, cost)
        var slow = [[0 for _ in 0..=n] for _ in 0..=n]
        for len in 2..=n:
            for i in 0..=n - len:
                let j = i + len
                slow[i][j] = min(slow[i][k] + slow[k][j] for k in i + 1..j)! + cost(i, j)
        for i in 0..n:
            for j in i + 1..=n: assert fast[i][j] == slow[i][j]

@test
fn merging_stones():
    // merge adjacent piles 4, 1, 1, 4: pay the size of each merged pile
    let pre = [0, 4, 5, 6, 10]
    assert knuth(4, (i, j) => pre[j] - pre[i])[0][4] == 2 + 6 + 10 + 0
