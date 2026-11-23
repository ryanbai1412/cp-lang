// Codeforces 2183 F: Jumping Man.
// The sum of squared counts is the number of ordered pairs of jump chains in
// the subtree that spell the same string. Let f(u, v) count pairs whose chains
// start at u and v: zero unless the letters match, else one (stop here) plus
// every pair that starts strictly below u and strictly below v. In preorder a
// subtree is a range, so "strictly below u and below v" is a rectangle of a 2D
// suffix sum over the cuts of t.Pre, and the answer for i is sub(i) x sub(i).
type Z = Mod<998_244_353>

@!multitest
let n = read(1..=5000)
domain V = 1..=n
let s: [V]Char = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V(1))

var f = Prefix<t.Pre, t.Pre, Z>.filling(order: FillOrder.Reverse)
for u in t.at.rev:
    let (x, su, bu) = (t.pos(u), s[u], t.below(u))
    for v in t.at.rev:
        let val = if su == s[v]: 1 + f.fold(bu, t.below(v)) else: 0
        f.set(x, t.pos(v), val)
print([f.fold(t.sub(v), t.sub(v)) for v in V])
