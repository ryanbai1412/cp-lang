// Universal Cup 5 Stage 3 (Gorkovskaya) F — Subset XOR via Krawtchouk / WHT
//
// Input gives `m + 1` subset sizes `a[0..=m]` in `0..=n` (matching the
// statement's `m + 1` integers). We count tuples of subsets `S_0, ..., S_m` of
// `{1..n}` with `|S_j| = a_j` whose XOR (symmetric difference) is empty.
//
// By Walsh–Hadamard transform over `T subset {1..n}`:
//   ans = (1 / 2^n) * sum_{k=0}^n C(n, k) * prod_{j=0}^m K_{a_j}(k)
// where `k = |T|` and `kraw[i] = K_i(k) = [x^i] (1 - x)^k * (1 + x)^{n - k}`
// is the Krawtchouk polynomial (grouping equal sizes gives `kraw[i] ** freq[i]`).
// - At `k = 0`: `kraw[i] = C(n, i)`.
// - Stepping `k -> k + 1` multiplies `(1 - x)^k (1 + x)^{n - k}` by `(1 - x) / (1 + x)`:
//   dividing by `(1 + x)` is the forward pass `kraw[i + 1] -= kraw[i]`, and
//   multiplying by `(1 - x)` is the backward pass `kraw[i + 1] -= kraw[i]`.

const MOD = 998_244_353
type Z = Mod<MOD>

let (n, m): (1..=5_000, 0..=5_000) = read()
let a: [0..=m]0..=n = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1

let comb = Comb<Z>()
var kraw = [comb.choose(n, i) for i in 0..=n]
var ans: Z = 0
for k in 0..=n:
    if k > 0:
        for i in 0..n: kraw[i + 1] -= kraw[i]
        for i in (0..n).rev: kraw[i + 1] -= kraw[i]
    ans += fold(*, kraw[i] ** freq[i] for i in 0..=n if freq[i] > 0) * comb.choose(n, k)
print(ans / (Z(2) ** n))
