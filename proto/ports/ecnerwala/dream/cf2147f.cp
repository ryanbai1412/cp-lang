// Sort items by value. Cut c (values ≤ c vs > c) is free when no item has one value
// on each side; free cuts split the items into SCCs, each reaching all lower ones.
// So the answer is n(n+1)/2 plus Σ d(d−1)/2 over gaps d between consecutive free
// cuts. Item i covers cuts min(p, s)..max(p, s): keep, under range add, the cuts
// at the minimum cover (always 0, since cuts 0 and n are free) and their gap sum.
use std.algebra.Action

@!multitest
let n: 2..=1e5 = read()
let q: 1..=1e5 = read()
domain Item = 1..=n
domain Cut = 0..=n
var p, s: [Item]Cut = read()

struct Low(cover: Int, first: Cut, last: Cut, gaps: Int)

impl Add for Low:
    fn add(a: Low, b: Low) -> Low = match a.cover <=> b.cover:
        less => a
        greater => b
        equal => Low(a.cover, a.first, b.last, a.gaps + b.gaps + choose2(b.first - a.last))

let add_cover = Action<Low, Int>(id: 0, compose: +, over: +, apply: (d, l, _) => (l with cover: l.cover + d))
var low = LazySegTree<Cut>(add_cover, a: [Cut] c => Low(0, c, c, 0))

fn cover(i: Item, by: Int):
    let (lo, hi) = minmax(p[i], s[i])
    low.apply(lo..hi, by)

for i in Item: cover(i, by: 1)
repeat q:
    let (t, i, j): (1..=2, Item, Item) = read()
    for x in [i, j]: cover(x, by: -1)
    if t == 1: p.swap(i, j) else: s.swap(i, j)
    for x in [i, j]: cover(x, by: 1)
    print(choose2(n + 1) + low.all.gaps)
