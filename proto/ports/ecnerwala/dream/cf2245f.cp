// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states,
// multiplying by inv(owner - i) at each split and n! at the end.
//
// Each position i owns a run of states: a fixed count c gives a chain of
// c + 1 states, each stepping to the next and the last ending it; an unknown
// count (and the padding position n) gives one state that steps to itself.
// `dp[i, s]` is built from the interval starting at i, reaching state s.
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
    if a.map(c => c ?? 0).scan(+).enumerate.any((i, t) => t > i): 0
    else: dp(0, runs[n].last) * comb.fact(n)
)

@memo
fn dp(i: Pos, s: State) -> Z:
    if s == runs[i].first: return 1
    sum(
        dp(i, runs[j].last) * dp(j + 1, k) * comb.inv(k.owner - i)
        for j in i..n for k in runs.items(from: j + 1) if next(k) == some(s)
    )
