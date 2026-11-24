// Universal Cup 5, stage 2 H: split charges on a line into two groups (some
// charges are pinned to A or B) so that the weakest pull q * q' / d^2 between
// two charges of one group is as strong as possible; print it as a fraction.
//
// As for any two-clustering that wants its weakest same-group pair strong, it
// is enough to two-colour the spanning tree of weakest pulls: pinned charges
// hang off a reference node `pin` on A's or B's side. Boruvka builds that tree.
// Each round, a charge finds its weakest pull to another component among the
// charges left of it (then, mirrored, right of it): a segment tree over
// components keeps a `Hull` per node, and climbing from the charge's own leaf
// queries every sibling, i.e. every other component.
//
// `Hull` holds charges added left to right and answers "which of them pulls
// weakest on x" for increasing x. A charge no weaker than the newest and
// farther away never wins, so charges strictly decrease, and the weakest one
// only moves right as x does.
// d^2 / (q * q'): larger is weaker.
// Pull of `c` on a unit charge at `x`: q / (x - c.x)^2.
// True iff `b` is never weaker than both `a` and `c` for any `x > c.x`.
// Equating `pull(a, x) = pull(b, x)` and `pull(b, x) = pull(c, x)` gives
// crossover points `x_bc <= x_ab` iff `sqrt(a2) >= sqrt(a1) + sqrt(a3)` where
// `a1 = (c.x - b.x)^2 * a.q`, `a2 = (c.x - a.x)^2 * b.q`, `a3 = (b.x - a.x)^2 * c.q`.
// Squaring twice removes the square roots: `a2 >= a1 + a3` and `(a2 - a1 - a3)^2 >= 4 * a1 * a3`.
// Weakest same-group pairs found by sweeping one side's charges through a `Hull`.

struct Charge(x: Int, q: Int, pin: Char)

fn gap(a: Charge, b: Charge) -> Ratio = Ratio.new((a.x - b.x) ** 2, a.q * b.q)

// `b` pulls at x no harder than `a`
fn no_stronger(b: Charge, a: Charge, x: Int) -> Bool = (a.x - x) ** 2 * b.q <= (b.x - x) ** 2 * a.q

fn never_weakest(a: Charge, b: Charge, c: Charge) -> Bool:
    let (a1, a2, a3) = ((c.x - b.x) ** 2 * a.q, (c.x - a.x) ** 2 * b.q, (b.x - a.x) ** 2 * c.q)
    a2 >= a1 + a3 and (I128(a2) - I128(a1) - I128(a3)) ** 2 >= I128(4) * I128(a1) * I128(a3)

struct Hull<Id>:
    pts: List<(ch: Charge, id: Id)>
    idx: Int

    fn advance(mut self, x: Int):
        while let next = pts.get(idx + 1) and no_stronger(next.ch, pts[idx].ch, x): idx += 1

    fn add(mut self, ch: Charge, id: Id):
        if pts.is_empty or ch.q < pts.last!.ch.q:
            // Keep `idx + 1 < pts.len` so `pts[idx]` is never popped under the monotone query pointer.
            pts.pop_while(l => idx + 1 < pts.len and never_weakest(pts[pts.len - 2].ch, l.ch, ch))
            pts.push((ch, id))
        self.advance(ch.x)

    fn query(mut self, x: Int) -> ?(ch: Charge, id: Id):
        self.advance(x)
        pts.get(idx)

fn weaker<Id>(a: ?(ch: Charge, id: Id), b: ?(ch: Charge, id: Id), x: Int) -> ?(ch: Charge, id: Id):
    guard let pa = a else: return b
    guard let pb = b else: return a
    if no_stronger(pb.ch, pa.ch, x): b else: a

@!multitest
let n: 2..=150_000 = read()
domain C = 0..n
let sorted = read<[C](q: Int, x: Int, pin: Char)>()
    .map(r => Charge(r.x, r.q, r.pin))
    .sorted(by: c => (c.x, c.q, c.pin))
let ch = [C] c => sorted[c.ord]
domain Node = 0..=n
let pin = Node(n)
fn node(c: C) -> Node = Node(c.ord)

var colour = ParityDsu<Node>()
for c in C if ch[c].pin == 'A' or ch[c].pin == 'B': colour.unite(node(c), pin, differ: ch[c].pin == 'A')
if colour.size(pin) == 1: colour.unite(node(C.first!), pin, differ: false)

while colour.components > 1:
    let groups = colour.groups()
    let v = groups.len
    var comp: [Node]Int = 0
    for (k, g) in groups.items:
        for u in g: comp[u] = k
    var best: [0..v]?(gap: Ratio, i: C, j: C) = None
    // a segment tree over components: each charge asks the hulls of every other component
    for (order, sign) in [(C.collect(), 1), (C.rev.collect(), -1)]:
        var hulls = [Hull<C>([], 0) for _ in 0..2 * v]
        for c in order:
            let x = sign * ch[c].x
            let k = comp[node(c)]
            var hit: ?(ch: Charge, id: C) = None
            var t = k + v
            while t > 1:
                hit = weaker(hit, hulls[t ^ 1].query(x), x)
                t >>= 1
            if let h = hit:
                let cand = gap(ch[h.id], ch[c])
                if (if let b = best[k]: cand >= b.gap else: true): best[k] = Some((cand, c, h.id))
            t = k + v
            while t > 1:
                hulls[t].add(Charge(x, ch[c].q, ch[c].pin), c)
                t >>= 1
    for e in best if let b = e and not colour.same(node(b.i), node(b.j)): colour.unite(node(b.i), node(b.j))

var weakest: ?Ratio = None
for side in [false, true]:
    var hull = Hull<C>([], 0)
    for c in C if (colour.diff(node(c), pin) == Some(false)) == side:
        if let prev = hull.query(ch[c].x):
            let g = gap(prev.ch, ch[c])
            weakest = Some(max(weakest ?? g, g))
        hull.add(ch[c], c)
print((weakest ?? Ratio.new(1, 226_000_000)).inv)
