//! Divide-and-conquer optimization: row minima of a matrix whose best column never moves
//! left as the row grows, in O((rows + cols) log rows) evaluations instead of rows * cols.

/// For every row `i` in `0..rows`, the smallest `f(i, j)` over the columns `j` in `0..cols`
/// where `f` is not `none`, with the leftmost column that attains it. `f` must be
/// monotone: the best column of row `i + 1` is not left of the best column of row `i`.
/// A row with no column at all gives `none`.
pub fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int) -> List<?(Int, Int)>:
    var out: List<?(Int, Int)> = List.filled(rows, none)
    // (row range, column range) still to solve; each row's answer splits the columns
    var todo = [(0, rows, 0, cols)]
    while not todo.is_empty:
        let (lo, hi, from, to) = todo.pop()!
        if lo >= hi: continue
        let mid = (lo + hi) / 2
        var best: ?(Int, Int) = none
        for j in from..to:
            let v = f(mid, j)
            if v != none and (best == none or v! < best!.0): best = (v!, j)
        out[mid] = best
        let cut = if best == none: from else: best!.1
        todo.push((lo, mid, from, if best == none: to else: cut + 1))
        todo.push((mid + 1, hi, cut, to))
    out

/// One layer of a partition DP: `cur[i] = min over j < i of prev[j] + cost(j, i)` for
/// `i` in `1..prev.len`, where `cost(j, i)` is the cost of one group covering `j..i`
/// and satisfies the quadrangle inequality. `cur[0]` is `none`, as are entries with no
/// reachable `j`. O(n log n) calls of `cost`.
pub fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>:
    let n = prev.len
    let rows = monotone_minima(
        n,
        n,
        (i, j) => if j < i and prev[j] != none: prev[j]! + cost(j, i) else: none,
    )
    [if r == none: none else: r!.0 for r in rows]

@test
fn layers_match_the_quadratic_dp():
    for _ in 0..25:
        let n = rand(1, 12)
        let a = [rand(0, 9) for _ in 0..n]
        var pre = [0]
        for x in a: pre.push(pre.last! + x)
        // cost of a group is the square of its sum: satisfies the quadrangle inequality
        let cost = (j, i) => (pre[i] - pre[j]) * (pre[i] - pre[j])
        var fast: List<?Int> = [if i == 0: 0 else: none for i in 0..=n]
        var slow: List<?Int> = [if i == 0: 0 else: none for i in 0..=n]
        for _ in 0..rand(1, 4):
            fast = dnc_layer(fast, cost)
            var next: List<?Int> = List.filled(n + 1, none)
            for i in 1..=n:
                for j in 0..i:
                    if slow[j] != none: next[i] min= slow[j]! + cost(j, i)
            slow = next
            assert fast == slow

@test
fn row_minima_of_a_monotone_matrix():
    // f(i, j) = (j - 2 i)^2: the best column is 2 i, clipped to the matrix
    let got = monotone_minima(5, 7, (i, j) => (j - 2 * i) * (j - 2 * i))
    assert got == [(0, 0), (0, 2), (0, 4), (0, 6), (4, 6)]
    assert monotone_minima(2, 3, (i, j) => none) == [none, none]
