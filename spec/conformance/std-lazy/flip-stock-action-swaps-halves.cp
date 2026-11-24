// expect: ok, output "1 3"
// rule: std.algebra.Action.of
domain I = 0..4
var st = SegTree(Flip<Int>.op(+), [I] i => Flip.of(i.ord != 2, 1, 0), act: flip)
st.apply(I(0)..I(2), true)
print(st.all.now, st.all.flipped)
