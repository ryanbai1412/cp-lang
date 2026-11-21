// Codeforces 2035 F: the fewest operations that zero every node, operation i acting in the subtree of node i mod n.
// With t operations node v owns ⌊t / n⌋ + [v <= t mod n] of them, and t works iff the need folded up from
// the leaves (a_v plus the children's need, minus what v owns; a surplus only cancels in pairs) is 0 at
// the root. If t works so does t + 2n, so each residue mod 2n binary-searches its number of rounds.
@!multitest
domain V = read(1..=2000)
let root: V = read()
let a: [V]0..=1e9 = read()
let t = Rooted(Graph<V>.undirected(read<[V.count - 1](V, V)>()), root)

fn settle(need: Int) = if need < 0: need % 2 else: need

fn works(ops: Int) -> Bool:
    let (round, rest) = ops.divmod(V.count)
    t.up(+, finish: (s, v) => settle(s + a[v] - round - (v <= rest).int))[root] == 0

let cycle = 2 * V.count
let rounds = 0..=2 * (V.count + a.max())
print(min(k * cycle + r for r in 0..cycle if let k = rounds.bsearch_first(k => works(k * cycle + r))) ?? -1)
