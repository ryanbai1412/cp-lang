// CCO 2020 "Interval Collection": add and remove intervals; after each change
// pick the set with no common interval if any (else the smallest one), then the
// shortest enclosing interval, and print its length.
//
// A set with no common interval holds a pair a.r <= b.l, whose span b.r - a.l
// is no longer, so the best such span is a fold over coordinates: per point the
// largest left end of intervals ending there and the smallest right end of
// intervals starting there. Otherwise the common part is [max l, min r] and
// the set is either that pair or one shortest interval.
const LIM = 1e6
type Coord = 1..=LIM
domain X = 1..=LIM
enum Op = @tag("A") Add(l: Coord, r: Coord) | @tag("R") Remove(l: Coord, r: Coord)

struct Gap(end_l: ?Int, start_r: ?Int, span: ?Int)

fn join(a: Gap, b: Gap) -> Gap =
    Gap(
        end_l: max(a.end_l, b.end_l),
        start_r: min(a.start_r, b.start_r),
        span: min(min(a.span, b.span), b.start_r - a.end_l),
    )

let q = read<1..=5e5>()
var gaps = SegTree<X, Gap>(Monoid(Gap(None, None, None), join))
var by_end = SortedBag<(r: Int, l: Desc<Int>)>()
var by_start = SortedBag<(l: Desc<Int>, r: Int)>()
var lens = SortedBag<Int>()

fn refresh(p: Int):
    let end_l = by_end.first_true(e => e.r >= p).filter(e => e.r == p).map(e => e.l.x)
    let start_r = by_start.first_true(e => e.l.x <= p).filter(e => e.l.x == p).map(e => e.r)
    gaps.set(X(p), Gap(end_l:, start_r:, span: start_r - end_l))

for op in read<[q]Op>():
    match op:
        Add(l, r) =>
            by_end.add((r:, l: desc(l)))
            by_start.add((l: desc(l), r:))
            lens.add(r - l)
            refresh(l)
            refresh(r)
        Remove(l, r) =>
            by_end.remove((r:, l: desc(l)))
            by_start.remove((l: desc(l), r:))
            lens.remove(r - l)
            refresh(l)
            refresh(r)
    if let Some(span) = gaps.all.span:
        print(span)
    else:
        let (a, b) = (by_end.first!, by_start.first!)
        let one = lens.first!
        print(min((common: one, span: one), (common: a.r - b.l.x, span: b.r - a.l.x)).span)
