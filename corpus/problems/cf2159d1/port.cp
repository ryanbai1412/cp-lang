// Source: Codeforces 2159D1 (Inverse Minimum Partition, Easy Version),
//   https://codeforces.com/contest/2159/problem/D1
// limits: 4000 ms, 1024 MB
// Problem: split a_1..a_n (n <= 4e5, a_i <= 1e18) into contiguous blocks, a block costing
//   ceil(last / min). Print the least total cost. Multitest, sum n <= 4e5.
// Solution: a block of cost l >= 4 splits into two of total cost <= l, so only costs 1..3
//   matter. Keep a stack of strictly increasing prefix values, each with its dp. Ending a
//   block at x: cost 1 starts after the nearest smaller value; cost l in 2..=3 starts after the
//   last stack entry below ceil(x / l), found by binary search (none: the whole prefix is one
//   block). O(n log n).

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
