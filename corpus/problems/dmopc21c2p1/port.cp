// Source: DMOPC '21 Contest 2 P1 (Bosses), https://dmoj.ca/problem/dmopc21c2p1
// limits: 2000 ms, 256 MB
// Problem: N levels in order with boss powers A_i; a level needs power >= A_i. H seconds buy 1
//   permanent power, P seconds 1 power for the next fight only. Print the least total time to
//   clear every level. N, H, P, A_i <= 1e6.
// Solution: permanent power is best bought up front, so pick a level L and pay H L +
//   P sum max(0, A_i - L). That is piecewise linear in L, so L is 0 or
//   some A_i: sort the powers descending, keep the count and sum of those above, and try each. O(N
//   log N).
// original: corpus/submissions/3946478.cpp
let n: 1..=1e6 = read()
let h: 1..=1e6 = read()
let p: 1..=1e6 = read()
var boss: [0..n]0..=1e6 = read()

// Buying L up front costs h L + p sum(max(0, A_i - L)), linear between powers, so L is 0 or
// some A_i. From the strongest boss down, `above` holds the bosses stronger than A_i.
var best = p * boss.sum()
var above = 0
var above_sum = 0
boss.sort()
for a in boss.rev:
    best min= h * a + p * (above_sum - above * a)
    above += 1
    above_sum += a
print(best)
