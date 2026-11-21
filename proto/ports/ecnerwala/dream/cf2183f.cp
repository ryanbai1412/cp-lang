// Codeforces 2183 F: Jumping Man (dream).
// The sum of squared counts is the number of ordered pairs of jump chains in
// the subtree that spell the same string. f(u, v) counts pairs starting at u
// and v: zero unless the letters match, else one (stop) plus every pair that
// starts strictly below u and strictly below v. A subtree is a preorder
// range, so that is a rectangle of f, and the answer for i is sub(i) x sub(i).
type Z = Mod<998244353>

--- multitest ---
let n = read(1..=5000)
domain V = 1..=n
let s: [V]Char = read()
let t = Rooted(read<Tree<V>>(), root: V(1))

var f: RectSums<[t.Pre, t.Pre]Z> = 0          // filled from the bottom-right corner
for (u, v) in (t.at.rev, t.at.rev):
    f[t.pos(u), t.pos(v)] = if s[u] == s[v]: 1 + f.sum(t.below(u), t.below(v)) else: 0
print([f.sum(t.sub(i), t.sub(i)) for i in V])
