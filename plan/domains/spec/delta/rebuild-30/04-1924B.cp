// cf1924b Space Harbour: a ship at i strictly between harbours L < i < R costs
// val[L] * (R - i), a line in i. Keep the costs in a lazy segment tree whose tag
// assigns a + b * i over a range; each node folds (sum, sum of indices).
let (n, m, q): (2..=3e5, 2..=n, 1..=3e5) = read()
domain P = 1..=n
let xs = read<[m]P>()
let vs = read<[m]1..=1e7>()

struct Cost:
    s: Int
    idx: Int

let both = Monoid(id: Cost(0, 0), op: (x, y) => Cost(x.s + y.s, x.idx + y.idx))
let line = Action(
    id: None,
    compose: (nf: ?(Int, Int), of: ?(Int, Int)) => nf ?? of,
    apply: (f: ?(Int, Int), c: Cost, span) => if let (a, b) = f: Cost(a * span + b * c.idx, c.idx) else: c,
    over: both,
)
var cost = SegTree(both, [P] i => Cost(0, Int(i)), act: line)

var val: [P]Int = 0
var harbours = SortedSet<P>.new()
for (x, v) in zip(xs, vs):
    val[x] = v
    harbours.set(x, true)

fn paint(l: P, r: P):
    cost.apply(l.next!..r, Some((val[l] * Int(r), -val[l])))

for (l, r) in harbours.each().adjacent: paint(l, r)

enum Query:
    @tag(1) Add(x: P, v: 1..=1e7)
    @tag(2) Ask(l: P, r: P)

for cmd in read<[q]Query>():
    match cmd:
        Add(x, v) =>
            let l = harbours.last_true(h => h < x)!
            let r = harbours.first_true(h => h > x)!
            val[x] = v
            harbours.set(x, true)
            cost.apply(x..=x, Some((0, 0)))
            paint(l, x)
            paint(x, r)
        Ask(l, r) => print(cost.fold(l..=r).s)
