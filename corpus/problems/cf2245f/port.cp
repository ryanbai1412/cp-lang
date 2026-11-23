// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states.
//
// Hook-length DP derivation: a valid sequence of `n` insertions forms a rooted
// binary tree whose linear-extension count is `n! / prod_v size(v)`. Rather
// than tracking binomial interleaving coefficients across subintervals, `dp[i, s]`
// accumulates `1 / prod size(v)` over all valid subtrees on `[i, s.owner)`.
// Splitting `[i, k.owner)` at `j` combines `[i, j]` (completed at `last[j]`)
// with `[j + 1, k.owner)` (at state `k` with `next(k) == some(nk)`); the newly
// formed subtree has size `k.owner - i`, contributing the factor
// `inv(k.owner - i)`. Multiplying `dp[Pos.first, last[Pos.last]]` by `n!`
// cancels the denominator and recovers the exact number of valid permutations.
//
// State runs: each position `i` owns a run of states — a fixed count `c` gives
// a chain of `c + 1` states stepping `0 -> 1 -> ... -> c`, while an unknown
// count (and the padding position `Pos.last`) gives one self-looping state.
type Z = Mod<998_244_353>
let comb = Comb<Z>()

struct State(owner: Pos, next: ?Int)

@!multitest
let n: 1..=500 = read()
domain Pos = 0..=n
let given: [0..n]?Int = read(none: -1)
let a: [Pos]?Int = [Pos] i => if i.ord < n: given[i.ord] else: none

fn solve() -> Z:
    let ends = prefix([Pos] i => (a[i] ?? 0) + 1)
    for i in Pos:
        if ends.fold(..=i) - (i.ord + 1) > i.ord: return 0

    fn start(i: Pos) -> Int = ends.prefix(i.l)
    let last = [Pos] i => ends.prefix(i.r) - 1
    fn next(i: Pos, s: Int) -> ?Int:
        guard let c = a[i] else: return some(s)
        if s < start(i) + c: some(s + 1) else: none
    let states = [State(i, next(i, s)) for i in Pos for s in start(i)..=last[i]]

    let invs: [Pos]Z = [Pos] x => if x == Pos.first: 0 else: comb.inv(x.ord)
    var dp: [Pos, 0..states.len]Z = 0
    for i in Pos.rev:
        dp[i, start(i)] = 1
        for j in i..Pos.last! if dp[i, last[j]] != 0:
            let v = dp[i, last[j]]
            let nj = j.next!
            for k in start(nj)..states.len if let nk = states[k].next:
                dp[i, nk] += v * dp[nj, k] * invs[Pos(states[k].owner - i)]
    dp[Pos.first!, last[Pos.last!]] * comb.fact(n)

print(solve())
