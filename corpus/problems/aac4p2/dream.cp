// Source: An Animal Contest 4 P2 (Lavish Lights), https://dmoj.ca/problem/aac4p2
// Problem: n lights (n <= 2e5) with a_i <= 1e9; light i is on at second t iff a_i | t.
//   For each of q <= 1e6 queries t (0 <= t <= 1e9) print the first light that is off at second
//   t, or -1 if all are on.
// Solution: lights 1..i are all on iff lcm(a_1..a_i) | t, and a multiple of a longer
//   prefix's lcm is a multiple of every shorter one, so "some light in 1..i is off" is
//   monotone in i. Prefix lcms are capped at 2e9 (above every t, so a capped prefix never
//   divides t); `prefix_fold` builds them and each query runs `first_true`. O(n log + q log n).
use std.prefix.{prefix_fold}
const CAP = 2e9  // above every query, so a capped prefix never divides one
let (n, q): (1..=2e5, 1..=1e6) = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let queries: [q]0..=1e9 = read()

let on_until = prefix_fold(Monoid(1, (x, y) => min(CAP, lcm(x, y))), a)
for t in queries:
    print(I.first_true(i => t % on_until[i.r] != 0), none: -1)
