// NOI '10 P2 - Super Piano (DMOJ: noi10p2, 25pt, TL 1.0s, ML 512MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Given a[1..=n] (|a_i| <= 1000) and bounds L <= R, choose k distinct
// subarrays [i, j] with L <= j - i + 1 <= R maximizing the sum of their sums.
//
// Build prefix sums pre[0..=n] and a SparseTable returning (val, pos) for RMQ
// over pre. For each start i, push the best right end in [i + L - 1, min(n, i + R - 1)]
// into a max-Heap. Pop the best candidate k times and split its right-end range
// around `best` into [lo, best - 1] and [best + 1, hi].
use std.sparse.{SparseTable}
use std.heap.{Heap}
use std.algebra.{Semigroup, Order}

let (n, k, lo_len, hi_len): (1..=5e5, 1..=5e5, 1..=5e5, 1..=5e5) = read()
    // WISH W10: non-typeable combinatorial upper bound on k:
    // k <= number of valid subarrays = sum_{len = lo_len..=hi_len} (n - len + 1)
    where lo_len <= hi_len and hi_len <= n
      and k <= (hi_len - lo_len + 1) * (2 * n - lo_len - hi_len + 2) / 2
domain I = 1..=n
let a: [I]-1000..=1000 = read()

fn solve() -> Int:
    var pre: [I.bounds]Int = 0
    for i in I: pre[i.r] = pre[i.l] + a[i]

    // WISH W11: `SparseTable<I.bounds, ...>` over a boundary domain + argmax RMQ helper
    let argmax_op = Semigroup<(val: Int, b: I.bounds)>((x, y) => if x.val >= y.val: x else: y)
    let rmq = SparseTable<I.bounds, (val: Int, b: I.bounds)>(
        argmax_op,
        [I.bounds] b => (val: pre[b], b: b)
    )

    struct Cand:
        score: Int
        start: I
        lo: I.bounds
        hi: I.bounds
        best: I.bounds

    fn make_cand(start: I, lo: I.bounds, hi: I.bounds) -> Cand:
        let mx = rmq.fold(lo..=hi)!
        Cand(mx.val - pre[start.l], start, lo, hi, mx.b)

    var pq = Heap<Cand>(Order<Cand>.by(c => -c.score))
    for i in I:
        let lo_idx = i.to_idx + lo_len
        if lo_idx <= n:
            let hi_idx = min(n, i.to_idx + hi_len)
            pq.push(make_cand(i, I.bounds.nth(lo_idx), I.bounds.nth(hi_idx)))

    var total = 0
    for _ in 0..k:
        let c = pq.pop()!
        total += c.score
        if c.lo < c.best:
            pq.push(make_cand(c.start, c.lo, (c.best - 1)!))
        if c.best < c.hi:
            pq.push(make_cand(c.start, (c.best + 1)!, c.hi))
    total

@brute(solve)
fn solve_brute() -> Int
    where n <= 80:
    var pre: [I.bounds]Int = 0
    for i in I: pre[i.r] = pre[i.l] + a[i]
    var sums: List<Int> = []
    for i in I:
        for len in lo_len..=hi_len:
            let r_idx = i.to_idx + len
            if r_idx <= n:
                sums.push(pre[I.bounds.nth(r_idx)] - pre[i.l])
    sums.sort(by: x => -x)
    sum(sums[t] for t in 0..k)

print(solve())
