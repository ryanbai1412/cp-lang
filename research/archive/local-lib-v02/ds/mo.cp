//! Mo's algorithm: answer offline range queries by moving one window between them,
//! in O((n + q) sqrt(n)) single-step moves.

/// The order in which to visit `queries` (half-open `(lo, hi)` ranges inside `0..n`) so
/// that walking a window from each to the next moves its two ends
/// O((n + q) sqrt(n)) steps in total. Returns indices into `queries`.
pub fn mo_order(n: Int, queries: List<(Int, Int)>) -> List<Int>:
    var block = 1
    while block * block * max(queries.len, 1) < 3 * n * n: block += 1
    var order = [i for i in 0..queries.len]
    // even blocks sweep the right end up, odd blocks sweep it back down
    order.sort(by: i => (
            queries[i].0 / block,
            if (queries[i].0 / block) % 2 == 0: queries[i].1 else: -queries[i].1,
        ))
    order

/// The window walk itself, for a window described by a state you fold yourself:
/// `steps(from, to)` lists the single moves that turn window `from` into window `to`, as
/// `(index, entering)` pairs: the position that enters (`true`) or leaves (`false`).
/// Grows before it shrinks, so the window is never negative on the way.
pub fn steps(from: (Int, Int), to: (Int, Int)) -> List<(Int, Bool)>:
    var out: List<(Int, Bool)> = []
    var (l, r) = from
    while l > to.0:
        l -= 1
        out.push((l, true))
    while r < to.1:
        out.push((r, true))
        r += 1
    while l < to.0:
        out.push((l, false))
        l += 1
    while r > to.1:
        r -= 1
        out.push((r, false))
    out

@test
fn distinct_values_in_ranges():
    for _ in 0..15:
        let n = rand(1, 30)
        let a = [rand(0, 5) for _ in 0..n]
        var queries: List<(Int, Int)> = []
        for _ in 0..rand(1, 30):
            let (x, y) = (rand(0, n), rand(0, n))
            queries.push((min(x, y), max(x, y)))
        var ans = List.filled(queries.len, 0)
        var cnt = List.filled(6, 0)
        var distinct = 0
        var win = (0, 0)
        for qi in mo_order(n, queries):
            for (i, entering) in steps(win, queries[qi]):
                if entering:
                    if cnt[a[i]] == 0: distinct += 1
                    cnt[a[i]] += 1
                else:
                    cnt[a[i]] -= 1
                    if cnt[a[i]] == 0: distinct -= 1
            win = queries[qi]
            ans[qi] = distinct
        for qi in 0..queries.len:
            let (lo, hi) = queries[qi]
            assert ans[qi] == count(any(a[i] == v for i in lo..hi) for v in 0..6)

@test
fn the_order_visits_every_query_once_and_stays_cheap():
    let n = 400
    let queries = [(min(x, y), max(x, y)) for (x, y) in [(rand(0, n), rand(0, n)) for _ in 0..400]]
    let order = mo_order(n, queries)
    assert order.sorted() == [i for i in 0..400]
    var moves = 0
    var win = (0, 0)
    for qi in order:
        moves += abs(queries[qi].0 - win.0) + abs(queries[qi].1 - win.1)
        win = queries[qi]
    assert moves < 400 * 400 / 4  // visiting in input order costs about n * q / 1.5
