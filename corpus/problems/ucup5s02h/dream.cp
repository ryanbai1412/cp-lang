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
use std.math.{Frac}

struct Charge(x: Int, q: Int, pin: ?Side)
enum Side: A | B

// d^2 / (q * q'): larger is weaker.
fn gap(a: Charge, b: Charge) -> Frac = Frac((a.x - b.x) ** 2, a.q * b.q)

// Pull of `c` on a unit charge at `x`: q / (x - c.x)^2.
fn pull(c: Charge, x: Int) -> Frac = Frac(c.q, (x - c.x) ** 2)

// True iff `b` is never weaker than both `a` and `c` for any `x > c.x`.
// Equating `pull(a, x) = pull(b, x)` and `pull(b, x) = pull(c, x)` gives
// crossover points `x_bc <= x_ab` iff `sqrt(a2) >= sqrt(a1) + sqrt(a3)` where
// `a1 = (c.x - b.x)^2 * a.q`, `a2 = (c.x - a.x)^2 * b.q`, `a3 = (b.x - a.x)^2 * c.q`.
// Squaring twice removes the square roots: `a2 >= a1 + a3` and `(a2 - a1 - a3)^2 >= 4 * a1 * a3`.
fn never_weakest(a: Charge, b: Charge, c: Charge) -> Bool:
    let (a1, a2, a3) = ((c.x - b.x) ** 2 * a.q, (c.x - a.x) ** 2 * b.q, (b.x - a.x) ** 2 * c.q)
    a2 >= a1 + a3 and (a2 - a1 - a3) ** 2 >= 4 * a1 * a3

struct Hull<Id>:
    pts: List<(ch: Charge, id: Id)>
    idx: Int

    fn advance(mut self, x: Int):
        while let next = pts.get(idx + 1) and pull(next.ch, x) <= pull(pts[idx].ch, x): idx += 1

    fn add(mut self, ch: Charge, id: Id):
        if pts.last.all(l => ch.q < l.ch.q):
            // Keep `idx + 1 < pts.len` so `pts[idx]` is never popped under the monotone query pointer.
            pts.pop_while(l => idx + 1 < pts.len and never_weakest(pts[-2].ch, l.ch, ch))
            pts.push((ch, id))
        self.advance(ch.x)

    fn query(mut self, x: Int) -> ?(ch: Charge, id: Id):
        self.advance(x)
        pts.get(idx)

// Weakest same-group pairs found by sweeping one side's charges through a `Hull`.
fn hull_pairs(cs: List<Charge>) -> List<(Charge, Charge)>:
    var h = Hull<Int>([], 0)
    var pairs: List<(Charge, Charge)> = []
    for (i, c) in cs.items:
        if let prev = h.query(c.x): pairs.push((prev.ch, c))
        h.add(c, i)
    pairs

@!multitest
let n: 1..=2e5 = read()
domain C = 0..n
let ch: [C]Charge = read<[C](q: Int, x: Int, pin: ?Side)>()
    .map(r => Charge(r.x, r.q, r.pin))
    .sorted(by: c => (c.x, c.q))
domain Node = C | Pin

var colour = ParityDsu<Node>()
for c in C if let s = ch[c].pin: colour.unite(c, Pin, differ: s == A)
if colour.size(Pin) == 1: colour.unite(C.first, Pin, differ: false)

while colour.components > 1:
    let comp = colour.component_ids()
    var best: [comp.groups]?(gap: Frac, i: C, j: C) = None
    for sweep in [C.ascending, C.descending.mirrored(x: 0)]:
        var hulls = GroupTree<Hull<C>>(comp.groups)
        for c in sweep:
            let cands = [(gap: gap(h.ch, ch[c]), i: c, j: h.id) for h in hulls.others(of: comp[c]).query(ch[c].x)]
            if let cand = min_by(cands, by: e => e.gap):
                if best[comp[c]].all(b => cand.gap > b.gap): best[comp[c]] = Some(cand)
            hulls.path(of: comp[c]).add(ch[c], c)
    for opt in best if let e = opt: colour.unite_new(e.i, e.j)

let weakest = min(
    gap(prev, c)
    for side in Side for (prev, c) in hull_pairs([ch[c] for c in C if colour.side(c, of: Pin) == side])
) ?? Frac(1, 226_000_000)
print(weakest.inv)
