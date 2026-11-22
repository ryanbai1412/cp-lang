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
let a: [Pos]?Int = read<[0..n]?Int>(none: -1).padded(to: Pos, with: none)

let runs = Runs<Pos>(sizes: [i => (a[i] ?? 0) + 1])
domain State = runs.items
fn next(s: State) -> ?State:
    guard let c = a[s.owner] else: return some(s)
    if s.rank < c: some(s + 1) else: none

print(
    if a.map(c => c ?? 0).scan(+).items.any((i, t) => t > i): 0
    else: dp(Pos.first, runs[Pos.last].last) * comb.fact(n)
)

@memo
fn dp(i: Pos, s: State) -> Z:
    if s == runs[i].first: return 1
    sum(
        dp(i, runs[j].last) * dp(j.next!, k) * comb.inv(k.owner - i)
        for j in i..Pos.last for k in runs.items(from: j.next!) if next(k) == some(s)
    )
