// Sort items by value. Cut c (values <= c vs > c) is free when no item has one value
// on each side; free cuts split the items into SCCs, each reaching all lower ones.
// So the answer is n(n+1)/2 plus sum d(d-1)/2 over gaps d between consecutive free
// cuts. Item i covers cuts min(p, s) ..< max(p, s): keep, under range add, the cuts
// at the minimum cover (always 0, since cuts 0 and n are free) and their gap sum.
use std.math.choose2

@!multitest
let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]Cut = read()
var s: [Item]Cut = read()

struct Low(cover: Int, first: Cut, last: Cut, gaps: Int)

fn merge(a: ?Low, b: ?Low) -> ?Low:
    let a = a ?? return b
    let b = b ?? return a
    if a.cover < b.cover: return a
    if b.cover < a.cover: return b
    Low(a.cover, a.first, b.last, a.gaps + b.gaps + choose2(b.first - a.last))

let act = Action<?Low, Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x, _) => if let l = x: Low(l.cover + f, l.first, l.last, l.gaps) else: none,
    over: Monoid(id: none, op: merge),
)
var low = LazySegTree<Cut, ?Low>(act, [Cut] c => Low(0, c, c, 0))

fn cover(i: Item, d: Int):
    let lo = min(p[i], s[i])
    let hi = max(p[i], s[i])
    low.apply(lo..hi, d)

for i in Item: cover(i, 1)
repeat q:
    let (t, i, j): (1..=2, Item, Item) = read()
    for x in [i, j]: cover(x, -1)
    if t == 1: p.swap(i, j) else: s.swap(i, j)
    for x in [i, j]: cover(x, 1)
    print(choose2(n + 1) + low.all!.gaps)
