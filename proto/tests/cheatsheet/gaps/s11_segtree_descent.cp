domain I = 0..4
let a: [I]Int = [1, 2, 3, 4]
var st = SegTree<I, Int>(+, a)
print(st.max_right(I(0), s => s <= 3), st.min_left(I(3), s => s <= 7))
