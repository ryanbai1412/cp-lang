// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi22p1 (Fish / Catching Fish)
// Original: corpus/submissions/4785724.cpp
// Exercises: Bool Index, Tropical Optionals (max=, max(gen)), for mut, .last_le(by:), @grader fn

type Fish = (y: Int, w: Int)

@grader
fn max_weights(n: Int, m: Int, xs: List<Int>, ys: List<Int>, ws: List<Int>) -> Int:
    // BLIND-GUESS(H9): can `domain` be declared locally inside a fn body?
    domain Col = 0..n
    var cols: [Col]List<Fish> = []
    for i in 0..m:
        let c = Col.from_idx(xs[i])!
        cols[c].push((y: ys[i], w: ws[i]))

    // BLIND-GUESS(H8): does `for mut` work over `[Col]List<Fish>` and nested `List<Fish>` records?
    for mut col in cols:
        col.sort(by: f => f.y)
        var run = 0
        for mut f in col:
            run += f.w
            f.w = run

    // Sum of fish weights in column `ci` with y < limit
    fn psum(ci: Int, limit: Int) -> Int:
        if let c = Col.from_idx(ci):
            // BLIND-GUESS(H6): does .last_le(key, by:) compare key: Int against f.y, and return index ?Int or element ?Fish?
            if let k = cols[c].last_le(limit - 1, by: f => f.y):
                return cols[c][k].w
        return 0

    fn height(ci: Int, j: Int) -> Int:
        if let c = Col.from_idx(ci):
            if j < cols[c].len:
                return cols[c][j].y
            else:
                return n
        return 0

    // BLIND-GUESS(H5): mixing structural range `0..=n` with `Bool` in 2D array index!
    // false = descending/flat pier profile (can catch in col i+1); true = ascending/can also catch in col i
    var dp: [0..=n, Bool]List<Int> = []
    dp[n, false] = [0]
    dp[n, true] = [0]

    for i in (0..n).rev:
        let ci = Col.from_idx(i)!
        let cnt = cols[ci].len
        let nxt_cnt = dp[i + 1, false].len

        // Tropical Optionals: max(gen) returns ?Int; BLIND-GUESS(H7): does max(Int, ?Int) return Int or ?Int?
        let best_nxt_true = dp[i + 1, true].max() ?? 0
        let best_nxt_false = dp[i + 1, false].max() ?? 0

        // State false (0): h[i] >= h[i+1] or reset
        dp[i, false] = List.filled(cnt + 1, max(best_nxt_true, best_nxt_false))
        var best_fall: ?Int = 0
        for k in 0..nxt_cnt:
            best_fall max= dp[i + 1, false][k] - psum(i + 1, height(i + 1, k))
        for j in 0..=cnt:
            dp[i, false][j] max= best_fall + psum(i + 1, height(i, j))

        // State true (1): can catch in col i using taller h[i+1], or fall into col i+1
        dp[i, true] = List.filled(cnt + 1, best_nxt_true)
        var best_rise: ?Int = 0
        for k in 0..nxt_cnt:
            best_rise max= dp[i + 1, true][k] + psum(i, height(i + 1, k))
        for j in 0..=cnt:
            dp[i, true][j] max= best_rise - psum(i, height(i, j))
        for j in 0..=cnt:
            dp[i, true][j] max= best_fall + psum(i + 1, height(i, j))

    return dp[0, true].max() ?? 0
