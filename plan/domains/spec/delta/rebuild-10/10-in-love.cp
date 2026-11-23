// CF 1883D In Love
// in: q, then q lines "+ l r" or "- l r" | out: after each, YES if two segments of the multiset are disjoint, else NO
enum Op = @tag("+") Add(l: 1..=1e9, r: 1..=1e9) | @tag("-") Del(l: 1..=1e9, r: 1..=1e9)

var lefts: SortedBag<Int> = SortedBag()
var rights: SortedBag<Int> = SortedBag()
for _ in 0..read<1..=1e5>():
    match read<Op>():
        Add(l, r) =>
            lefts.add(l)
            rights.add(r)
        Del(l, r) =>
            lefts.remove(l)
            rights.remove(r)
    print(yes(lefts.len > 0 and rights.first! < lefts.last!))
