// Source: Codeforces 2147E (Maximum OR Popcount), https://codeforces.com/contest/2147/problem/E
// limits: 2000 ms, 256 MB
// Problem: n <= 1e5 numbers a[i] <= 1e9 and q <= 1e5 independent queries b <= 1e9: using at
//   most b increments (+1 to any element), maximize the popcount of the OR of the array.
//   Multitest, sums of n and of q <= 1e5.
// Solution: more ones are cheapest as a full low block: for each unset bit b of the OR, set
//   every missing bit x from b down to 0 by raising the element with the largest
//   a & (2^x - 1) until bit x turns on (cost 2^x minus those low bits). Record the running
//   (cost, popcount) after each b, O(31^2 n) per test; a query binary-searches the costs.
// Bounds: n, q: 1..=100_000; queries: 0..=10 ** 9

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
