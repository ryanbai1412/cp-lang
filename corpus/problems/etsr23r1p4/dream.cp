// etsr23r1p4: range add, range floor division, range min and range sum. Floor division is the
// segment-tree-beats operation: a node stops once its whole span moves by one shared amount.
let (n, q) = read<(1..=200_000, 1..=200_000)>()
domain I = 1..=n
var t = BeatsSegTree(read<[I]Int>())
enum Op:
    @tag("1") Add(r: Range<I>, x: Int)
    @tag("2") Div(r: Range<I>, d: 1..=1_000_000_000)
    @tag("3") Min(r: Range<I>)
    @tag("4") Sum(r: Range<I>)
for op in read<[q]Op>():
    match op:
        Add(r, x) => t.add(r, x)
        Div(r, d) => t.chdiv(r, d)
        Min(r) => print(t.min_val(r)!)
        Sum(r) => print(t.sum(r))
