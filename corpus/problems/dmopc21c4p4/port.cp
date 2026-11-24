// Source: DMOPC '21 Contest 4 P4 (Christmas Graph), https://dmoj.ca/problem/dmopc21c4p4
// limits: 2000 ms, 256 MB
// Problem: each of N nodes (2 <= N <= 1e6) joins an edge to another node chosen uniformly at
//   random (multi-edges allowed). Print the expected number of connected components modulo
//   1e9 + 7.
// Solution: each node's choice is a functional graph whose components each hold exactly one
//   cycle (a 2-cycle is a double edge), so the answer is the expected number of cycles. For
//   length i there are C(N, i) * (i - 1)! candidate cycles, each present with probability
//   (N - 1)^-i. Sum over i = 2..=N with factorial tables and one modular inverse; O(N).
// original: corpus/submissions/4131060.cpp
// Bounds: n: 2..=1e6

const MOD = 1_000_000_007
type Z = Mod<MOD>

let n: 2..=1e6 = read()
let comb = Comb<Z>(n)
let pick = Z(n - 1).inv()  // the chance a node picks one given other node
print(sum(comb.choose(n, len) * comb.fact(len - 1) * pick ** len for len in 2..=n))
