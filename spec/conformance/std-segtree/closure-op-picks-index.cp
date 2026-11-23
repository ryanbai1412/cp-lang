// expect: ok, output "1"
// A closure is an op like `min`: here the tree holds positions and keeps the lighter one.
domain V = 0..4
let ws: [V]Int = [3, 1, 4, 1]
var st = SegTree((i: V, j: V) => if ws[i] <= ws[j]: i else: j, [V] v => v)
print(st.all)
