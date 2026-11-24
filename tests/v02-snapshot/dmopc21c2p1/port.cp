// Source: DMOPC '21 Contest 2 P1 (Bosses), https://dmoj.ca/problem/dmopc21c2p1
// Problem: N levels in order with boss powers A_i; a level needs power >= A_i. H seconds buy 1
//   permanent power, P seconds 1 power for the next fight only. Print the least total time to
//   clear every level. N, H, P, A_i <= 1e6.
// Solution: permanent power is best bought up front, so pick a level L and pay H L +
//   P sum max(0, A_i - L) (a = H, b = P in the code). That is piecewise linear in L, so L is 0 or
//   some A_i: sort the powers descending, keep the count and sum of those above, and try each. O(N
//   log N).
let (n, a, b): (1..=1e6, 1..=1e6, 1..=1e6) = read()
let heights: [0..n]0..=1e6 = read()

var best = b * heights.sum()  // level 0
var taller = 0                // posts above the current level
var taller_sum = 0
for h in heights.sorted().rev:
    best min= a * h + b * (taller_sum - taller * h)
    taller += 1
    taller_sum += h
print(best)
