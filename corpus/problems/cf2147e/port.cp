// Codeforces 2147 E — Maximum OR Popcount
//
// Each query gives a budget `budget` of +1 increments across `a[0..n]` and
// asks for the maximum popcount of the bitwise OR of `a`.
//
// Raising the OR's popcount by 1 is always cheapest by filling the lowest
// unset bit `b` (and any still-unset lower bits `x` from `b` down to `0`).
// Setting bit `x` on the element maximizing `a[idx] & ((1 << x) - 1)` minimizes
// the carry cost `(1 << x) - (a[idx] & mask)` and preserves all higher bits of
// the OR. Recording the monotone `(costs, pops)` frontier answers each query
// by binary search in O(log 31).

use std.seq.{argmax}

@!multitest
let (n, q): (1..=100_000, 1..=100_000) = read()
var a: [0..n]Int = read()

var v = fold(|, a)
var cur_cost = 0
var costs = [0]
var pops = [popcount(v)]

for b in 0..31 if not v.bit(b):
    for x in (0..=b).rev if not v.bit(x):
        let mask = (1 << x) - 1
        let idx = a.argmax(by: y => y & mask)!
        let cheapest = (1 << x) - (a[idx] & mask)
        a[idx] += cheapest
        cur_cost += cheapest
        v = fold(|, a)
    costs.push(cur_cost)
    pops.push(popcount(v))

let queries: [0..q]0..=10 ** 9 = read()
for budget in queries:
    let pos = costs.first_ge(budget + 1) ?? costs.len
    print(pops[pos - 1])
