// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states,
// multiplying by inv(owner - i) at each split and n! at the end.
//
// Each position i owns a run of states: a fixed count c gives a chain of
// c + 1 states, each stepping to the next and the last ending it; an unknown
// count (and the padding position n) gives one state that steps to itself.
// `dp[i, s]` is built from the interval starting at i, reaching state s.
type Z = Mod<998_244_353>
let comb = Comb<Z>()

struct State(owner: Int, next: ?Int)

@!multitest
let n: Int = read()
let given: [0..n]?Int = read(none: -1)
let a: [0..=n]?Int = [0..=n] i => if i < n: given[i] else: none

fn solve() -> Z:
    var tot = 0
    for i in 0..=n:
        tot += a[i] ?? 0
        if tot > i: return 0

    var first: [0..=n + 1]Int = 0
    for i in 0..=n: first[i + 1] = first[i] + (a[i] ?? 0) + 1
    let last = [0..=n] i => first[i + 1] - 1
    fn next(i: Int, s: Int) -> ?Int:
        guard let c = a[i] else: return some(s)
        if s < first[i] + c: some(s + 1) else: none
    let states = [State(i, next(i, s)) for i in 0..=n for s in first[i]..first[i + 1]]

    let invs: [0..=n]Z = [0..=n] x => if x == 0: 0 else: comb.inv(x)
    var dp: [0..=n, 0..states.len]Z = 0
    for i in (0..=n).rev:
        dp[i, first[i]] = 1
        for j in i..n if dp[i, last[j]] != 0:
            let v = dp[i, last[j]]
            for k in first[j + 1]..states.len if let nk = states[k].next:
                dp[i, nk] += v * dp[j + 1, k] * invs[states[k].owner - i]
    dp[0, last[n]] * comb.fact(n)

print(solve())
