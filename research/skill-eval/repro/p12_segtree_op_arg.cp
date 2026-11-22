// Repro 1: SegTree<D, Int>(+, a) fails E0302 even though stdlib/algebra.md documents it
domain I = 1..=3
let a = [I] i => Int(i)
var st = SegTree<I, Int>(+, a)
print(st.fold(I(1)..=I(3)))
