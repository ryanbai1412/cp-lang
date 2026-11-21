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
use std.dsu.{ParityDsu}
use std.math.{Frac}

struct Charge(x: Int, q: Int, pin: ?Side)
enum Side = A | B

// d^2 / (q * q'): larger is weaker.
fn gap(a: Charge, b: Charge) -> Frac = Frac((a.x - b.x) ^ 2, a.q * b.q)

struct Hull:
    pts: List<Charge>
    idx: Int

    fn advance(mut self, x: Int):
        while let next = pts.get(idx + 1) and next.pull(at: x) <= pts[idx].pull(at: x): idx += 1

    fn add(mut self, p: Charge):
        if pts.last.all(l => p.q < l.q):
            pts.pop_while(l => idx + 1 < pts.len and never_weakest(pts[-2], l, p))
            pts.push(p)
        self.advance(p.x)

    fn query(mut self, x: Int) -> ?Charge:
        self.advance(x)
        pts.get(idx)

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
    var best: [comp.groups]?(gap: Frac, i: C, j: C) = none
    for sweep in [C.ascending, C.descending.mirrored(x: 0)]:
        var hulls = GroupTree<Hull>(comp.groups)
        for c in sweep:
            let near = min_by(hulls.others(of: comp[c]).query(ch[c].x), by: h => gap(h, ch[c]))
            if let h = near: best[comp[c]] max_by= (gap(h, ch[c]), c, h.id), by: e => e.gap
            hulls.path(of: comp[c]).add(ch[c])
    for e in best if let e: colour.unite_new(e.i, e.j)

let weakest = min(
    gap(prev, c)
    for side in Side for (prev, c) in hull_pairs(C.filter(c => colour.side(c, of: Pin) == side))
) ?? Frac(1, 226_000_000)
print(weakest.inv)
