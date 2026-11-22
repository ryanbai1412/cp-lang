// CCO 2020 "Interval Collection": add and remove intervals; after each change
// print the shortest span over all sets whose intersection is smallest. If
// two intervals are disjoint the intersection is empty and the best span is the
// closest disjoint pair, kept by a segment tree over coordinates: per point the
// largest left end of intervals ending there and the smallest right end of
// intervals starting there. Otherwise the intersection is fixed by the
// smallest right end and the largest left end.
enum Op = Add(l: X, r: X) | Remove(l: X, r: X)
domain X = 1..=1e6

struct Gap(end_l: ?X, start_r: ?X, span: ?Int)

fn join(a: Gap, b: Gap) -> Gap:
    let across = if a.end_l is some(l) and b.start_r is some(r): some(Int(r) - Int(l)) else: none
    Gap(
        end_l: [a.end_l, b.end_l].filter_map(x => x).max(),
        start_r: [a.start_r, b.start_r].filter_map(x => x).min(),
        span: [a.span, b.span, across].filter_map(x => x).min(),
    )

let q = read<1..=5e5>()
var gaps = SegTree(join, [X] _ => Gap(end_l: none, start_r: none, span: none))
var lefts_ending = Map<X, SortedBag<X>>()
var rights_starting = Map<X, SortedBag<X>>()
var by_end = SortedBag<(r: X, l: Desc<X>)>()
var by_start = SortedBag<(l: Desc<X>, r: X)>()
var lens = SortedBag<Int>()

fn refresh(l: X, r: X):
    let at_r = gaps.get(r)
    gaps.set(r, Gap(end_l: lefts_ending.get(r).and_then(b => b.last), start_r: at_r.start_r, span: none))
    let at_l = gaps.get(l)
    gaps.set(l, Gap(end_l: at_l.end_l, start_r: rights_starting.get(l).and_then(b => b.first), span: none))

for op in read<[q]Op>():
    match op:
        Add(l, r) =>
            lefts_ending.get_or_insert(r).add(l)
            rights_starting.get_or_insert(l).add(r)
            by_end.add((r: r, l: desc(l)))
            by_start.add((l: desc(l), r: r))
            lens.add(Int(r) - Int(l))
            refresh(l, r)
        Remove(l, r) =>
            lefts_ending[r].remove(l)
            rights_starting[l].remove(r)
            by_end.remove((r: r, l: desc(l)))
            by_start.remove((l: desc(l), r: r))
            lens.remove(Int(r) - Int(l))
            refresh(l, r)
    if gaps.fold(X.range).span is some(s):
        print(s)
    else:
        let (a, b) = (by_end.first!, by_start.first!)
        print(if Int(a.r) == Int(b.l.x): lens.first! else: Int(b.r) - Int(a.l.x))
