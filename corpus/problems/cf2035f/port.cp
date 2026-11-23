// Source: Codeforces 2035F (Tree Operations), https://codeforces.com/contest/2035/problem/F
// Problem: a tree on n nodes rooted at x, values a_i <= 1e9 (multitest, sum of n <= 2000).
//   Operation i acts on node ((i - 1) mod n) + 1 and adds or subtracts 1 at one node of its
//   subtree; none may be skipped. Print the fewest operations zeroing every value, or -1.
// Solution: with t operations node v owns floor(t / n) + [v <= t mod n] of them, and t works
//   iff the need folded up from the leaves (a_v plus the children's need minus what v owns;
//   a surplus only cancels in pairs) is 0 at the root. If t works so does t + 2n, so each
//   residue mod 2n binary-searches its number of rounds; print the least t found.
//   O(n^2 log(n + max a)) per test.
@!multitest
let n: 1..=2000 = read()
domain V = 1..=n
let root: V = read()
let a: [V]0..=1e9 = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), root)

fn settle(need: Int) -> Int = if need < 0: -need % 2 else: need

fn works(ops: Int) -> Bool:
    t.up(+, finish: (s, v) => settle(s + a[v] - ops / n - Int(v.label <= ops % n)))[root] == 0

let cycle = 2 * n
let rounds = 0..=2 * (n + a.max())
print(
    [
        k * cycle + r for r in 0..cycle if let k = rounds.first_true(k => works(k * cycle + r))
    ].min() ?? -1
)
