// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states.
//
// Hook-length DP derivation: a valid sequence of `n` insertions forms a rooted
// binary tree whose linear-extension count is `n! / prod_v size(v)`. Rather
// than tracking binomial interleaving coefficients across subintervals, `dp(i, s)`
// accumulates `1 / prod size(v)` over all valid subtrees on `[i, s.owner)`.
// Splitting `[i, k.owner)` at `j` combines `[i, j]` (completed at `runs[j].last`)
// with `[j + 1, k.owner)` (at state `k` with `next(k) == some(s)`); the newly
// formed subtree has size `k.owner - i`, contributing the factor
// `inv(k.owner - i)`. Multiplying `dp(Pos.first, runs[Pos.last].last)` by `n!`
// cancels the denominator and recovers the exact number of valid permutations.
//
// State runs: each position `i` owns a run of states — a fixed count `c` gives
// a chain of `c + 1` states stepping `0 -> 1 -> ... -> c`, while an unknown
// count (and the padding position `Pos.last`) gives one self-looping state.
type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: 1..=500 = read()
domain Pos = 0..=n
let given: [0..n]?Int = read(none: -1)
let a: [Pos]?Int = [Pos] i => if i.ord < n: given[i.ord] else: None

let ends = prefix([Pos] i => (a[i] ?? 0) + 1)
fn first(i: Pos) -> Int = ends.prefix(i.l)
fn last(i: Pos) -> Int = ends.prefix(i.r) - 1
let owner = [i for i in Pos for _ in first(i)..=last(i)]
fn next(s: Int) -> ?Int:
    guard let c = a[owner[s]] else: return Some(s)
    if s - first(owner[s]) < c: Some(s + 1) else: None

print(
    if Pos.any(i => ends.fold(..=i) - (i.ord + 1) > i.ord): 0
    else: dp(Pos.first!, last(Pos.last!)) * comb.fact(n)
)

@memo
fn dp(i: Pos, s: Int) -> Z:
    if s == first(i): return 1
    sum(
        dp(i, last(j)) * dp(j.next!, k) * comb.inv(owner[k].ord - i.ord)
        for k in [s - 1, s] for j in i..owner[k] if next(k) == Some(s)
    )
