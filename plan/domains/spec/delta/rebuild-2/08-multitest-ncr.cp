// t <= 1e5 test cases, each "n k" with 0 <= n, k <= 1e6: print C(n, k) mod 1e9 + 7.
// The factorial tables are shared by every case, so Comb is built above the divider.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>(1e6)

@!multitest
let (n, k) = read<(0..=1e6, 0..=1e6)>()
print(comb.choose(n, k))
