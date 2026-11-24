// Source: Codeforces 2245F (Familiar?), https://codeforces.com/contest/2245/problem/F
// limits: 5000 ms, 128 MB
// Problem: mystery(p) pushes p[1..n] on a stack, first popping every larger top; b[i] is how many
//   it pops for p[i]. Given a[1..n] (-1 = free), count permutations p of 1..n with b[i] = a[i]
//   wherever a[i] != -1, mod 998244353. n <= 500, sum n^3 <= 500^3, memory 128 MB. Multitest.
// Solution: who pops whom forms a tree whose permutations number n! / prod subtree sizes
//   (hook length), so the DP sums 1 / prod size rather than interleaving binomials. Expand each
//   position into states: a known count c is a chain of c + 1 states, an unknown one a single
//   self-loop. dp[i, s] sums subtrees on [i, owner(s)); splitting at j glues [i, j] to
//   [j + 1, owner(k)) times 1 / (owner(k) - i). O(n) states, O(n^3); answer dp * n!.
type Z = Mod<998_244_353>
let comb = Comb<Z>()

struct State(owner: Pos, next: ?Int)

@!multitest
let n: 1..=500 = read()
domain Pos = 0..=n
let given: [0..n]?Int = read(none: -1)
let a: [Pos]?Int = [Pos] i => if i.ord < n: given[i.ord] else: None

fn solve() -> Z:
    let ends = prefix([Pos] i => (a[i] ?? 0) + 1)
    for i in Pos:
        if ends.fold(..=i) - (i.ord + 1) > i.ord: return 0

    fn start(i: Pos) -> Int = ends.prefix(i.l)
    let last = [Pos] i => ends.prefix(i.r) - 1
    fn next(i: Pos, s: Int) -> ?Int:
        guard let c = a[i] else: return Some(s)
        if s < start(i) + c: Some(s + 1) else: None
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
