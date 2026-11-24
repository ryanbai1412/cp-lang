// expect: ok, output "5\n8"
// rule: std.segtree.SegTree.new
// A tree can run over a domain's cuts: `D` is `Town.cuts`, and its positions come from the blanket `Positions` impl every domain has.
domain Town = 1..=3
let act = Action<Int, Int>(0, (f, g) => f + g, (f, x, _) => x + f, over: Monoid<Int>(1e9, min))
var st = SegTree(act.over, [Town.cuts] c => 0, act: act)
st.apply(Town.cuts.first!..=Town.cuts.last!, 5)
print(st.fold(Town.cuts.first!..=Town.cuts.last!))
var sum = SegTree(+, [Town.cuts] c => 2)
print(sum.all)
