@!multitest
let n: 2..=100_000 = read()
let q: 1..=100_000 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]Cut = read()
var s: [Item]Cut = read()

struct Low(cover: Int, first: Cut, last: Cut, gaps: Int)

fn choose2(k: Int) -> Int = k * (k - 1) / 2

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
    if lo < hi: low.apply(lo..hi, d)

for i in Item: cover(i, 1)
repeat q:
    let (t, i, j): (1..=2, Item, Item) = read()
    for x in [i, j]: cover(x, -1)
    if t == 1: (p[i], p[j]) = (p[j], p[i])
    else: (s[i], s[j]) = (s[j], s[i])
    for x in [i, j]: cover(x, 1)
    print(choose2(n + 1) + low.all!.gaps)
