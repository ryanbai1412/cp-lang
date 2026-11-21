// Codeforces 2262 C: the sorted values `a` must interleave two arithmetic
// progressions with common difference `d`:
//   - `half = n / 2` terms `0, d, 2d, ..., (half - 1) * d`, and
//   - `n - half` terms `top, top - d, ..., top - (n - half - 1) * d`.
// The second-largest element `a[n - 2]` must be either `top - d` (giving
// `d = top - a[n - 2]`) or `(half - 1) * d` (giving `d = a[n - 2] / (half - 1)`),
// so only those two candidates for `d` need checking.
//
// Counting permutations: once `d` is fixed, the `half` terms of the ascending
// progression permute freely in `half!` ways, while the descending progression
// has its two boundary elements pinned, leaving `(n - half - 2)!` ways to order
// the remaining terms — giving `(n - half - 2)! * half!` per valid `d`.
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
