// Codeforces 2159 D1 — Prefix Partition with Quotient Costs
//
// Partition `a[0..n]` into contiguous blocks where a block ending at `i` with
// minimum `v` costs `ceil(a[i] / v)`. Maintain a strictly increasing monotone
// stack `stk` of `(val: a[j], dist: dp[j])` over prefix minima:
// - Popping `top.val >= x` leaves the nearest smaller element `stk.last` (so
//   starting a cost-1 block at `i` gives `dist = 1 + stk.last.dist`, or `1`
//   if `stk` is empty).
// - A last block of cost `l >= 2` can start right after any `stk[lo]` with
//   `stk[lo].val < ceil_div(x, l)` (the skipped suffix has minimum
//   `>= ceil_div(x, l)`), giving `dist min= prev + l` where `prev = stk[lo].dist`
//   (or `0` when no such `lo` exists, meaning the whole prefix `0..=i` forms one
//   block).
// - Only `l in 2..=min(dist, 3)` needs checking: any block of cost `l >= 4`
//   can be split into two blocks of total cost `<= 2 + (l - 2) = l` since
//   `2 * (l - 2) >= l` for `l >= 4`.

@!multitest
let n: 1..=400_000 = read()
let a: [0..n]1..=10 ** 18 = read()

var stk: List<(val: Int, dist: Int)> = []
for x in a:
    while let top = stk.last and top.val >= x: stk.pop()
    var dist = if let top = stk.last: 1 + top.dist else: 1
    for l in 2..=min(dist, 3):
        let tgt = ceil_div(x, l)
        let prev = if let lo = (0..stk.len).last_true(md => stk[md].val < tgt):
            stk[lo].dist
        else:
            0
        dist min= prev + l
    stk.push((x, dist))

print(stk.last!.dist)
