// `SegTree(+, a, act: ...)` as in spec/lib/ops-ranges.md. `cpl check` accepts; `cpl run` rejects `+`.
domain I = 1..=3
var st = SegTree(+, [I] i => Int(i), act: affine)
st.apply(I(1)..=I(2), (1, 10))
print(st.fold(I(1)..=I(3)))
