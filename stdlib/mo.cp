//! Mo's algorithm: offline ordering and window stepping over `Range<D>`.
use std.seq.{ends}

/// Offline traversal order of `queries` minimizing window endpoint moves to O((n + q) sqrt n).
/// Uses alternating right-endpoint direction within each sqrt-sized left block.
/// Returns query indices `0..queries.len` in traversal order. O(q log q).
pub fn mo_order<D>(queries: List<Range<D>>) -> List<Int>:
    let q = queries.len
    if q == 0: return []
    let b = max(1, D.K / max(1, isqrt(q)))
    var idx = [i for i in 0..q]
    fn key(i: Int) -> (Int, Int):
        let (l, r) = queries[i].ends
        let blk = l / b
        (blk, if blk % 2 == 0: r else: -r)
    sort(mut idx, by: key)
    idx

/// Positions entering (`true`) or leaving (`false`) as a window moves from `from` to `to`.
/// Always expands before shrinking so the active window is never inverted. O(|dl| + |dr|).
pub fn mo_steps<D>(from: Range<D>, to: Range<D>) -> List<(pos: D, entering: Bool)>:
    var (l, r) = from.ends
    let (nl, nr) = to.ends
    var out: List<(pos: D, entering: Bool)> = []
    while l > nl:
        l -= 1
        out.push((pos: D.nth(l), entering: true))
    while r < nr:
        out.push((pos: D.nth(r), entering: true))
        r += 1
    while l < nl:
        out.push((pos: D.nth(l), entering: false))
        l += 1
    while r > nr:
        r -= 1
        out.push((pos: D.nth(r), entering: false))
    out

@test
fn mo_order_and_steps_answer_distinct_counts():
    domain I = 1..=20
    let a = [I] _ => rand(0, 5)
    var queries: List<Range<I>> = []
    for _ in 0..40:
        let l = I.nth(rand(0, 19))
        let r = I.nth(rand(l.to_idx, 19))
        queries.push(l..=r)
    var freq = List.filled(6, 0)
    var distinct = 0
    var win = I.first.l..I.first.l
    var ans = List.filled(queries.len, 0)
    for qi in mo_order(queries):
        let target = queries[qi]
        for s in mo_steps(win, target):
            let v = a[s.pos]
            if s.entering:
                if freq[v] == 0: distinct += 1
                freq[v] += 1
            else:
                freq[v] -= 1
                if freq[v] == 0: distinct -= 1
        win = target
        ans[qi] = distinct
    for qi in 0..queries.len:
        var seen = List.filled(6, false)
        for i in queries[qi]: seen[a[i]] = true
        assert ans[qi] == count(seen[v] for v in 0..6)
