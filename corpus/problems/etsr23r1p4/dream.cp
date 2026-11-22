// ETSR 2023 R1 P4: range add, range floor-divide, range min, range sum. Floor division is a beats
// update: it recurses until a node's min and max move by the same amount, then it is an add.
domain I = 1..=read Int
let q = read Int
var t = SegTree(each(lo: min, hi: max, sum: +), read [I]Int, act: add)
enum Op: Add(r: Range<I>, x: Int) = 1, Div(r: Range<I>, d: Int) = 2, Min(r: Range<I>) = 3, Sum(r: Range<I>) = 4
for op in read [q]Op:
    match op:
        Add(r, x): t.apply(r, x)
        Div(r, d): t.apply_beats(r, node => node.hi.floor_div(d) - node.hi == node.lo.floor_div(d) - node.lo,
                                 node => node.lo.floor_div(d) - node.lo)
        Min(r): print(t.fold(r).lo)
        Sum(r): print(t.fold(r).sum)
