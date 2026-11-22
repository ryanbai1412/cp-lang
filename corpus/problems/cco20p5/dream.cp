// CCO 2020 "Interval Collection": add and remove intervals; after each change
// print the shortest span over all sets whose intersection is smallest.
enum Op = Add(l: Int, r: Int) | Remove(l: Int, r: Int)

@journal
struct DisjointPair:
    left_of: Fenwick<1..=1e6, Int>
    right_of: Fenwick<1..=1e6, Int>
    best: Int = inf

    fn add(mut self, iv: Range<1..=1e6>):
        let (l, r) = (Int(iv.l), Int(iv.r))
        self.best = min(self.best, r - (self.left_of.fold(..=l) ?? -inf), (self.right_of.fold(r..) ?? inf) - l)
        self.left_of.apply(r, l)
        self.right_of.apply(l, r)

let q = read<1..=5e5>()
let ops = read<[q]Op>()
let disjoint = timeline(ops, add: o => o.range, state: DisjointPair(left_of: Fenwick(max), right_of: Fenwick(min)))

var by_end = SortedBag<(r: Int, l: Desc<Int>)>()
var by_start = SortedBag<(l: Desc<Int>, r: Int)>()
var lens = SortedBag<Int>()
for (t, op) in ops.items:
    match op:
        Add(l, r) => { by_end.add((r, desc(l))); by_start.add((desc(l), r)); lens.add(r - l) }
        Remove(l, r) => { by_end.remove((r, desc(l))); by_start.remove((desc(l), r)); lens.remove(r - l) }
    var best = (overlap: lens.first!, span: lens.first!)
    let (a, b) = (by_end.first!, by_start.first!)
    if a.r >= b.l.0:
        best = min(best, (overlap: a.r - b.l.0, span: b.r - a.l.0))
    if disjoint[t].best < inf:
        best = min(best, (overlap: 0, span: disjoint[t].best))
    print(best.span)
