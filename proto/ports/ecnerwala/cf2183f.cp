// Codeforces 2183 F: Jumping Man.
// The sum of squared counts is the number of ordered pairs of jump chains in
// the subtree that spell the same string. Let f(u, v) count pairs whose chains
// start at u and v: zero unless the letters match, else one (stop here) plus
// every pair that starts strictly below u and strictly below v. In preorder a
// subtree is an interval, so "strictly below u and below v" is a rectangle of
// a 2D suffix sum over preorder positions, and the answer for i is the square
// subtree(i) x subtree(i).
type Z = Mod<998_244_353>

@!multitest
let n = read(1..=5000)
domain V = 1..=n
let s: [V]Char = read()
let t = Rooted(Graph.undirected(read<[0..n - 1](V, V)>()), root: V(1))
domain P = 0..=n
let lo = [V] v => t.pos(v).to_idx
let hi = [V] v => lo[v] + t.size(v)

// suf[x, y]: the sum of f(u, v) over preorder lo[u] >= x, lo[v] >= y.
var suf: [P, P]Z = 0
fn rect(x0: Int, x1: Int, y0: Int, y1: Int) -> Z =
    suf[P(x0), P(y0)] - suf[P(x1), P(y0)] - suf[P(x0), P(y1)] + suf[P(x1), P(y1)]
for u in t.at.rev:
    for v in t.at.rev:
        let (x, y) = (lo[u], lo[v])
        let f = if s[u] == s[v]: 1 + rect(x + 1, hi[u], y + 1, hi[v]) else: 0
        suf[P(x), P(y)] = f + suf[P(x + 1), P(y)] + suf[P(x), P(y + 1)] - suf[P(x + 1), P(y + 1)]
print([rect(lo[v], hi[v], lo[v], hi[v]) for v in V])
