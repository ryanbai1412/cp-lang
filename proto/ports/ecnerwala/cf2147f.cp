@!multitest
struct Node(mn: Int, first: Int, last: Int, pairs: Int)

fn merge(a: ?Node, b: ?Node) -> ?Node:
    let a = a ?? return b
    let b = b ?? return a
    if a.mn < b.mn: return a
    if b.mn < a.mn: return b
    let d = b.first - a.last
    Node(a.mn, a.first, b.last, a.pairs + b.pairs + d * (d - 1) / 2)

let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]1..=n = read()
var s: [Item]1..=n = read()

let act = Action<?Node, Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x, _) => if let v = x: Node(v.mn + f, v.first, v.last, v.pairs) else: none,
    over: Monoid(id: none, op: merge),
)
var seg = LazySegTree<Cut, ?Node>(act, [Cut] c => Node(0, c.to_idx, c.to_idx, 0))

fn update_item(i: Item, delta: Int):
    let lo = min(p[i], s[i])
    let hi = max(p[i], s[i])
    if lo < hi: seg.apply(Cut(lo)..Cut(hi), delta)

for i in Item: update_item(i, 1)

let base = n * (n + 1) / 2
repeat q:
    let (tp, i, j): (1..=2, Item, Item) = read()
    update_item(i, -1)
    update_item(j, -1)
    if tp == 1: (p[i], p[j]) = (p[j], p[i])
    else: (s[i], s[j]) = (s[j], s[i])
    update_item(i, 1)
    update_item(j, 1)
    print(base + seg.all!.pairs)
