// Source: DMOPC '21 Contest 4 P4 (Christmas Graph), https://dmoj.ca/problem/dmopc21c4p4
// Problem: each of N nodes (2 <= N <= 1e6) joins an edge to another node chosen uniformly at
//   random (multi-edges allowed). Print the expected number of connected components modulo
//   1e9 + 7.
// Solution: each node's choice is a functional graph whose components each hold exactly one
//   cycle (a 2-cycle is a double edge), so the answer is the expected number of cycles. For
//   length i there are C(N, i) * (i - 1)! candidate cycles, each present with probability
//   (N - 1)^-i. Sum over i = 2..=N with factorial tables and one modular inverse; O(N).
// original: corpus/submissions/4131060.cpp

const MOD = 1_000_000_007
type Z = Mod<MOD>

let n: Int = read()
let comb = Comb<Mod<MOD>>(n)
let step = Z(n - 1).inv()

var ans: Z = 0
var scale = step
for i in 2..=n:
    scale *= step
    ans += comb.choose(n, i) * comb.fact(i - 1) * scale
print(ans.v)
