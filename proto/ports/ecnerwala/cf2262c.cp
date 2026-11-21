// Codeforces 2262 C: the sorted values must interleave two arithmetic
// progressions with common difference d = M - M2: 0, d, 2d, ... (n/2 terms)
// and M, M - d, M - 2d, ... (the rest).  Try both candidates for d.
const MOD = 1e9 + 7
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 4..=2e5 = read()
let a: [0..n]Int = read()
let (top, half) = (a[n - 1], n / 2)
let ds = [top - a[n - 2], a[n - 2] / (half - 1)].sorted().deduped()
let ok = count(
    ([d * i for i in 0..half] + [top - d * i for i in 0..n - half]).sorted() == a for d in ds
)
print(comb.fact(n - half - 2) * comb.fact(half) * ok)
