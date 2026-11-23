// expect: ok, output "13 45"
// Range add, range sum is `SegTree(+, a, act: add)` (spec/lib/ops-ranges.md, the RangeFenwick replacement).
domain V = 0..5
let a: [V]Int = [1, 2, 3, 4, 5]
var st = SegTree(+, a, act: add)
st.apply(1..4, 10)
print(st[2], st.fold(0..5))
