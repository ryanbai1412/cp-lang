// Source: Codeforces 2138B (Antiamuny Wants to Learn Swap),
//   https://codeforces.com/contest/2138/problem/B
// limits: 2000 ms, 512 MB
// Problem: a permutation a of n <= 5e5 and q <= 5e5 queries l r (multitest, sums <= 5e5). A
//   subarray is perfect when one allowed swap of b_i and b_{i+2} never sorts it in fewer moves
//   than adjacent swaps alone; print YES or NO for each a[l..=r].
// Solution: a subarray is perfect iff it has no decreasing triple x < y < z (checked against a
//   BFS over the statement's moves on small subarrays). For each middle i the tightest triple
//   takes the nearest greater element to its left and the nearest smaller to its right (std
//   prev_smaller / next_smaller); bad[l] keeps the least such right end, and suffix minima make
//   min_bad[l] the first right end that fails a query starting at l. O(n + q).
// Bounds: n, q: 1..=500_000; a: 1..=n

use std.seq.{next_smaller, prev_smaller}

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]1..=n = read()

let prev_gt = a.prev_smaller(by: x => desc(x), strict: false)
let next_lt = a.next_smaller(strict: false)
var min_bad: [I]?I = None
for i in I:
    if let l = prev_gt[i] and let r = next_lt[i]:
        min_bad[l] min= r
for i in I.rev:
    if let p = i.prev and let r = min_bad[i]:
        min_bad[p] min= r

for _ in 0..q:
    let (l, r): (I, I) = read()
    print(if let b = min_bad[l] and r >= b: "NO" else: "YES")
