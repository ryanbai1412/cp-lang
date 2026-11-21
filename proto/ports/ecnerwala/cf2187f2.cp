// Codeforces 2187 F2: Al Fine (counting version).
enum Kind: Leaf, Decr, Other
type Node = (kind: Kind, l: Int, r: Int)
type Z = Mod<998_244_353>
type Open = (left: Int, lo: Int, lo_gap: Int, hi: Int, hi_gap: Int, node: Int)

@!multitest
let n: 2..=5000 = read()
domain V = 1..=n
let a: [0..n]V = read()
let b: [0..n]V = read()
var pos_a: [V]Int = 0
for i in 0..n: pos_a[a[i]] = i
let p: [0..n]Int = [0..n] i => pos_a[b[n - 1 - i]]

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

fn count(v: Int) -> Z:
    let node = nodes[v]
    if node.kind == Leaf: return 1
    if node.kind != Decr: return count(node.l) * count(node.r)
    var leafs = List<Bool>()
    var ways: Z = 1
    var cur = v
    while nodes[cur].kind == Decr:
        let r = nodes[cur].r
        leafs.push(nodes[r].kind == Leaf)
        if nodes[r].kind != Leaf: ways *= count(r)
        cur = nodes[cur].l
    ways *= count(cur)
    var ends: List<Z> = [1]
    for leaf in leafs.rev:
        ends.push(0)
        if leaf:
            ends = scan(ends, (x, y) => x + y)
    ways * sum(ends)
print(count(stk[0].node))
