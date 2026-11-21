// Codeforces 2035 F: the fewest operations that zero every node, operation i acting in the subtree of node i mod n.
// With t operations node v owns ⌊t / n⌋ + [v <= t mod n] of them, and t works iff the need folded up from
// the leaves (a_v plus the children's need, minus what v owns; a surplus only cancels in pairs) is 0 at
// the root. If t works so does t + 2n, so each residue mod 2n binary-searches its number of rounds.
@!multitest
let n: 1..=2000 = read()
domain V = 1..=n
let root: V = read()
let a: [V]0..=1e9 = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), root)

fn settle(need: Int) -> Int = if need < 0: -need % 2 else: need

fn works(ops: Int) -> Bool:
    let need = t.up(
        +,
        (c, _, _) => c,
        (s, v) => settle(s + a[v] - ops / n - Int(v.label <= ops % n)),
    )
    need[root] == 0

let cycle = 2 * n
let rounds = 0..=2 * (n + a.max())
print(
    [
        k * cycle + r for r in 0..cycle if let k = bsearch_first(rounds, k => works(k * cycle + r))
    ].min() ?? -1
)
