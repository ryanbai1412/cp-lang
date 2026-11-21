// Codeforces 2183 F: Jumping Man.
// The sum of squared counts is the number of ordered pairs of jump chains in
// the subtree that spell the same string. Let f(u, v) count pairs whose chains
// start at u and v: zero unless the letters match, else one (stop here) plus
// every pair that starts strictly below u and strictly below v. In preorder a
// subtree is a range, so "strictly below u and below v" is a rectangle of a 2D
// suffix sum over the cuts of t.Pre, and the answer for i is sub(i) x sub(i).
use std.graph.{Tree}
type Z = Mod<998_244_353>

@!multitest
let n = read(1..=5000)
domain V = 1..=n
let s: [V]Char = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V(1))

// suf[x, y]: the sum of f(u, v) over the positions of u right of cut x and of v right of y.
var suf: [Bounds<t.Pre>, Bounds<t.Pre>]Z = 0
// R is Range<t.Pre>, which a signature cannot name yet.
fn rect<R>(a: R, b: R) -> Z:
    suf[a.start, b.start] - suf[a.end, b.start] - suf[a.start, b.end] + suf[a.end, b.end]
for u in t.at.rev:
    for v in t.at.rev:
        let (x, y) = (t.pos(u), t.pos(v))
        let f = if s[u] == s[v]: 1 + rect(t.below(u), t.below(v)) else: 0
        suf[x.l, y.l] = f + suf[x.r, y.l] + suf[x.l, y.r] - suf[x.r, y.r]
print([rect(t.sub(v), t.sub(v)) for v in V])
