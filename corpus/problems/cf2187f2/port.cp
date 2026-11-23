// Codeforces 2187 F2: Common Trees.
// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// Increasing and prime nodes multiply their children. Down a decreasing chain,
// ends(v)[k] counts v's ways with k single-vertex right children still free to
// climb; each such child adds a slot and lets every open one move up.
use std.seq.{inverse}
use std.tree.{PermTree, PermKind}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=5000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree([I] i => pos_a[b[I.at(n - 1 - i.ord)!]])

fn ends<Node>(v: Node) -> List<Z>:
    if t.kind[v] == PermKind.Leaf: return [Z(1)]
    let (l, r) = (t.left(v), t.right(v))
    if t.kind[v] != PermKind.Decr: return [ways(l) * ways(r)]
    var res = ends(l)
    res.push(0)
    if t.kind[r] == PermKind.Leaf:
        let pre = res.scan(+)
        [pre.prefix(i + 1) for i in res.keys]
    else:
        let wr = ways(r)
        res.map(e => e * wr)

fn ways<Node>(v: Node) -> Z = ends(v).sum()
print(ways(t.root))
