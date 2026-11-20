// EGOI '22 P4 (Tourists): range-move tourists on a tree with city-wide energy bonuses.
// Rooted<City> supplies O(1) tree distances; an ordered Map maintains maximal tourist
// intervals (ODT) via .last_le / .last_lt; each city owns a lazy-add treap over a flat
// arena (using v2.1 RHS-before-LHS evaluation) and point queries walk down without splitting.
// original: corpus/submissions/5199667.cpp

let (n, m, q): (1..=200_000, 1..=200_000, 1..=200_000) = read()
domain City = 1..=n
domain Pos = 1..=m

let init_city: [Pos]City = read()
let edges: [1..n](City, City) = read()
let tree = Rooted<City>(Graph.undirected(edges), City.first)

type Id = 0..=200_005
struct Node:
    pos: Pos
    pri: U64
    val: Int = 0
    lazy: Int = 0
    lc: ?Id = none
    rc: ?Id = none

var rng: U64 = 0x9e3779b97f4a7c15
fn next_pri() -> U64:
    rng ^= rng << 13
    rng ^= rng >> 7
    rng ^= rng << 17
    rng

var nodes: List<Node> = []
var root: [City]?Id = none

fn push(u: Id):
    let d = nodes[u].lazy
    if d == 0: return
    nodes[u].val += d
    if let l = nodes[u].lc: nodes[l].lazy += d
    if let r = nodes[u].rc: nodes[r].lazy += d
    nodes[u].lazy = 0

fn split(o: ?Id, key: Pos) -> (?Id, ?Id):
    if o == none: return (none, none)
    let u = o!
    push(u)
    if key <= nodes[u].pos:
        let (l, r) = split(nodes[u].lc, key)
        nodes[u].lc = r
        (l, u)
    else:
        let (l, r) = split(nodes[u].rc, key)
        nodes[u].rc = l
        (u, r)

fn merge(a: ?Id, b: ?Id) -> ?Id:
    if a == none: return b
    if b == none: return a
    let (u, v) = (a!, b!)
    if nodes[u].pri < nodes[v].pri:
        push(u)
        nodes[u].rc = merge(nodes[u].rc, v)
        u
    else:
        push(v)
        nodes[v].lc = merge(u, nodes[v].lc)
        v

fn split_range(o: ?Id, l: Pos, r: Pos) -> (?Id, ?Id, ?Id):
    let (a, bc) = split(o, l)
    let (b, c) = if let r1 = r + 1: split(bc, r1) else: (bc, none)
    (a, b, c)

fn move_add(l: Pos, r: Pos, old_c: City, new_c: City, delta: Int):
    let (a, mid, c) = split_range(root[old_c], l, r)
    root[old_c] = merge(a, c)
    if let u = mid: nodes[u].lazy += delta
    let (nl, nr) = split(root[new_c], l)
    root[new_c] = merge(merge(nl, mid), nr)

fn query_energy(p: Pos, c: City) -> Int:
    var cur = root[c]
    var acc = 0
    while let u = cur:
        acc += nodes[u].lazy
        if p == nodes[u].pos: return acc + nodes[u].val
        cur = if p < nodes[u].pos: nodes[u].lc else: nodes[u].rc
    0

var odt = Map<Pos, (r: Pos, city: City)>()
for p in Pos:
    let c = init_city[p]
    odt[p] = (p, c)
    let id: Id = nodes.push_id(Node(pos: p, pri: next_pri()))
    root[c] = merge(root[c], id)

fn assign_range(l: Pos, r: Pos, x: City):
    var cur = odt.last_le(r)
    while let (cl, seg) = cur:
        if seg.r < l: break
        cur = odt.last_lt(cl)
        let (cr, px) = seg
        let ov_l = max(l, cl)
        let ov_r = min(r, cr)
        move_add(ov_l, ov_r, px, x, -tree.dist(px, x))
        if cl < l:
            odt[cl] = ((l - 1)!, px)
            if r < cr: odt[(r + 1)!] = (cr, px)
        elif r < cr:
            odt.remove(cl)
            odt[(r + 1)!] = (cr, px)
        else:
            odt.remove(cl)
    odt[l] = (r, x)

for _ in 0..q:
    let op: Char = read()
    if op == 't':
        let (l, r, x): (Pos, Pos, City) = read()
        assign_range(l, r, x)
    elif op == 'e':
        let (c, v): (City, Int) = read()
        if let u = root[c]: nodes[u].lazy += v
    else:
        let p: Pos = read()
        let (_, seg) = odt.last_le(p)!
        print(query_energy(p, seg.city))
