// Source: Codeforces 2183F (Jumping Man), https://codeforces.com/contest/2183/problem/F
// Problem: a tree rooted at 1, n <= 5000 (sum over tests), a letter per node. For each i,
//   take every chain u_1, u_2, ... in i's subtree with each node strictly inside the previous
//   one's subtree; print the sum over strings of (number of chains spelling it)^2, mod
//   998244353. Multitest.
// Solution: the sum of squared counts is the number of ordered pairs of chains that spell
//   the same string. f(u, v) counts such pairs starting at u and v: 0 unless the letters
//   match, else 1 plus the sum of f over (strictly below u) x (strictly below v). In
//   preorder that is a rectangle of a 2D suffix sum filled in reverse preorder, and the
//   answer for i is the sum over sub(i) x sub(i). O(n^2) time and memory.
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
