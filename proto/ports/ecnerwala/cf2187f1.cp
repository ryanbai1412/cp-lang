// Codeforces 2187 F1: Al Fine (maximizing version).
use std.seq.{inverse}

enum Kind: Leaf, Decr, Other
type Node = (kind: Kind, l: Int, r: Int)
type Open = (left: Int, lo: Int, lo_gap: Int, hi: Int, hi_gap: Int, node: Int)

@!multitest
let n: 2..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let p: [0..n]Int = [0..n] i => pos_a[b[I(n - 1 - i)]].to_idx

var prv: [0..n]Int = [0..n] v => v - 1
var nxt: [0..n]Int = [0..n] v => v + 1
for i in (0..n).rev:
    let (l, r) = (prv[p[i]], nxt[p[i]])
    if l >= 0: nxt[l] = r
    if r < n: prv[r] = l

var nodes: [0..2 * n - 1]Node = (kind: Leaf, l: -1, r: -1)
var stk = List<Open>()
fn join(u: Open, t: Open, kind: Kind) -> Open:
    let id = 2 * t.left - 1
    nodes[id] = (kind: kind, l: u.node, r: t.node)
    (
        left: u.left,
        lo: min(u.lo, t.lo),
        lo_gap: u.lo_gap,
        hi: max(u.hi, t.hi),
        hi_gap: u.hi_gap,
        node: id,
    )
for i in 0..n:
    let x = p[i]
    while let top = stk.last, x < top.lo_gap or x > top.hi_gap:
        let t = stk.pop()!
        stk.push(join(stk.pop()!, t, Other))
    var t = (left: i, lo: x, lo_gap: prv[x] + 1, hi: x, hi_gap: nxt[x] - 1, node: 2 * i)
    while let u = stk.last, max(u.hi, t.hi) - min(u.lo, t.lo) == i - u.left:
        t = join(stk.pop()!, t, if t.lo < u.lo: Decr else: Other)
    stk.push(t)

fn depth(v: Int) -> Int:
    let node = nodes[v]
    if node.kind == Leaf: return 1
    if node.kind == Decr and nodes[node.r].kind == Leaf: return 1 + depth(node.l)
    max(depth(node.l), depth(node.r))
print(depth(stk[0].node))
