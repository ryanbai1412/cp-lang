// cp: 0.1 bea787c+dirty
domain I = 0..4
var st = SegTree<I, Int>(0, (a, b) => a + b)
st.set(I(2), 5)
print(st.fold(I(0)..=I(3)))
