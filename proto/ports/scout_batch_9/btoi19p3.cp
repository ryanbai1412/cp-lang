// original: corpus/submissions/4456164.cpp
// BOI 2019 Valley: villages joined into a tree by roads with lengths. Some
// have shops, and one is the way out of the valley. Each query closes one
// road and names a village: can it still get out, and if not, how far is
// its nearest shop?
let n: 2..=1e5 = read()
let s: 1..=n = read()
let q: 1..=1e5 = read()
domain Village = 1..=n
domain Road = 1..n
let out: Village = read()

fn random_tree() -> [Road](u: Village, v: Village, len: 1..=1e9):
    [Road] r => (Village(rand(1, r.label)), Village(r.label + 1), rand(1, 1000000000))

let roads: [Road](u: Village, v: Village, len: 1..=1e9) = read(gen: random_tree)
let shops: [s]Village = read()
let tree = Rooted(roads, out, weight: .len)
let far: [Village]Int = [Village] v => tree.dist(out, v)       // the way from v to the exit

// A closed road traps the subtree under it. From x, the walk to a shop
// climbs to some y and then goes down inside y's subtree, at a cost of
// far[x] - far[y] plus y's nearest shop below. Keep the part that depends
// on y, laid out so that a climb is a few ranges.
var shop_below: [Village]?Int = none
for v in shops: shop_below[v] = 0
for v in tree.preorder.rev:
    if let p = tree.parent(v) and let d = shop_below[v]: shop_below[p] min= d + far[v] - far[p]
var turn = SegTree<tree.Pre, ?Int>(none, (a, b) => min(a, b))
for v in Village:
    if let d = shop_below[v]: turn.set(tree.sub(v).l, d - far[v])

for _ in 0..q:
    let closed: Road = read()
    let x: Village = read()
    let (u, v, _) = roads[closed]
    let bottom = if tree.is_anc(u, v): v else: u
    if not tree.is_anc(bottom, x):
        print("escaped")
    elif let best = min(turn.query(l, r) for (l, r) in tree.chains(bottom, x)):
        print(far[x] + best)
    else:
        print("oo")
